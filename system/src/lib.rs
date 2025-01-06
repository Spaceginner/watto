#![feature(let_chains)]
#![feature(ascii_char)]

pub mod kernels;
pub mod device;

use std::time::{Duration, Instant};
use crate::device::Device;
use crate::kernels::Kernel;

struct Timer {
    delay: Duration,
    left: Duration,
}


impl Timer {
    pub fn new(delay: Duration) -> Self {
        Self { delay, left: Duration::new(0, 0) }
    }

    pub fn advance(&mut self, step: Duration) -> bool {
        if self.left.is_zero() {
            self.left = self.delay;
            true
        } else if let Some(next_left) = self.left.checked_sub(step) {
            self.left = next_left;
            false
        } else {
            self.left = self.delay;
            true
        }
    }
    
    pub fn delay(&self) -> Duration {
        self.delay
    }
    
    pub fn left(&self) -> Duration {
        self.left
    }
}


pub struct System {
    devices: [Option<(Device, Timer)>; 16],
    bus_freq: u32,
    bus_timer: Timer,
    last_dev_locked_bus: Option<u8>,
}


pub struct DeviceDescription {
    bus_addr: u8,
    kernel: Box<dyn Kernel>,
    clock_freq: u32,
    verbose: bool,
}


impl DeviceDescription {
    pub fn new(bus_addr: u8, kernel: Box<dyn Kernel>, clock_freq: u32, verbose: bool) -> Self {
        Self { bus_addr, kernel, clock_freq, verbose }
    }
}


impl System {
    pub fn new(devices: Vec<DeviceDescription>, bus_freq: u32) -> Self {
        let mut devs = [const { None }; 16];
        for dev in devices {
            devs[dev.bus_addr as usize] = Some((
                Device::new(dev.kernel, dev.bus_addr, dev.clock_freq, dev.verbose),
                Timer::new(Duration::from_secs_f32(1.0 / dev.clock_freq as f32))
            ));
        };

        Self {
            devices: devs,
            bus_freq,
            bus_timer: Timer::new(Duration::from_secs_f32(1.0 / bus_freq as f32)),
            last_dev_locked_bus: None,
        }
    }

    pub fn tick_bus(&mut self) {
        let mut new_msg = None;
        for (msg, from, to) in self.devices.iter().enumerate()
            .filter_map(|(i, d)| Some((i, d.as_ref()?)))
            .filter_map(|(i, d)| Some((d.0.bus_msg_send?.0, i as u8, d.0.bus_msg_send?.1)))
            .filter(|(_, _, to)| self.devices[*to as usize].as_ref().is_none_or(|d| d.0.kernel.can_rcv_bus_msg())) {
            new_msg = Some((msg, from, to));
            
            if self.last_dev_locked_bus.is_none_or(|d_i| from > d_i) {
                self.last_dev_locked_bus = Some(from);
                break;
            };
        };
        
        if let Some((msg, from, to)) = new_msg {
            let sender = &mut self.devices[from as usize].as_mut().unwrap().0;
            sender.bus_msg_send = None;
            sender.kernel.end_send_bus_msg();
            
            if let Some(slot) = self.devices.get_mut(to as usize)
                && let Some(rcv) = slot.as_mut().map(|s| &mut s.0) {
                rcv.bus_msg_rcv = Some((msg, from));
            };
        };
    }

    pub fn tick(&mut self, step: Duration) -> Duration {
        let mut next_step = Duration::from_secs(1000);
        for (dev, timer) in self.devices.iter_mut().filter_map(|d| d.as_mut()) {
            if timer.advance(step) {
                dev.tick();
            };
            
            next_step = next_step.min(timer.left());
        };

        if self.bus_timer.advance(step) {
            self.tick_bus();
        };
        
        next_step.min(self.bus_timer.left())
    }

    #[inline]
    pub fn run(&mut self, dur: Option<Duration>) {
        let mut runtime = Duration::new(0, 0); 
        
        let mut tick_delay = Duration::new(0, 0);
        while dur.is_none_or(|d| runtime < d) {
            std::thread::sleep(tick_delay);

            // todo take into account how long did tick take
            tick_delay = self.tick(tick_delay);

            if dur.is_some() {
                runtime += tick_delay;
            };
        };
    }
    
    #[inline]
    pub fn run_and_kill_cpu(&mut self, dur: Option<Duration>) {
        let mut runtime = Duration::new(0, 0);
        
        let mut tick_time = Duration::new(0, 0); 
        let start = Instant::now();
        while dur.is_none_or(|d| runtime < d) {
            let _ = self.tick(tick_time);
            
            let end = Instant::now();
            let total_time = end.checked_duration_since(start).unwrap_or(Duration::new(0, 1));
            tick_time = total_time - runtime;
            
            
            runtime = total_time;
        };
    }
}


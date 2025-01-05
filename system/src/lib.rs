#![feature(let_chains)]
#![feature(ascii_char)]

pub mod kernels;
pub mod device;

use std::time::{Duration, Instant};
use crate::device::Device;
use crate::kernels::Kernel;

struct Clock {
    delay: u32,
    left: u32,
}


impl Clock {
    pub fn new(delay: u32) -> Self {
        Self { delay, left: 1 }
    }

    pub fn advance(&mut self) -> bool {
        if self.left == 1 {
            self.left = self.delay;
            true
        } else {
            self.left -= 1;
            false
        }
    }
}


pub struct System {
    devices: [Option<(Device, Clock)>; 16],
    tick_delay: f32,
    tick_freq: u32,
    bus_freq: u16,
    bus_clock: Clock,
    last_dev_locked_bus: Option<u8>,
}


pub struct DeviceDescription {
    bus_addr: u8,
    kernel: Box<dyn Kernel>,
    clock_freq: u16,
    verbose: bool,
}


impl DeviceDescription {
    pub fn new(bus_addr: u8, kernel: Box<dyn Kernel>, clock_freq: u16, verbose: bool) -> Self {
        Self { bus_addr, kernel, clock_freq, verbose }
    }
}


impl System {
    pub fn new(devices: Vec<DeviceDescription>, bus_freq: u16) -> Self {
        let tick_freq = devices.iter().map(|DeviceDescription { clock_freq, .. }| *clock_freq as u32).fold(bus_freq as u32, num::integer::lcm);

        let mut devs = [const { None }; 16];
        for dev in devices {
            devs[dev.bus_addr as usize] = Some((
                Device::new(dev.kernel, dev.bus_addr, dev.clock_freq, dev.verbose),
                Clock::new(tick_freq / dev.clock_freq as u32)
            ));
        };

        Self {
            devices: devs,
            tick_freq, bus_freq,
            tick_delay: 1.0 / tick_freq as f32,
            bus_clock: Clock::new(tick_freq / bus_freq as u32),
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

    pub fn tick(&mut self) {
        for (dev, clock) in self.devices.iter_mut().filter_map(|d| d.as_mut()) {
            if clock.advance() {
                dev.tick();
            };
        };

        if self.bus_clock.advance() {
            self.tick_bus();
        };
    }

    pub fn run(&mut self, seconds: Option<f32>) {
        let mut ticks = seconds.map(|s| (s / self.tick_delay).ceil() as u32);
        let delay = Duration::from_secs_f32(self.tick_delay); 
        
        // todo better performing parallelization or idk
        while ticks.is_none_or(|t| t != 0) {
            let start = Instant::now();
            
            self.tick();
            
            let end = Instant::now();
            let tick_time = end - start;
            
            if tick_time < delay {
                std::thread::sleep(delay - tick_time);
            };

            if let Some(t) = ticks {
                ticks = Some(t - 1);
            };
        };
    }
}


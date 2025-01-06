use crate::kernels::{Kernel, DeviceKernel};

pub struct Device {
    pub bus_msg_send: Option<(u8, u8)>,
    pub bus_msg_rcv: Option<(u8, u8)>,
    pub kernel: DeviceKernel,
    pub clock_freq: u32,
    pub verbose: bool,
    pub addr: u8,
    pub ticks: u64,
}


impl Device {
    pub fn new(kernel: impl Into<DeviceKernel>, addr: u8, clock_freq: u32, verbose: bool) -> Self {
        let mut kernel = kernel.into();
        
        kernel.init_bus(addr);
        
        Self {
            clock_freq, kernel, verbose, addr,
            bus_msg_send: None,
            bus_msg_rcv: None,
            ticks: 0,
        }
    }

    pub fn tick(&mut self) {
        if let Some(msg) = self.bus_msg_rcv
            && self.kernel.can_rcv_bus_msg() {
            self.bus_msg_rcv = None;
            self.kernel.rcv_bus_msg(msg);
        };
        
        self.kernel.tick();
        self.ticks = self.ticks.wrapping_add(1);
        
        if self.verbose {
            eprintln!("{} (t{}): {}", self.kernel.name(), self.ticks, self.kernel);
        };
        
        if let Some(msg) = self.kernel.send_bus_msg() {
            self.bus_msg_send = Some(msg);
        };
    }
}

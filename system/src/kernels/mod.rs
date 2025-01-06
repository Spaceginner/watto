use std::fmt::{Display, Formatter};
use enum_dispatch::enum_dispatch;

mod cpu;
mod serial;


pub use cpu::Cpu;
pub use serial::Serial;


#[enum_dispatch(DeviceKernel)]
pub trait Kernel: Display {
    fn name(&self) -> &'static str;
    
    fn init_bus(&mut self, addr: u8);
    
    fn tick(&mut self);
    
    fn send_bus_msg(&mut self) -> Option<(u8, u8)>;
    
    fn end_send_bus_msg(&mut self);
    
    fn rcv_bus_msg(&mut self, msg: (u8, u8));
    
    fn can_rcv_bus_msg(&self) -> bool;
}



#[enum_dispatch]
pub enum DeviceKernel {
    Cpu,
    Serial
}


// xxx so sad it isnt possible to derive this as well
impl Display for DeviceKernel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cpu(cpu) => write!(f, "{cpu}"),
            Self::Serial(serial) => write!(f, "{serial}"),
        }
    }
}

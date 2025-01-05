use std::fmt::Display;

mod cpu;
mod serial;


pub use cpu::Cpu;
pub use serial::Serial;


pub trait Kernel: Display {
    fn name(&self) -> &'static str;
    
    fn init_bus(&mut self, addr: u8);
    
    fn tick(&mut self);
    
    fn send_bus_msg(&mut self) -> Option<(u8, u8)>;
    
    fn end_send_bus_msg(&mut self);
    
    fn rcv_bus_msg(&mut self, msg: (u8, u8));
    
    fn can_rcv_bus_msg(&self) -> bool;
}

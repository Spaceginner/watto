use std::ascii::Char;
use std::fmt::{Display, Formatter};
use std::io::Write;
use crate::kernels::Kernel;

#[derive(Debug, Clone, Default)]
pub struct Serial {
    bus_rcv_buf: Option<(u8, u8)>,
    last_printed_c: Option<Char>,
}

impl Serial {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Display for Serial {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "last: {:?}", self.last_printed_c)
    }
}

impl Kernel for Serial {
    fn name(&self) -> &'static str {
        "serial"
    }

    fn init_bus(&mut self, _addr: u8) {
        //
    }
    
    fn send_bus_msg(&mut self) -> Option<(u8, u8)> {
        None
    }

    fn end_send_bus_msg(&mut self) {
        //
    }

    fn rcv_bus_msg(&mut self, msg: (u8, u8)) {
        assert!(self.bus_rcv_buf.is_none());
        self.bus_rcv_buf = Some(msg);
    }

    fn can_rcv_bus_msg(&self) -> bool {
        self.bus_rcv_buf.is_none()
    }

    fn tick(&mut self) {
        self.last_printed_c = None;
        if let Some((msg, _)) = self.bus_rcv_buf.take()
            && let Some(c) = Char::from_u8(msg) {
            print!("{c}");
            std::io::stdout().flush().unwrap();
            self.last_printed_c = Some(c);
        };
    }
}

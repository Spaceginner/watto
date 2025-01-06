use std::fmt::{Display, Formatter};
use watto::{Instruction, reg, Register};
use super::Kernel;


enum HaltState {
    Paused,
    Stopped,
}

pub struct Cpu {
    mem: Vec<u8>,
    regs: [u16; 11],
    bus_addr: u8,
    bus_buf_send: Option<(u8, u8)>,
    bus_buf_send_end: bool,
    bus_buf_rcv: Option<(u8, u8)>,
    bus_rcv_waiting: bool,
    halt: Option<HaltState>,
    last_instr: Option<Instruction>,
}

impl Cpu {
    pub fn new(ram_size: u16, prog: &[u8]) -> Self {
        let mut mem = vec![0x00; ram_size as usize];
        
        // load prog
        prog.iter().copied().enumerate().for_each(|(i, b)| mem[i] = b);
        
        Self {
            mem,
            regs: [0x0000; 11],
            bus_addr: 0x00,
            bus_buf_send: None,
            bus_buf_send_end: true,
            bus_buf_rcv: None,
            bus_rcv_waiting: false,
            halt: None,
            last_instr: None,
        }
    }
    
    fn advance_si(&mut self, cur: Instruction) {
        // todo handle overflow
        self.regs[reg!(si)] = self.regs[reg!(si)].wrapping_add(cur.to_id().size() as u16);
    }
}

impl Display for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (si, oa, ob, oc, ga, gb, gc, gd, da, db) = (
            self.regs[reg!(si)],
            self.regs[reg!(oa)],
            self.regs[reg!(ob)],
            self.regs[reg!(oc)],
            self.regs[reg!(ga)],
            self.regs[reg!(gb)],
            self.regs[reg!(gc)],
            self.regs[reg!(gd)],
            self.regs[reg!(da)],
            self.regs[reg!(db)],
        );
        
        write!(f, "si: 0x{si:0>4x} | oa: 0x{oa:0>4x} | ob: 0x{ob:0>4x} | oc: 0x{oc:0>4x} | ga: 0x{ga:0>4x} | gb: 0x{gb:0>4x} | gc: 0x{gc:0>4x} | gd: 0x{gd:0>4x} | da: {da} (0x{da:0>4x}) | db: {db} (0x{db:0>4x}) | last: {}", self.last_instr.map(|i| i.to_string()).unwrap_or_else(|| String::from("n/a")))
    }
}

impl Kernel for Cpu {
    fn name(&self) -> &'static str {
        "cpu"
    }

    fn init_bus(&mut self, addr: u8) {
        self.bus_addr = addr;
    }

    fn send_bus_msg(&mut self) -> Option<(u8, u8)> {
        let msg = self.bus_buf_send.take();
        if msg.is_some() {
            self.bus_buf_send_end = false;
        };
        msg
    }

    fn end_send_bus_msg(&mut self) {
        self.bus_buf_send_end = true;
    }
    
    fn rcv_bus_msg(&mut self, msg: (u8, u8)) {
        self.bus_buf_rcv = Some(msg);
    }

    fn can_rcv_bus_msg(&self) -> bool {
        self.bus_rcv_waiting || self.bus_buf_rcv.is_none()
    }

    fn tick(&mut self) {
        // todo better handle this
        if self.halt.is_some() {
            return;
        };

        // todo somehow handle errors idk
        let instr = Instruction::decode_from_iter(
            &mut self.mem.iter()
                .skip(self.regs[reg!(si)] as usize)
                .copied()
        );

        match instr {
            Err(_) => { self.halt = Some(HaltState::Stopped); },
            Ok(instr) => {
                match instr {
                    Instruction::Skip => {},
                    Instruction::Pause => { self.halt = Some(HaltState::Paused); },
                    Instruction::Stop => { self.halt = Some(HaltState::Stopped); },
                    Instruction::Wait => {
                        let reg_oa = &mut self.regs[reg!(oa)];
                        if *reg_oa != 0 {
                            *reg_oa -= 1;
                        } else {
                            self.advance_si(instr);
                        };
                    },

                    Instruction::Set(reg, val) => {
                        self.regs[reg.to_index()] = val;
                        
                        if reg != Register::ServiceInstruction {
                            self.advance_si(instr);
                        };
                    },
                    Instruction::SetIfNotZero(reg, val) => {
                        if self.regs[reg!(oc)] != 0 {
                            self.regs[reg.to_index()] = val;
                            
                            if reg != Register::ServiceInstruction {
                                self.advance_si(instr);
                            };
                        } else {
                            self.advance_si(instr);
                        };
                    },
                    Instruction::SetIfZero(reg, val) => {
                        if self.regs[reg!(oc)] == 0 {
                            self.regs[reg.to_index()] = val;
                            
                            if reg != Register::ServiceInstruction {
                                self.advance_si(instr);
                            };
                        } else {
                            self.advance_si(instr);
                        };
                    },
                    Instruction::Copy(a, b) => {
                        self.regs[b.to_index()] = self.regs[a.to_index()];
                        
                        if b != Register::ServiceInstruction {
                            self.advance_si(instr);
                        };
                    },
                    Instruction::Swap(a, b) => {
                        (self.regs[a.to_index()], self.regs[b.to_index()])
                            = (self.regs[b.to_index()], self.regs[a.to_index()]);
                        
                        if !((a == Register::ServiceInstruction) ^ (b == Register::ServiceInstruction)) {
                            self.advance_si(instr);
                        };
                    },

                    Instruction::WriteByte => {
                        let addr = self.regs[reg!(oc)] as usize;
                        let val = self.regs[reg!(oa)] as u8;

                        // todo handle out-of-bounds addr
                        self.mem[addr] = val;

                        self.advance_si(instr);
                    }
                    Instruction::WriteWord => {
                        let addr = self.regs[reg!(oc)] as usize;
                        let val = self.regs[reg!(oa)];

                        // todo handle out-of-bounds addr
                        self.mem[addr..=(addr+1)].copy_from_slice(&val.to_le_bytes());

                        self.advance_si(instr);
                    }
                    Instruction::ReadByte => {
                        let addr = self.regs[reg!(oc)] as usize;
                        self.regs[reg!(oa)] &= 0xff00;
                        self.regs[reg!(oa)] |= self.mem[addr] as u16;
                        self.advance_si(instr);
                    }
                    Instruction::ReadWord => {
                        let addr = self.regs[reg!(oc)] as usize;
                        self.regs[reg!(oa)] = u16::from_le_bytes([self.mem[addr], self.mem[addr+1]]);
                        self.advance_si(instr);
                    }

                    Instruction::Add => {
                        let sum = self.regs[reg!(oa)].overflowing_add(self.regs[reg!(ob)]);
                        self.regs[reg!(oc)] = sum.0;
                        if sum.1 {
                            self.regs[reg!(ss)] &= 0b1111111111111110;
                        } else {
                            self.regs[reg!(ss)] |= 0b0000000000000001;
                        };
                        self.advance_si(instr);
                    },
                    Instruction::CompareUnsigned => {
                        let reg_oa = self.regs[reg!(oa)];
                        let reg_ob = self.regs[reg!(ob)];

                        let are_eq = (reg_oa == reg_ob) as u16;
                        let is_lg = (reg_oa > reg_ob) as u16;

                        self.regs[reg!(oc)] = are_eq | (is_lg << 1);
                        self.advance_si(instr);
                    },
                    Instruction::CompareSigned => {
                        let reg_oa = self.regs[reg!(oa)] as i16;
                        let reg_ob = self.regs[reg!(ob)] as i16;

                        let are_eq = (reg_oa == reg_ob) as u16;
                        let is_lg = (reg_oa > reg_ob) as u16;

                        self.regs[reg!(oc)] = are_eq | (is_lg << 1);
                        self.advance_si(instr);
                    },
                    Instruction::And => {
                        self.regs[reg!(oc)] = self.regs[reg!(oa)] & self.regs[reg!(ob)];
                        self.advance_si(instr);
                    },
                    Instruction::Or => {
                        self.regs[reg!(oc)] = self.regs[reg!(oa)] | self.regs[reg!(ob)];
                        self.advance_si(instr);
                    },
                    Instruction::Xor => {
                        self.regs[reg!(oc)] = self.regs[reg!(oa)] ^ self.regs[reg!(ob)];
                        self.advance_si(instr);
                    },
                    Instruction::Rotate => {
                        self.regs[reg!(oa)] = self.regs[reg!(oa)].rotate_left(1);
                        self.advance_si(instr);
                    }

                    Instruction::IoWrite => {
                        let addr = self.regs[reg!(oc)] as u8;

                        if addr != 0 {
                            let msg = self.regs[reg!(oa)].to_le_bytes()[0];

                            self.bus_buf_send_end = false;
                            self.bus_buf_send = Some((msg, addr));
                        };

                        self.advance_si(instr);
                    },
                    Instruction::IoRead => {
                        if let Some((msg, addr)) = self.bus_buf_rcv
                            && (self.regs[reg!(oc)] as u8 == 0 || addr == self.regs[reg!(oc)] as u8) {
                            self.regs[reg!(oa)] &= 0xFF00;
                            self.regs[reg!(oa)] |= msg as u16;
                        } else {
                            self.regs[reg!(oc)] = 0x0000;
                        };

                        self.advance_si(instr);
                    },
                    Instruction::IoWaitForWrite => {
                        if self.bus_buf_send_end || self.regs[reg!(oa)] == 0 {
                            self.advance_si(instr);
                        } else if self.regs[reg!(oa)] != 0xFFFF {
                            self.regs[reg!(oa)] -= 1;
                        };
                    },
                    Instruction::IoWaitForRead => {
                        self.bus_rcv_waiting = true;
                        if self.bus_buf_rcv.is_some_and(|(_, addr)| addr == self.regs[reg!(oc)] as u8)
                            || self.regs[reg!(oa)] == 0 {
                            self.bus_rcv_waiting = false;
                            self.advance_si(instr);
                        } else if self.regs[reg!(oa)] != 0xFFFF {
                            self.regs[reg!(oa)] -= 1;
                        };
                    },
                    Instruction::IoBufClearWrite => {
                        self.bus_buf_send = None;
                        self.advance_si(instr);
                    }
                    Instruction::IoBufClearRead => {
                        self.bus_buf_rcv = None;
                        self.advance_si(instr);
                    },
                    Instruction::IoBufReadWrite => {
                        if let Some((msg, addr)) = self.bus_buf_send {
                            self.regs[reg!(oc)] = addr as u16;

                            self.regs[reg!(oa)] &= 0xFF00;
                            self.regs[reg!(oa)] |= msg as u16;
                        } else {
                            self.regs[reg!(oc)] = 0x0000;
                        };
                    }
                };
                self.last_instr = Some(instr);
            },
        };
    }
}

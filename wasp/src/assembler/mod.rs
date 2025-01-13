use std::collections::HashMap;
use std::error::Error;
use err::{AssemblingError, InvalidInstructInfo};
use crate::processor::{Argument, Instruct, Op, ValueArgument};

mod err;

pub struct Assembler<P, PE>
    where P: Iterator<Item = Result<Instruct, PE>>,
        PE: Error + Clone + 'static
{
    processor: P
}


impl<P, PE> Assembler<P, PE>
    where P: Iterator<Item = Result<Instruct, PE>>,
          PE: Error + Clone + 'static
{
    pub fn new(processor: P) -> Self {
        Self { processor }
    }
    
    // xxx should it assemble into some Program(Vec<Instruction>)?
    pub fn assemble(mut self) -> Result<Vec<u8>, AssemblingError<PE>> {
        let instructs = self.processor.try_collect::<Vec<_>>().map_err(AssemblingError::ProcessingError)?;
        
        let (mut variables, addrs) = {
            let mut variables = HashMap::new();
            let mut addrs = Vec::new();
            let mut cur_addr = 0u16;
            for instruct in instructs.iter() {
                for label in instruct.labels() {
                    variables.insert(label.clone(), Variable { value: cur_addr, is_label: true });
                };

                let instr_size = instruct.operation().size() as u16;

                addrs.push((cur_addr, instr_size));

                if let Some(new_addr) = cur_addr.checked_add(instr_size) {
                    cur_addr = new_addr
                } else {
                    return Err(AssemblingError::ProgTooLarge);
                };
            };
            (variables, addrs)
        };
        
        let mut prog = Vec::new();
        for (i, instruct) in instructs.into_iter().enumerate() {
            let pos = instruct.pos();
            let labels = instruct.labels().to_vec();
            match instruct.into_operation() {
                Op::InsertCpuInstruction(id, args) => {
                    prog.push(id.code());

                    for arg in args.iter() {
                        match arg {
                            Argument::Register(reg) => { prog.push(reg.to_addr()); }
                            Argument::Value(val) =>
                                match val {
                                    ValueArgument::Literal(n) => { prog.extend(n.to_le_bytes()) },
                                    ValueArgument::Reference(delta) => {
                                        #[allow(clippy::collapsible_else_if)]
                                        if *delta > 0 {
                                            if let Some(addr) = addrs.get((i as isize + *delta as isize - 1) as usize) {
                                                prog.extend((addr.0 + addr.1).to_le_bytes());
                                            } else {
                                                return Err(AssemblingError::InvalidInstruct { instruct: Instruct::new(pos, labels, Op::InsertCpuInstruction(id, args)), info: InvalidInstructInfo::ReferenceOutOfBounds });
                                            };
                                        } else {
                                            if i >= delta.unsigned_abs() as usize
                                                && let Some(addr) = addrs.get((i as isize + *delta as isize) as usize) {
                                                prog.extend(addr.0.to_le_bytes());
                                            } else {
                                                return Err(AssemblingError::InvalidInstruct { instruct: Instruct::new(pos, labels, Op::InsertCpuInstruction(id, args)), info: InvalidInstructInfo::ReferenceOutOfBounds });
                                            };
                                        };
                                    },
                                    ValueArgument::Variable(name) => {
                                        if let Some(val) = variables.get(name) {
                                            prog.extend(val.value.to_le_bytes());
                                        } else {
                                            return Err(AssemblingError::InvalidInstruct { instruct: Instruct::new(pos, labels, Op::InsertCpuInstruction(id, args)), info: InvalidInstructInfo::UnknownVariable });
                                        };
                                    },
                                },
                        };
                    };
                },
                Op::SetVariable(name, value) => {
                    if let Some(Variable { is_label: true, .. }) = variables.get(&name) {
                        return Err(AssemblingError::InvalidInstruct { instruct: Instruct::new(pos, labels, Op::SetVariable(name, value)), info: InvalidInstructInfo::ModifyingLabel });
                    };

                    variables.entry(name)
                        .and_modify(|v| v.value = value)
                        .or_insert(Variable { value, is_label: false });
                },
                Op::InsertByte(b) => { prog.push(b); },
                Op::InsertWord(w) => { prog.extend(w.to_le_bytes()); },
                Op::InsertBytes(mut bytes) => { prog.append(&mut bytes); },
                Op::InsertMultipleBytes(b, count) => { (0..count).for_each(|_| prog.push(b)) ;},
                Op::InsertCString(cstr) => { prog.extend(cstr.into_bytes_with_nul()); },
                Op::Void => {}
            }
        };
        Ok(prog)
    }
}


struct Variable {
    value: u16,
    is_label: bool,
}

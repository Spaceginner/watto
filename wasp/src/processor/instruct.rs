use std::ffi::CString;
use std::fmt::{Display, Formatter};
use watto::{InstructionId, Register};
use crate::lexer::Pos;

#[derive(Debug, Clone)]
pub struct Instruct {
    pub(super) pos: Pos,
    pub(super) labels: Vec<String>,
    pub(super) operation: Op,
}

impl Display for Instruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pos, self.operation)?;
        
        if !self.labels.is_empty() {
            write!(f, " (: {}) ", self.labels.join(" "))?;
        };
        
        Ok(())
    }
}

impl Instruct {
    pub fn new(pos: Pos, labels: Vec<String>, operation: Op) -> Self {
        Self { pos, labels, operation }
    }
    
    pub fn pos(&self) -> Pos {
        self.pos
    }
    
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
    
    pub fn operation(&self) -> &Op {
        &self.operation
    }
    
    pub fn into_operation(self) -> Op {
        self.operation
    }
}


#[derive(Debug, Clone)]
pub enum Op {
    InsertCpuInstruction(InstructionId, Vec<Argument>),

    SetVariable(String, u16),

    InsertByte(u8),
    InsertWord(u16),
    InsertBytes(Vec<u8>),  // todo dont load entire file into memory, instead provide a file handle
    InsertMultipleBytes(u8, u16),
    InsertCString(CString),

    Void,
}


impl Op {
    pub fn size(&self) -> usize {
        match self {
            Self::InsertCpuInstruction(id, ..) => id.size(),
            
            Self::SetVariable(..) => 0,
            
            Self::InsertByte(..) => 1,
            Self::InsertWord(..) => 2,
            Self::InsertBytes(bytes) => bytes.len(),
            Self::InsertMultipleBytes(_, count) => *count as usize,
            Self::InsertCString(cstr) => cstr.as_bytes_with_nul().len(),
            
            Self::Void => 0,
        }
    }
}


impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InsertCpuInstruction(id, args) =>
                if args.is_empty() {
                    write!(f, "{id}")
                } else {
                    write!(f, "{id} {}", args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>().join(" "))
                },

            Op::SetVariable(name, val) => write!(f, "!set %{name} #d{val}"),
            
            Op::InsertByte(b) => write!(f, "!byte #d{b}"),
            Op::InsertWord(w) => write!(f, "!word #d{w}"),
            Op::InsertBytes(_) => write!(f, "!file \"...\""),
            Op::InsertMultipleBytes(b, count) => write!(f, "!bytes #d{b} #d{count}"),
            Op::InsertCString(s) => write!(f, "!cstr \"{}\"", s.to_string_lossy()),
            
            Op::Void => write!(f, "!void")
        }
    }
}


#[derive(Debug, Clone)]
pub enum Argument {
    Register(Register),
    Value(ValueArgument)
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(reg) => write!(f, "{reg}"),
            Self::Value(val) => write!(f, "{val}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueArgument {
    Literal(u16),
    Reference(i16),
    Variable(String),
}


impl Display for ValueArgument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(n) => write!(f, "#d{n}"),
            Self::Reference(delta) => write!(f, "~{delta}"),
            Self::Variable(name) => write!(f, "%{name}"),
        }
    }
}

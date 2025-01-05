use std::fmt::{Display, Formatter};
use watto::Register;
use crate::lexer::Pos;

#[derive(Debug, Clone)]
pub struct Element {
    pub(crate) pos: Pos,
    pub(crate) value: ElementValue,
}


impl Element {
    pub fn new(pos: Pos, value: ElementValue) -> Self {
        Self { pos, value }
    }
    
    pub fn pos(&self) -> Pos {
        self.pos
    }
    
    pub fn value(&self) -> &ElementValue {
        &self.value
    }
    
    pub fn into_value(self) -> ElementValue {
        self.value
    }
}


#[derive(Debug, Clone)]
pub enum ElementValue {
    CpuInstruction(String),
    ProcessorInstruction(String),
    Label(String),
    Variable(String),
    Literal(LiteralValue),
    Register(Register),
    Reference(i16),
    Substitute(usize),
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    Number(u16),
    String(String),
    Char(char)
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pos, self.value)
    }
}

impl Display for ElementValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CpuInstruction(instr) => write!(f, "{instr}"),
            Self::ProcessorInstruction(instr) => write!(f, "!{instr}"),
            Self::Label(name) => write!(f, ":{name}"),
            Self::Variable(name) => write!(f, "%{name}"),
            Self::Reference(delta) => write!(f, "~{delta}"),
            Self::Literal(lit) => write!(f, "{lit}"),
            Self::Register(reg) => write!(f, "${reg}"),
            Self::Substitute(i) => write!(f, "@{i}"),
        }
    }
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(val) => write!(f, "#d{val}"),
            Self::String(s) => write!(f, "\"{s}\""),  // todo escape s
            Self::Char(c) => write!(f, "'{c}"),  // todo escape c..?
        }
    }
}

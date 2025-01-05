use std::fmt::{Display, Formatter};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    ServiceInstruction,
    ServiceStatus,
    
    OperandA,
    OperandB,
    OperandC,

    GeneralA,
    GeneralB,
    GeneralC,
    GeneralD,

    DisplayA,
    DisplayB,
}


impl Register {
    pub fn to_addr(self) -> u8 {
        match self {
            Self::ServiceInstruction => 0b_0000_0000,
            Self::ServiceStatus => 0b_0000_0001,

            Self::OperandA => 0b_0001_0000,
            Self::OperandB => 0b_0001_0001,
            Self::OperandC => 0b_0001_0010,

            Self::GeneralA => 0b_0010_0000,
            Self::GeneralB => 0b_0010_0001,
            Self::GeneralC => 0b_0010_0010,
            Self::GeneralD => 0b_0010_0011,

            Self::DisplayA => 0b_0011_0000,
            Self::DisplayB => 0b_0011_0001,
        }
    }

    pub fn from_addr(addr: u8) -> Option<Self> {
        match addr {
            0b_0000_0000 => Some(Self::ServiceInstruction),
            0b_0000_0001 => Some(Self::ServiceStatus),

            0b_0001_0000 => Some(Self::OperandA),
            0b_0001_0001 => Some(Self::OperandB),
            0b_0001_0010 => Some(Self::OperandC),

            0b_0010_0000 => Some(Self::GeneralA),
            0b_0010_0001 => Some(Self::GeneralB),
            0b_0010_0010 => Some(Self::GeneralC),
            0b_0010_0011 => Some(Self::GeneralD),

            0b_0011_0000 => Some(Self::DisplayA),
            0b_0011_0001 => Some(Self::DisplayB),

            _ => None,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::ServiceInstruction => 0,
            Self::ServiceStatus => 1,

            Self::OperandA => 2,
            Self::OperandB => 3,
            Self::OperandC => 4,

            Self::GeneralA => 5,
            Self::GeneralB => 6,
            Self::GeneralC => 7,
            Self::GeneralD => 8,

            Self::DisplayA => 9,
            Self::DisplayB => 10,
        }
    }
}


impl TryFrom<&str> for Register {
    type Error = ();
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "si" => Ok(Self::ServiceInstruction),
            "ss" => Ok(Self::ServiceStatus),

            "oa" => Ok(Self::OperandA),
            "ob" => Ok(Self::OperandB),
            "oc" => Ok(Self::OperandC),

            "ga" => Ok(Self::GeneralA),
            "gb" => Ok(Self::GeneralB),
            "gc" => Ok(Self::GeneralC),
            "gd" => Ok(Self::GeneralD),

            "da" => Ok(Self::DisplayA),
            "db" => Ok(Self::DisplayB),

            _ => Err(()),
        }
    }
}


impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceInstruction => write!(f, "$si"),
            Self::ServiceStatus => write!(f, "$ss"),

            Self::OperandA => write!(f, "$oa"),
            Self::OperandB => write!(f, "$ob"),
            Self::OperandC => write!(f, "$oc"),

            Self::GeneralA => write!(f, "$ga"),
            Self::GeneralB => write!(f, "$gb"),
            Self::GeneralC => write!(f, "$gc"),
            Self::GeneralD => write!(f, "$gd"),

            Self::DisplayA => write!(f, "$da"),
            Self::DisplayB => write!(f, "$db"),
        }
    }
}


#[macro_export] macro_rules! reg {
    (si) => { Register::ServiceInstruction.to_index() };
    (ss) => { Register::ServiceStatus.to_index() };
    
    (oa) => { Register::OperandA.to_index() };
    (ob) => { Register::OperandB.to_index() };
    (oc) => { Register::OperandC.to_index() };
    
    (ga) => { Register::GeneralA.to_index() };
    (gb) => { Register::GeneralB.to_index() };
    (gc) => { Register::GeneralC.to_index() };
    (gd) => { Register::GeneralD.to_index() };
    
    (da) => { Register::DisplayA.to_index() };
    (db) => { Register::DisplayB.to_index() };
}

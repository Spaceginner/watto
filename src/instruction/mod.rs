mod id;

use std::fmt::{Display, Formatter};
use crate::Register;

pub use id::{InstructionId, Argument};


#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Skip,
    Pause,
    Stop,
    Wait,
    
    Set(Register, u16),
    SetIfNotZero(Register, u16),
    SetIfZero(Register, u16),
    Copy(Register, Register),
    Swap(Register, Register),

    WriteByte,
    WriteWord,
    ReadByte,
    ReadWord,
    
    Add,
    CompareUnsigned,
    CompareSigned,
    And,
    Or,
    Xor,
    Rotate,
    
    IoWrite,
    IoRead,
    IoWaitForWrite,
    IoWaitForRead,
    IoBufClearWrite,
    IoBufClearRead,
    IoBufReadWrite
}


impl Instruction {
    pub fn to_id(self) -> InstructionId {
        match self {
            Self::Skip => InstructionId::Skip,
            Self::Pause => InstructionId::Pause,
            Self::Stop => InstructionId::Stop,
            Self::Wait => InstructionId::Wait,

            Self::Set(..) => InstructionId::Set,
            Self::SetIfNotZero(..) => InstructionId::SetIfNotZero,
            Self::SetIfZero(..) => InstructionId::SetIfZero,
            Self::Copy(..) => InstructionId::Copy,
            Self::Swap(..) => InstructionId::Swap,

            Self::WriteByte => InstructionId::WriteByte,
            Self::WriteWord => InstructionId::WriteWord,
            Self::ReadByte => InstructionId::ReadByte,
            Self::ReadWord => InstructionId::ReadWord,

            Self::Add => InstructionId::Add,
            Self::CompareUnsigned => InstructionId::CompareUnsigned,
            Self::CompareSigned => InstructionId::CompareSigned,
            Self::And => InstructionId::And,
            Self::Or => InstructionId::Or,
            Self::Xor => InstructionId::Xor,
            Self::Rotate => InstructionId::Rotate,

            Self::IoWrite => InstructionId::IoWrite,
            Self::IoRead => InstructionId::IoRead,
            Self::IoWaitForWrite => InstructionId::IoWaitForWrite,
            Self::IoWaitForRead => InstructionId::IoWaitForRead,
            Self::IoBufClearWrite => InstructionId::IoBufClearWrite,
            Self::IoBufClearRead => InstructionId::IoBufClearRead,
            Self::IoBufReadWrite => InstructionId::IoBufReadWrite
        }
    }
    
    pub fn encode(self) -> Vec<u8> {
        match self {
            instr @ (
                Self::Set(reg, val)
                | Self::SetIfNotZero(reg, val)
                | Self::SetIfZero(reg, val)
            ) => {
                let [val_0, val_1] = val.to_le_bytes();
                vec![instr.to_id().code(), reg.to_addr(), val_0, val_1]
            },
            instr @ (
                Self::Copy(a, b)
                | Self::Swap(a, b)
            ) => vec![instr.to_id().code(), a.to_addr(), b.to_addr()],
            instr => vec![instr.to_id().code()]
        }
    }
    
    pub fn decode_from_iter(bytes: &mut impl Iterator<Item = u8>) -> Result<Self, InstructionDecodingError> {
        let mut next_byte = || bytes.next().ok_or(InstructionDecodingError::EarlyEOB);
        
        match InstructionId::try_from(next_byte()?) {
            Ok(instr) =>
                match instr {
                    id @ (InstructionId::Set | InstructionId::SetIfNotZero | InstructionId::SetIfZero) => {
                        let reg = Register::from_addr(next_byte()?).ok_or(InstructionDecodingError::InvalidRegister)?;
                        let value = u16::from_le_bytes([next_byte()?, next_byte()?]);
                        Ok(match id {
                            InstructionId::Set => Self::Set,
                            InstructionId::SetIfNotZero => Self::SetIfNotZero,
                            InstructionId::SetIfZero => Self::SetIfZero,
                            _ => unreachable!(),
                        }(reg, value))
                    },
                    id @ (InstructionId::Copy | InstructionId::Swap) => {
                        let a = Register::from_addr(next_byte()?).ok_or(InstructionDecodingError::InvalidRegister)?;
                        let b = Register::from_addr(next_byte()?).ok_or(InstructionDecodingError::InvalidRegister)?;
                        Ok(match id {
                            InstructionId::Copy => Self::Copy,
                            InstructionId::Swap => Self::Swap,
                            _ => unreachable!()
                        }(a, b))
                    },
                    id => Ok(Self::try_from(id).unwrap())
                },
            Err(()) => Err(InstructionDecodingError::InvalidId),
        }
    }
}


impl TryFrom<InstructionId> for Instruction {
    type Error = ();

    fn try_from(value: InstructionId) -> Result<Self, Self::Error> {
        match value {
            InstructionId::Skip => Ok(Self::Skip),
            InstructionId::Pause => Ok(Self::Pause),
            InstructionId::Stop => Ok(Self::Stop),
            InstructionId::Wait => Ok(Self::Wait),
            
            InstructionId::Set => Err(()),
            InstructionId::SetIfNotZero => Err(()),
            InstructionId::SetIfZero => Err(()),
            InstructionId::Copy => Err(()),
            InstructionId::Swap => Err(()),
            
            InstructionId::WriteByte => Ok(Self::WriteByte),
            InstructionId::WriteWord => Ok(Self::WriteWord),
            InstructionId::ReadByte => Ok(Self::ReadByte),
            InstructionId::ReadWord => Ok(Self::ReadWord),
            
            InstructionId::Add => Ok(Self::Add),
            InstructionId::CompareUnsigned => Ok(Self::CompareUnsigned),
            InstructionId::CompareSigned => Ok(Self::CompareSigned),
            InstructionId::And => Ok(Self::And),
            InstructionId::Or => Ok(Self::Or),
            InstructionId::Xor => Ok(Self::Xor),
            InstructionId::Rotate => Ok(Self::Rotate),
            
            InstructionId::IoWrite => Ok(Self::IoWrite),
            InstructionId::IoRead => Ok(Self::IoRead),
            InstructionId::IoWaitForWrite => Ok(Self::IoWaitForWrite),
            InstructionId::IoWaitForRead => Ok(Self::IoWaitForRead),
            InstructionId::IoBufClearWrite => Ok(Self::IoBufClearWrite),
            InstructionId::IoBufClearRead => Ok(Self::IoBufClearRead),
            InstructionId::IoBufReadWrite => Ok(Self::IoBufReadWrite),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum InstructionDecodingError {
    EarlyEOB, InvalidId, InvalidRegister 
}

impl Display for InstructionDecodingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EarlyEOB => write!(f, "too early end of byte stream"),
            Self::InvalidId => write!(f, "specified a non-existent instruction"),
            Self::InvalidRegister => write!(f, "specified a non-existent register"),
        }
    }
}

impl std::error::Error for InstructionDecodingError {}


impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Set(reg, val) | Self::SetIfNotZero(reg, val) | Self::SetIfZero(reg, val) => write!(f, "{} {} #d{}", self.to_id(), reg, val),
            Self::Copy(a, b) | Self::Swap(a, b) => write!(f, "{} {} {}", self.to_id(), a, b),
            _ => write!(f, "{}", self.to_id()),
        }
    }
}

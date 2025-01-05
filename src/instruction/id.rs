use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum InstructionId {
    Skip,
    Pause,
    Stop,
    Wait,

    Set,
    SetIfNotZero,
    SetIfZero,
    Copy,
    Swap,

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


impl InstructionId {
    pub fn name(self) -> &'static str {
        match self {
            InstructionId::Skip => "skip",
            InstructionId::Pause => "pause",
            InstructionId::Stop => "stop",
            InstructionId::Wait => "wait",
            
            InstructionId::Set => "set",
            InstructionId::SetIfNotZero => "setnz",
            InstructionId::SetIfZero => "setz",
            InstructionId::Copy => "copy",
            InstructionId::Swap => "swap",
            
            InstructionId::WriteByte => "writeb",
            InstructionId::WriteWord => "writew",
            InstructionId::ReadByte => "readb",
            InstructionId::ReadWord => "readw",
            
            InstructionId::Add => "add",
            InstructionId::CompareUnsigned => "cmp",
            InstructionId::CompareSigned => "cmps",
            InstructionId::And => "and",
            InstructionId::Or => "or",
            InstructionId::Xor => "xor",
            InstructionId::Rotate => "rot",
            
            InstructionId::IoWrite => "iow",
            InstructionId::IoRead => "ior",
            InstructionId::IoWaitForWrite => "ioww",
            InstructionId::IoWaitForRead => "iowr",
            InstructionId::IoBufClearWrite => "iocw",
            InstructionId::IoBufClearRead => "iocr",
            InstructionId::IoBufReadWrite => "iorw",
        }
    }
    
    pub fn code(self) -> u8 {
        match self {
            Self::Skip  => 0b_0000_0000,
            Self::Wait  => 0b_0000_0001,
            Self::Pause => 0b_0000_0010,
            Self::Stop  => 0b_0000_0011,

            Self::Set  => 0b_0001_0000,
            Self::SetIfNotZero => 0b_0001_0010,
            Self::SetIfZero    => 0b_0001_0011,
            Self::Copy => 0b_0001_0100,
            Self::Swap => 0b_0001_0101,

            Self::WriteByte => 0b_0010_0000,
            Self::WriteWord => 0b_0010_0001,
            Self::ReadByte => 0b_0010_0010,
            Self::ReadWord => 0b_0010_0011,

            Self::Add => 0b_0011_0000,
            Self::CompareUnsigned => 0b_0011_0010,
            Self::CompareSigned   => 0b_0011_0011,
            Self::And    => 0b_0011_0100,
            Self::Or     => 0b_0011_0101,
            Self::Xor    => 0b_0011_0110,
            Self::Rotate => 0b_0011_0111,

            Self::IoWrite => 0b_0100_0000,
            Self::IoRead  => 0b_0100_0001,
            Self::IoWaitForWrite  => 0b_0100_0010,
            Self::IoWaitForRead   => 0b_0100_0011,
            Self::IoBufClearWrite => 0b_0100_0100,
            Self::IoBufClearRead  => 0b_0100_0101,
            Self::IoBufReadWrite  => 0b_0100_0111,
        }
    }

    pub fn size(self) -> usize {
        match self {
            Self::Set | Self::SetIfNotZero | Self::SetIfZero => 4,
            Self::Copy | Self::Swap => 3,
            _ => 1,
        }
    }
    
    pub fn arguments(self) -> Vec<Argument> {
        match self {
            Self::Set | Self::SetIfNotZero | Self::SetIfZero => vec![Argument::Register, Argument::Number],
            Self::Swap | Self::Copy => vec![Argument::Register, Argument::Register],
            _ => vec![]
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub enum Argument {
    Register,
    Number,
}


impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register => write!(f, "register"),
            Self::Number => write!(f, "number"),
        }
    }
}


impl TryFrom<u8> for InstructionId {
    type Error = ();
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b_0000_0000 => Ok(Self::Skip),
            0b_0000_0001 => Ok(Self::Wait),
            0b_0000_0010 => Ok(Self::Pause),
            0b_0000_0011 => Ok(Self::Stop),

            0b_0001_0000 => Ok(Self::Set),
            0b_0001_0010 => Ok(Self::SetIfNotZero),
            0b_0001_0011 => Ok(Self::SetIfZero),
            0b_0001_0100 => Ok(Self::Copy),
            0b_0001_0101 => Ok(Self::Swap),

            0b_0010_0000 => Ok(Self::WriteByte),
            0b_0010_0001 => Ok(Self::WriteWord),
            0b_0010_0010 => Ok(Self::ReadByte),
            0b_0010_0011 => Ok(Self::ReadWord),

            0b_0011_0000 => Ok(Self::Add),
            0b_0011_0010 => Ok(Self::CompareUnsigned),
            0b_0011_0011 => Ok(Self::CompareSigned),
            0b_0011_0100 => Ok(Self::And),
            0b_0011_0101 => Ok(Self::Or),
            0b_0011_0110 => Ok(Self::Xor),
            0b_0011_0111 => Ok(Self::Rotate),

            0b_0100_0000 => Ok(Self::IoWrite),
            0b_0100_0001 => Ok(Self::IoRead),
            0b_0100_0010 => Ok(Self::IoWaitForWrite),
            0b_0100_0011 => Ok(Self::IoWaitForRead),
            0b_0100_0100 => Ok(Self::IoBufClearWrite),
            0b_0100_0101 => Ok(Self::IoBufClearRead),
            0b_0100_0111 => Ok(Self::IoBufReadWrite),
            
            _ => Err(())
        }
    }
}


impl TryFrom<&str> for InstructionId {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "skip" => Ok(InstructionId::Skip),
            "pause" => Ok(InstructionId::Pause),
            "stop" => Ok(InstructionId::Stop),
            "wait" => Ok(InstructionId::Wait),

            "set" => Ok(InstructionId::Set),
            "setnz" => Ok(InstructionId::SetIfNotZero),
            "setz" => Ok(InstructionId::SetIfZero),
            "copy" => Ok(InstructionId::Copy),
            "swap" => Ok(InstructionId::Swap),

            "writeb" => Ok(InstructionId::WriteByte),
            "writew" => Ok(InstructionId::WriteWord),
            "readb" => Ok(InstructionId::ReadByte),
            "readw" => Ok(InstructionId::ReadWord),

            "add" => Ok(InstructionId::Add),
            "cmp" => Ok(InstructionId::CompareUnsigned),
            "cmps" => Ok(InstructionId::CompareSigned),
            "and" => Ok(InstructionId::And),
            "or" => Ok(InstructionId::Or),
            "xor" => Ok(InstructionId::Xor),
            "rot" => Ok(InstructionId::Rotate),

            "iow" => Ok(InstructionId::IoWrite),
            "ior" => Ok(InstructionId::IoRead),
            "ioww" => Ok(InstructionId::IoWaitForWrite),
            "iowr" => Ok(InstructionId::IoWaitForRead),
            "iocw" => Ok(InstructionId::IoBufClearWrite),
            "iocr" => Ok(InstructionId::IoBufClearRead),
            "iorw" => Ok(InstructionId::IoBufReadWrite),
            
            _ => Err(()),
        }
    }
}

impl Display for InstructionId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

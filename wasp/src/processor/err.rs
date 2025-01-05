use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::lexer::LexingError;
use crate::parser::{Element, ParsingError};

#[derive(Debug, Clone)]
pub enum ProcessingError<PE>
    where PE: Error + Clone + 'static,
{
    EarlyEoE,  // todo more info
    ParsingFailure(PE),
    InvalidElement {
        elem: Element,
        info: InvalidElementInfo,
    },
}

impl<PE> Error for ProcessingError<PE>
    where PE: Error + Clone + 'static
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParsingFailure(err) => Some(err),
            Self::InvalidElement { info: InvalidElementInfo::MacroParsingFailure(err), .. } => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum InvalidElementInfo
{
    Unexpected,
    CpuInstructionName,
    ProcessorInstructName,
    CpuInstructionArg {
        expected: watto::Argument
    },
    ProcessorInstructArg,  // todo more info
    NonAsciiCharAsArg,
    MacroParsingFailure(ParsingError<LexingError>),
    MacroName,
}

impl<PE> Display for ProcessingError<PE>
    where PE: Error + Clone + 'static
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EarlyEoE => write!(f, "early end of elements"),
            Self::ParsingFailure(_) => write!(f, "parsing error occurred"),
            Self::InvalidElement { elem, info } => write!(f, "invalid element ({elem}): {info}"),
        }
    }
}


impl Display for InvalidElementInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected => write!(f, "unexpected element encountered"),
            Self::CpuInstructionName => write!(f, "unknown cpu instruction name"),
            Self::ProcessorInstructName => write!(f, "unknown processor instruct name"),
            Self::CpuInstructionArg { expected } => write!(f, "expected {expected} as an arg"),
            Self::ProcessorInstructArg => write!(f, "expected other process instruct arg"),
            Self::NonAsciiCharAsArg => write!(f, "can't encode non-ascii char as byte"),
            Self::MacroParsingFailure(_) => write!(f, "macro parsing error"),
            Self::MacroName => write!(f, "unknown macro name")
        }
    }
}

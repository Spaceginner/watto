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
            Self::InvalidElement { info: InvalidElementInfo::IncludedCodeParsingFailure(err), .. } => Some(err),
            Self::InvalidElement { info: InvalidElementInfo::IncludedFileProcessingFailure(err), .. } => Some(err),
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
    IncludedCodeParsingFailure(ParsingError<LexingError>),
    IncludedFileProcessingFailure(Box<ProcessingError<ParsingError<LexingError>>>),
    MacroName,
    AbsolutePathsForbidden,
    NoRelPathGiven,
    NoLibPathGiven,
    PathBreaksOut,
    FailedToReadFile {
        reason: String,  // cant pass std::io::Error because it is not-cloneable
    }
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
            Self::IncludedCodeParsingFailure(_) => write!(f, "an error while parsing included code"),
            Self::MacroName => write!(f, "unknown macro name"),
            Self::IncludedFileProcessingFailure(_) => write!(f, "an error while processing included file"),
            Self::FailedToReadFile { reason } => write!(f, "failed to read file: {reason}"),
            Self::AbsolutePathsForbidden => write!(f, "absolute paths are disabled"),
            Self::NoRelPathGiven => write!(f, "no base path for relative paths has been given"),
            Self::NoLibPathGiven => write!(f, "no base path for lib paths has been given"),
            Self::PathBreaksOut => write!(f, "path escapes out of base path"),
        }
    }
}


#[derive(Debug)]
pub enum ProcessorInitializationError {
    FailedToProcessLibPath(std::io::Error),
    FailedToProcessRelPath(std::io::Error),
}


impl Display for ProcessorInitializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToProcessLibPath(_) => write!(f, "failed to process lib path"),
            Self::FailedToProcessRelPath(_) => write!(f, "failed to process rel path"),
        }
    }
}


impl Error for ProcessorInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FailedToProcessRelPath(err) | Self::FailedToProcessLibPath(err) => Some(err)
        }
    }
}

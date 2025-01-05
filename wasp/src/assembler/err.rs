use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::processor::Instruct;

#[derive(Debug, Clone)]
pub enum AssemblingError<PE>
    where PE: Error + Clone + 'static
{
    ProcessingError(PE),
    ProgTooLarge,
    InvalidInstruct {
        instruct: Instruct,
        info: InvalidInstructInfo
    },
}


impl<PE> Error for AssemblingError<PE>
    where PE: Error + Clone + 'static
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProcessingError(err) => Some(err),
            _ => None,
        }
    }
}


impl<PE> Display for AssemblingError<PE>
    where PE: Error + Clone + 'static
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessingError(_) => write!(f, "processing error occurred"),
            Self::ProgTooLarge => write!(f, "program too large to fit within 64KiB"),
            Self::InvalidInstruct { instruct, info } => write!(f, "invalid instruct ({instruct}): {info}"),
        }
    }
}


#[derive(Debug, Clone)]
pub enum InvalidInstructInfo {
    ModifyingLabel,
    ReferenceOutOfBounds,
}


impl Display for InvalidInstructInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ModifyingLabel => write!(f, "reassigning label is forbidden"),
            Self::ReferenceOutOfBounds => write!(f, "reference leads to non-existent instruction"),
        }
    }
}

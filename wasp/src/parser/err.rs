use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use crate::lexer::Word;

#[derive(Debug, Clone)]
pub enum ParsingError<LE: Error + Clone + 'static> {
    LexingError(LE),
    InvalidWord {
        word: Word,
        info: InvalidWordInfo
    }
}


impl<LE: Error + Clone> Display for ParsingError<LE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LexingError(_) => write!(f, "lexing error occurred"),
            Self::InvalidWord { word, info } => write!(f, "invalid word ({word}): {info}"),
        }
    }
}

impl<LE: Error + Clone> Error for ParsingError<LE> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LexingError(err) => Some(err),
            Self::InvalidWord { 
                info: InvalidWordInfo::LiteralInteger(err)
                    | InvalidWordInfo::Reference(err)
                    | InvalidWordInfo::Substitute(err), .. 
            } => Some(err),
            _ => None,
        }
    }
}


#[derive(Debug, Clone)]
pub enum InvalidWordInfo {
    SurroundPair,
    Register,
    LiteralInteger(ParseIntError),
    IntegerRadix,
    Char,
    Reference(ParseIntError),
    Substitute(ParseIntError),
}

impl Display for InvalidWordInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SurroundPair => write!(f, "prefix-suffix pair"),
            Self::Register => write!(f, "register name"),
            Self::LiteralInteger(_) => write!(f, "invalid integer"),
            Self::Reference(_) => write!(f, "invalid reference"),
            Self::Char => write!(f, "char word must be 1-char long"),
            Self::IntegerRadix => write!(f, "integer: radix"),
            Self::Substitute(_) => write!(f, "invalid substitute index"),
        }
    }
}

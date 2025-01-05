use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::lexer::Pos;

#[derive(Debug, Clone)]
pub struct LexingError {
    pub(super) pos: Pos,
    pub(super) info: LexingErrorInfo,
}


impl LexingError {
    pub fn pos(&self) -> Pos {
        self.pos
    } 
    
    pub fn info(&self) -> LexingErrorInfo {
        self.info
    }
}


#[derive(Debug, Clone, Copy)]
pub enum LexingErrorInfo {
    MultiplePrefixesEncountered,
    PrefixAtEnd,
    PrefixDetached,
    UnclosedSurroundPair,
    EscapingVoid,
}


impl Display for LexingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.info, self.pos)
    }
}


impl Display for LexingErrorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MultiplePrefixesEncountered => write!(f, "multiple prefixes in a row"),
            Self::PrefixAtEnd => write!(f, "prefix at the end of stream"),
            Self::PrefixDetached => write!(f, "prefix before whitespace"),
            Self::UnclosedSurroundPair => write!(f, "prefix-suffix pair hasn't been closed"),
            Self::EscapingVoid => write!(f, "you can't escape void."),
        }
    }
}


impl Error for LexingError {}

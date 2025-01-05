use std::fmt::{Display, Formatter};
use crate::lexer::pos::Pos;

#[derive(Debug, Clone)]
pub struct Word {
    pub(super) pos: Pos,
    pub(super) prefix: Option<char>,
    pub(super) suffix: Option<char>,
    pub(super) value: String,
}


impl Word {
    pub fn pos(&self) -> Pos {
        self.pos
    }
    
    pub fn prefix(&self) -> Option<char> {
        self.prefix
    }
    
    pub fn suffix(&self) -> Option<char> {
        self.suffix
    }
    
    pub fn value(&self) -> &str {
        &self.value
    }
    
    pub fn into_value(self) -> String {
        self.value
    }
}


impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ", self.pos)?;
        if let Some(prefix) = self.prefix {
            write!(f, "{prefix}")?;
        };
        write!(f, "{}", self.value)?;
        if let Some(suffix) = self.suffix {
            write!(f, "{suffix}")?;
        };
        Ok(())
    }
}

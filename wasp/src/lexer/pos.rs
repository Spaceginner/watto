use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, Default)]
pub struct Pos {
    abs: usize,
    line: usize,
    column: usize,
}


impl Pos {
    pub fn line(self) -> usize {
        self.line
    }
    
    pub fn column(self) -> usize {
        self.column
    }
    
    pub(super) fn next_line(&mut self) {
        self.abs += 1;
        self.line += 1;
        self.column = 0;
    }

    pub(super) fn next_col(&mut self) {
        self.abs += 1;
        self.column += 1;
    }
}


impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}:{:0>2}", self.line + 1, self.column + 1)
    }
}

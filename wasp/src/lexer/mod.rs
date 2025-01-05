use std::str::Chars;
pub use err::{LexingError, LexingErrorInfo};
pub use pos::Pos;
pub use word::Word;

mod word;
mod pos;
mod err;


pub struct Lexer<C: Iterator<Item = char>> {
    chars: C,
    pos: Pos,
    err: Option<LexingError>,
    buf: Option<char>,
}


impl<C: Iterator<Item = char>> Lexer<C> {
    pub const PREFIXES: [char; 8] = ['!', ':', '%', '~', '$', '#', '\'', '@'];
    pub const SURROUND_PAIRS: [(char, char); 1] = [('"', '"')];
    pub const ESCAPE: char = '\\';
    pub const COMMENT_PAIR: (char, char) = ('/', '\n');
    
    pub fn new(chars: C) -> Self {
        Self {
            chars,
            pos: Default::default(),
            err: None,
            buf: None,
        }
    }
    
    fn next_c(&mut self) -> Option<char> {
        let c = self.buf.take().or_else(|| self.chars.next())?;
        
        if c == '\n' {
            self.pos.next_line();
        } else {
            self.pos.next_col();
        };
        
        Some(c)
    }
    
    fn err(&mut self, err: LexingErrorInfo) -> Option<Result<Word, LexingError>> {
        let full_err = LexingError { pos: self.pos, info: err }; 
        self.err = Some(full_err.clone());
        Some(Err(full_err))
    }
}

impl Lexer<Chars<'_>> {
    pub fn lex(s: &str) -> Result<Vec<Word>, LexingError> {
        let mut chars = s.chars();
        Lexer::new(&mut chars).try_collect()
    }
}


impl<C: Iterator<Item = char>> Iterator for Lexer<C> {
    type Item = Result<Word, LexingError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.err.clone() {
            return Some(Err(err));
        };
        
        let start_pos = self.pos;
        let mut prefix = None;
        let mut buf = String::new();
        let mut in_comment = false;
        let mut in_surround = None;
        let mut escaping = false;
        
        loop {
            match self.next_c() {
                None => {
                    if in_surround.is_some() {
                        return self.err(LexingErrorInfo::UnclosedSurroundPair);
                    } else if escaping {
                        return self.err(LexingErrorInfo::EscapingVoid);
                    } else if prefix.is_some() && buf.is_empty() {
                        return self.err(LexingErrorInfo::PrefixAtEnd);
                    } else if !buf.is_empty() {
                        return Some(Ok(Word { prefix, value: buf, pos: start_pos, suffix: None }));
                    } else {
                        return None;
                    };
                },
                Some(c) => {
                    if escaping {
                        buf.push(c);
                        in_comment = false;
                        continue;
                    };
                    
                    if c == Self::ESCAPE {
                        escaping = true;
                    } else if let Some(end) = in_surround {
                        if c == end {
                            return Some(Ok(Word { prefix, value: buf, pos: start_pos, suffix: Some(end) }));
                        } else {
                            buf.push(c);
                        };
                    } else if in_comment {
                        if c == Self::COMMENT_PAIR.1 {
                            in_comment = false;
                        };
                    } else if c == Self::COMMENT_PAIR.0 {
                        in_comment = true;
                    } else if let Some((start, end)) = Self::SURROUND_PAIRS.iter().copied().find(|(start, _)| *start == c) {
                        prefix = Some(start);
                        in_surround = Some(end);
                    } else if Self::PREFIXES.contains(&c) {
                        if !buf.is_empty() {
                            self.buf = Some(c);
                            return Some(Ok(Word { prefix, value: buf, pos: start_pos, suffix: None }));
                        } else if prefix.is_some() {
                            return self.err(LexingErrorInfo::MultiplePrefixesEncountered);
                        } else {
                            prefix = Some(c);
                        };
                    } else if c.is_whitespace() {
                        if !buf.is_empty() {
                            return Some(Ok(Word { prefix, value: buf, pos: start_pos, suffix: None }));
                        } else if prefix.is_some() {
                            return self.err(LexingErrorInfo::PrefixDetached);
                        };
                    } else {
                        buf.push(c);
                    };
                },
            };
        };
    }
}

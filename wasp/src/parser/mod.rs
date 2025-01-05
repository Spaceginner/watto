use std::error::Error;
use std::str::{Chars, FromStr};
use watto::Register;
use crate::lexer::{Lexer, LexingError, Word};

pub use element::{Element, LiteralValue, ElementValue};
pub use err::{InvalidWordInfo, ParsingError};

mod element;
mod err;

pub struct Parser<L, LE>
    where
        L: Iterator<Item = Result<Word, LE>>,
        LE: Error + Clone + 'static,
{
    lexer: L,
    err: Option<ParsingError<LE>>,
}

impl<L, LE> Parser<L, LE>
    where
        L: Iterator<Item = Result<Word, LE>>,
        LE: Error + Clone + 'static,
{
    pub fn new(lexer: L) -> Self {
        Self { lexer, err: None }
    }

    fn err(&mut self, err: ParsingError<LE>) -> Option<Result<Element, ParsingError<LE>>> {
        self.err = Some(err.clone());
        Some(Err(err))
    }
}


impl<L, LE> Iterator for Parser<L, LE>
    where
        L: Iterator<Item = Result<Word, LE>>,
        LE: Error + Clone + 'static,
{
    type Item = Result<Element, ParsingError<LE>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.err.clone() {
            return Some(Err(err));
        };

        Some(match self.lexer.next()? {
            Ok(word) =>
                Ok(match (word.prefix(), word.suffix()) {
                    (None, None) => Element { pos: word.pos(), value: ElementValue::CpuInstruction(word.into_value()) },
                    (Some('!'), None) => Element { pos: word.pos(), value: ElementValue::ProcessorInstruction(word.into_value()) },
                    (Some('%'), None) => Element { pos: word.pos(), value: ElementValue::Variable(word.into_value()) },
                    (Some('~'), None) => {
                        match word.value().parse() {
                            Ok(delta) => Element { pos: word.pos(), value: ElementValue::Reference(delta) },
                            Err(err) => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::Reference(err) }); },
                        }
                    },
                    (Some('#'), None) => {
                        let radix = match word.value().chars().next().unwrap() {
                            'x' => 16,
                            'o' => 8,
                            'b' => 2,
                            'd' => 10,
                            _ => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::IntegerRadix }) }
                        };

                        match u16::from_str_radix(word.value().split_at(1).1, radix) {
                            Ok(val) => Element { pos: word.pos(), value: ElementValue::Literal(LiteralValue::Number(val)) },
                            Err(err) => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::LiteralInteger(err) }); }
                        }
                    },
                    (Some('\''), None) => {
                        if word.value().len() != 1 {
                            return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::Char });
                        } else {
                            Element { pos: word.pos(), value: ElementValue::Literal(LiteralValue::Char(word.into_value().chars().next().unwrap())) }
                        }
                    },
                    (Some(':'), None) => Element { pos: word.pos(), value: ElementValue::Label(word.into_value()) },
                    (Some('$'), None) => {
                        match Register::try_from(word.value()) {
                            Ok(reg) => Element { pos: word.pos(), value: ElementValue::Register(reg) },
                            Err(()) => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::Register }); }
                        }
                    },
                    (Some('"'), Some('"')) => Element { pos: word.pos(), value: ElementValue::Literal(LiteralValue::String(word.into_value())) },
                    (Some('@'), None) => {
                        match word.value().parse::<usize>() {
                            Ok(i) => Element { pos: word.pos(), value: ElementValue::Substitute(i) },
                            Err(err) => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::Substitute(err) }); },
                        }
                    },
                    (_, _) => { return self.err(ParsingError::InvalidWord { word, info: InvalidWordInfo::SurroundPair }); }
                }),
            Err(err) => { return self.err(ParsingError::LexingError(err)); }
        })
    }
}


impl Parser<Lexer<Chars<'_>>, LexingError> {
    pub fn parse(s: &str) -> Result<Vec<Element>, ParsingError<LexingError>> {
        Parser::new(Lexer::new(s.chars())).try_collect()
    }
}

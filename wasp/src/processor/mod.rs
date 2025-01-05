use std::collections::HashMap;
use std::error::Error;
use std::ffi::CString;
use std::path::PathBuf;
use std::str::Chars;
use watto::InstructionId;
use crate::parser::{Element, ElementValue, LiteralValue, Parser, ParsingError};
use r#macro::{CurrentMacro, Macro};
use crate::lexer::{Lexer, LexingError};

pub use err::{InvalidElementInfo, ProcessingError};
pub use instruct::{Argument, Instruct, Op, ValueArgument};

mod instruct;
mod r#macro;
mod err;

pub struct Processor<P, PE>
    where
        P: Iterator<Item = Result<Element, PE>>,
        PE: Error + Clone + 'static
{
    parser: P,
    err: Option<ProcessingError<PE>>,

    include_lib_root: Option<PathBuf>,
    include_rel_root: Option<PathBuf>,
    include_abs: bool,
    
    macros: HashMap<String, Macro>,
    cur_macro: Vec<CurrentMacro>,
}


impl<P, PE> Processor<P, PE>
    where
        P: Iterator<Item = Result<Element, PE>>,
        PE: Error + Clone + 'static
{
    pub fn new(parser: P, include_lib_root: Option<PathBuf>, include_rel_root: Option<PathBuf>, include_abs: bool) -> Self {
        Self { parser, include_lib_root, include_rel_root, include_abs, err: None, macros: HashMap::new(), cur_macro: vec![] }
    }
    
    fn err(&mut self, err: ProcessingError<PE>) -> Option<Result<Instruct, ProcessingError<PE>>> {
        self.err = Some(err.clone());
        Some(Err(err))
    }
    
    fn next_el(&mut self) -> Option<Result<Element, PE>> {
        while let Some(cur_macro) = self.cur_macro.last_mut() {
            if let Some(el) = cur_macro.next() {
                return Some(Ok(el))
            } else {
                self.cur_macro.pop();
            };
        };

        self.parser.next()
    }
}


macro_rules! nextcel {
    ($s:expr, $pat:pat => $code:expr) => {
        match $s.next_el() {
            Some(Ok($pat)) => $code,
            Some(Ok(elem)) => { return $s.err(ProcessingError::InvalidElement { elem, info: InvalidElementInfo::ProcessorInstructArg }) },
            Some(Err(err)) => { return $s.err(ProcessingError::ParsingFailure(err)); }
            None => { return $s.err(ProcessingError::EarlyEoE); },
        }
    };
    ($s:expr) => {
        match $s.next_el() {
            Some(Ok(elem)) => elem,
            Some(Err(err)) => { return $s.err(ProcessingError::ParsingFailure(err)); }
            None => { return $s.err(ProcessingError::EarlyEoE); },
        }
    }
}


impl<P, PE> Iterator for Processor<P, PE>
    where
        P: Iterator<Item = Result<Element, PE>>,
        PE: Error + Clone + 'static
{
    type Item = Result<Instruct, ProcessingError<PE>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.err.clone() {
            return Some(Err(err));
        };

        let mut labels = Vec::new();

        loop {
            match self.next_el()? {
                Ok(elem) =>
                    match elem {
                        Element { pos, value: ElementValue::CpuInstruction(name) } => {
                            match InstructionId::try_from(name.as_str()) {
                                Ok(id) => {
                                    let mut args = Vec::new();
                                    for expected in id.arguments().into_iter() {
                                        match self.next_el() {
                                            Some(Ok(elem)) => {
                                                match expected {
                                                    watto::Argument::Register => {
                                                        match elem {
                                                            Element { value: ElementValue::Register(reg), .. } => { args.push(Argument::Register(reg)); },
                                                            elem => { return self.err(ProcessingError::InvalidElement { elem, info: InvalidElementInfo::CpuInstructionArg { expected } }); },
                                                        };
                                                    },
                                                    watto::Argument::Number => {
                                                        match elem {
                                                            Element { value: ElementValue::Literal(lit), pos } =>
                                                                match lit {
                                                                    LiteralValue::Char(c) => {
                                                                        match c.as_ascii() {
                                                                            Some(cc) => args.push(Argument::Value(ValueArgument::Literal(cc.to_u8() as u16))),
                                                                            None => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::Literal(LiteralValue::Char(c))), info: InvalidElementInfo::NonAsciiCharAsArg }); },
                                                                        }
                                                                    },
                                                                    LiteralValue::Number(n) => args.push(Argument::Value(ValueArgument::Literal(n))),
                                                                    _ => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::Literal(lit)), info: InvalidElementInfo::CpuInstructionArg { expected } }); }
                                                                },
                                                            Element { value: ElementValue::Reference(delta), .. } => { args.push(Argument::Value(ValueArgument::Reference(delta))); },
                                                            Element { value: ElementValue::Variable(name), .. } => { args.push(Argument::Value(ValueArgument::Variable(name))); },
                                                            elem => { return self.err(ProcessingError::InvalidElement { elem, info: InvalidElementInfo::CpuInstructionArg { expected } }); },
                                                        }
                                                    }
                                                };
                                            },
                                            Some(Err(err)) => { return self.err(ProcessingError::ParsingFailure(err)); },
                                            None => { return self.err(ProcessingError::EarlyEoE); }
                                        }
                                    };
                                    return Some(Ok(Instruct { pos, labels, operation: Op::InsertCpuInstruction(id, args) }));
                                },
                                Err(()) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::CpuInstruction(name)), info: InvalidElementInfo::CpuInstructionName }); }
                            };
                        },
                        Element { pos, value: ElementValue::ProcessorInstruction(name) } =>
                            match name.as_str() {
                                "byte" => nextcel!{ self,
                                    Element { value: ElementValue::Literal(LiteralValue::Number(n)), pos: epos } => {
                                        match n.try_into() {
                                            Ok(b) => { return Some(Ok(Instruct { pos, labels, operation: Op::InsertByte(b) })); },
                                            Err(_) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::Number(n))), info: InvalidElementInfo::ProcessorInstructArg }) },
                                        };
                                    }
                                },
                                "bytes" => {
                                    let b = nextcel!{ self,
                                        Element { value: ElementValue::Literal(LiteralValue::Number(n)), pos: epos } => {
                                            match n.try_into() {
                                                Ok(b) => { b },
                                                Err(_) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::Number(n))), info: InvalidElementInfo::ProcessorInstructArg }) },
                                            }
                                        }
                                    };
                                    
                                    let count = nextcel!{ self, Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => n };
                                    
                                    return Some(Ok(Instruct { pos, labels, operation: Op::InsertBytes(b, count) }));
                                }
                                "word" => nextcel!{ self,
                                    Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => {
                                        return Some(Ok(Instruct { pos, labels, operation: Op::InsertWord(n) }));
                                    }
                                },
                                "file" => nextcel!{ self,
                                    Element { value: ElementValue::Literal(LiteralValue::String(s)), .. } => {
                                        return Some(Ok(Instruct { pos, labels, operation: Op::InsertFile(std::path::PathBuf::from(s)) }));
                                    }
                                },
                                "cstr" => nextcel!{ self,
                                    Element { value: ElementValue::Literal(LiteralValue::String(s)), pos: epos } => {
                                        match CString::new(s) {
                                            Ok(cstr) => { return Some(Ok(Instruct { pos, labels, operation: Op::InsertCString(cstr) })); },
                                            Err(err) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::String(unsafe { String::from_utf8_unchecked(err.into_vec()) }))), info: InvalidElementInfo::ProcessorInstructArg }) },
                                        };
                                    }
                                },
                                "set" => {
                                    let name = nextcel!{ self, Element { value: ElementValue::Variable(name), .. } => name };
                                    let val = nextcel!{ self, Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => n };
                                    
                                    return Some(Ok(Instruct { pos, labels, operation: Op::SetVariable(name, val) }));
                                },
                                "include" => { todo!() },
                                "lib" => { todo!() },
                                "macro" => {
                                    let name = nextcel!{ self, Element { value: ElementValue::CpuInstruction(name), .. } => name };
                                    let arg_count = nextcel!{ self, Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => n as usize };
                                    let source = nextcel!{ self, 
                                        Element { value: ElementValue::Literal(LiteralValue::String(source)), pos } => {
                                            match Parser::parse(&source) {
                                                Ok(elems) => elems,
                                                Err(err) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::Literal(LiteralValue::String(source))), info: InvalidElementInfo::MacroParsingFailure(err) }); },
                                            }
                                        }
                                    };
                                    
                                    self.macros.insert(name, Macro { sub_count: arg_count, source });
                                },
                                "m" => {
                                    let r#macro = nextcel!{ self, 
                                        Element { value: ElementValue::CpuInstruction(name), pos } => {
                                            match self.macros.get(&name) {
                                                Some(m) => m,
                                                None => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::CpuInstruction(name)), info: InvalidElementInfo::MacroName }); }, 
                                            }
                                        }
                                    };
                                    
                                    let source = r#macro.source.clone();
                                    let mut subs = Vec::new();
                                    for _ in 0..r#macro.sub_count {
                                        subs.push(nextcel! { self });
                                    };
                                    
                                    self.cur_macro.push(CurrentMacro::new(source, subs));
                                },
                                "void" => { return Some(Ok(Instruct { pos, labels, operation: Op::Void })); },
                                _ => { return Some(Err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::ProcessorInstruction(name)), info: InvalidElementInfo::ProcessorInstructName })); }
                            },
                        Element { value: ElementValue::Label(name), .. } => { labels.push(name); },
                        elem => { return self.err(ProcessingError::InvalidElement { elem, info: InvalidElementInfo::Unexpected }); }
                    }
                Err(err) => { return self.err(ProcessingError::ParsingFailure(err)); }
            }

        }
    }
}


impl Processor<Parser<Lexer<Chars<'_>>, LexingError>, ParsingError<LexingError>> {
    pub fn process(src: &str) -> Result<Vec<Instruct>, ProcessingError<ParsingError<LexingError>>> {
        Processor::new(Parser::new(Lexer::new(src.chars())), None, None, false).try_collect()
    }
}

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::CString;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::Chars;
use std::string::IntoChars;
use normalize_path::NormalizePath;
use resolve_path::PathResolveExt;
use watto::InstructionId;
use crate::parser::{Element, ElementValue, LiteralValue, Parser, ParsingError};
use crate::lexer::{Lexer, LexingError, Pos};
use r#macro::{CurrentMacro, Macro};

pub use err::{InvalidElementInfo, ProcessingError, ProcessorInitializationError};
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

    paths_lib_root: Option<PathBuf>,  // todo allow specifying multiple lib directories
    paths_rel_root: Option<PathBuf>,
    allow_abs_paths: bool,

    included_files: HashMap<PathBuf, HashMap<String, Macro>>,

    defined_macros: HashMap<String, Macro>,
    included_macros: HashMap<String, Macro>,
    cur_macro: Vec<CurrentMacro>,

    cur_processor: Option<(Box<Processor<Parser<Lexer<IntoChars>, LexingError>, ParsingError<LexingError>>>, PathBuf, Pos)>,
}


macro_rules! proc_path {
    ($path:expr, $err:ident) => {
        $path.as_mut().map(|p| *p = p.normalize());
        if let Some(Err(err)) = $path.as_mut().filter(|p| !p.is_absolute()).map(|p| Ok(*p = p.try_resolve()?.into_owned())) {
            return Err(ProcessorInitializationError::$err(err));
        };
    };
}


impl<P, PE> Processor<P, PE>
    where
        P: Iterator<Item = Result<Element, PE>>,
        PE: Error + Clone + 'static
{
    pub fn new(parser: P, mut paths_lib_root: Option<PathBuf>, mut paths_rel_root: Option<PathBuf>, allow_abs_paths: bool) -> Result<Self, ProcessorInitializationError> {
        proc_path!(paths_lib_root, FailedToProcessLibPath);
        proc_path!(paths_rel_root, FailedToProcessRelPath);

        Ok(Self { parser, paths_lib_root, paths_rel_root, allow_abs_paths, included_files: HashMap::new(), err: None, included_macros: HashMap::new(), defined_macros: HashMap::new(), cur_processor: None, cur_macro: vec![] })
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

        // todo handle not in macro
        self.parser.next()
    }
}


macro_rules! nextpath {
    ($s:expr, $base:expr, $err:ident, $path:ident, $pos:ident, $code:expr) => {
        nextcel!{ $s,
            Element { value: ElementValue::Literal(LiteralValue::String(s)), pos: $pos } => {
                let given_path = std::path::PathBuf::from(s);

                let $path;
                if given_path.is_absolute() {
                    if !$s.allow_abs_paths {
                        return $s.err(ProcessingError::InvalidElement { elem: Element::new($pos, ElementValue::Literal(LiteralValue::String(given_path.to_string_lossy().to_string()))), info: InvalidElementInfo::AbsolutePathsForbidden });
                    } else {
                        $path = given_path;
                    };
                } else {
                    if let Some(base_path) = &$base {
                        let mut path = base_path.clone();
                        path.push(given_path);
                        $path = path.normalize();

                        if $path.strip_prefix(base_path).is_err() {
                            return $s.err(ProcessingError::InvalidElement { elem: Element::new($pos, ElementValue::Literal(LiteralValue::String($path.to_string_lossy().to_string()))), info: InvalidElementInfo::PathBreaksOut });
                        };
                    } else {
                        return $s.err(ProcessingError::InvalidElement { elem: Element::new($pos, ElementValue::Literal(LiteralValue::String(given_path.to_string_lossy().to_string()))), info: InvalidElementInfo::$err });
                    };
                };

                $code
            }
        }
    };
    ($s:expr, rel, $path:ident, $pos:ident, $code:expr) => {
        nextpath!($s, $s.paths_rel_root, NoRelPathGiven, $path, $pos, $code)
    };
    ($s:expr, lib, $path:ident, $pos:ident, $code:expr) => {
        nextpath!($s, $s.paths_lib_root, NoLibPathGiven, $path, $pos, $code)
    };
}


macro_rules! nextfile {
    ($s:expr, bin, rel, $path:ident, $data:ident, $code:expr) => {
        nextpath!{ $s, rel, $path, epos, {
            match std::fs::read(&$path) {
                Ok($data) => $code,
                Err(err) => { return $s.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::String($path.to_string_lossy().to_string()))), info: InvalidElementInfo::FailedToReadFile { reason: err.to_string() } }); },  // fixme provide correct pos
            };
        }}
    };
    ($s:expr, bin, lib, $path:ident, $data:ident, $code:expr) => {
        nextpath!{ $s, lib, $path, epos, {
            match std::fs::read(&$path) {
                Ok($data) => $code,
                Err(err) => { return $s.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::String($path.to_string_lossy().to_string()))), info: InvalidElementInfo::FailedToReadFile { reason: err.to_string() } }); },  // fixme provide correct pos
            };
        }}
    };
    ($s:expr, str, rel, $path:ident, $data:ident, $code:expr) => {
        nextpath!{ $s, rel, $path, epos, {
            match std::fs::read_to_string(&$path) {
                Ok($data) => $code,
                Err(err) => { return $s.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::String($path.to_string_lossy().to_string()))), info: InvalidElementInfo::FailedToReadFile { reason: err.to_string() } }); },  // fixme provide correct pos
            };
        }}
    };
    ($s:expr, str, lib, $path:ident, $data:ident, $code:expr) => {
        nextpath!{ $s, lib, $path, epos, {
            match std::fs::read_to_string(&$path) {
                Ok($data) => $code,
                Err(err) => { return $s.err(ProcessingError::InvalidElement { elem: Element::new(epos, ElementValue::Literal(LiteralValue::String($path.to_string_lossy().to_string()))), info: InvalidElementInfo::FailedToReadFile { reason: err.to_string() } }); },  // fixme provide correct pos
            };
        }}
    };
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
        loop {
            if let Some(err) = self.err.clone() {
                return Some(Err(err));
            };

            let mut err = None;
            if let Some((proc, path, pos)) = &mut self.cur_processor {
                match proc.next() {
                    Some(Ok(instr)) => { return Some(Ok(instr)); },
                    Some(Err(err_)) => { err = Some(ProcessingError::InvalidElement { elem: Element::new(*pos, ElementValue::Literal(LiteralValue::String(path.to_string_lossy().to_string()))), info: InvalidElementInfo::IncludedFileProcessingFailure(Box::new(err_)) }); },
                    None => {
                        self.included_files.extend(std::mem::take(&mut proc.included_files).into_iter());
                        self.included_macros.extend(proc.defined_macros.iter().map(|(k, v)| (k.clone(), v.clone())));
                        self.included_files.insert(path.clone(), std::mem::take(&mut proc.defined_macros));
                        self.cur_processor = None;
                    },
                };
            };
            if let Some(err) = err {
                return self.err(err);
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

                                        return Some(Ok(Instruct { pos, labels, operation: Op::InsertMultipleBytes(b, count) }));
                                    }
                                    "word" => nextcel!{ self,
                                    Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => {
                                        return Some(Ok(Instruct { pos, labels, operation: Op::InsertWord(n) }));
                                    }
                                },
                                    "file" => nextfile!{ self, bin, rel, path, bytes, {
                                    return Some(Ok(Instruct { pos, labels, operation: Op::InsertBytes(bytes) }));
                                }},
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
                                    "include" => nextfile!{ self, str, rel, path, code, {
                                    if let Some(macros) = self.included_files.get(&path) {
                                        self.included_macros.extend(macros.iter().map(|(k, v)| (k.clone(), v.clone())));
                                        continue;
                                    };

                                    let mut processor = Processor::new(Parser::new(Lexer::new(code.into_chars())), self.paths_lib_root.clone(), Some(path.parent().unwrap().to_path_buf()), self.allow_abs_paths).unwrap();
                                    processor.included_files = std::mem::take(&mut self.included_files);
                                    self.cur_processor = Some((Box::new(processor), path, pos));
                                    break;
                                }},
                                    "lib" => nextfile!{ self, str, lib, path, code, {
                                    if let Some(macros) = self.included_files.get(&path) {
                                        self.included_macros.extend(macros.iter().map(|(k, v)| (k.clone(), v.clone())));
                                        continue;
                                    };

                                    let mut processor = Processor::new(Parser::new(Lexer::new(code.into_chars())), self.paths_lib_root.clone(), Some(path.parent().unwrap().to_path_buf()), true).unwrap();
                                    processor.included_files = std::mem::take(&mut self.included_files);
                                    self.cur_processor = Some((Box::new(processor), path, pos));
                                    break;
                                }},
                                    "macro" => {
                                        let name = nextcel!{ self, Element { value: ElementValue::CpuInstruction(name), .. } => name };
                                        let arg_count = nextcel!{ self, Element { value: ElementValue::Literal(LiteralValue::Number(n)), .. } => n as usize };
                                        let source = nextcel!{ self,
                                        Element { value: ElementValue::Literal(LiteralValue::String(source)), pos } => {
                                            match Parser::parse(&source) {
                                                Ok(elems) => elems,
                                                Err(err) => { return self.err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::Literal(LiteralValue::String(source))), info: InvalidElementInfo::IncludedCodeParsingFailure(err) }); },
                                            }
                                        }
                                    };

                                        self.defined_macros.insert(name, Macro { sub_count: arg_count, source });
                                    },
                                    "m" => {
                                        let r#macro = nextcel!{ self,
                                        Element { value: ElementValue::CpuInstruction(name), pos } => {
                                            match self.defined_macros.get(&name).or_else(|| self.included_macros.get(&name)) {
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
                                    "ifenv" => {
                                        todo!()
                                    },
                                    "iffeat" => {
                                        todo!()
                                    },
                                    "void" => { return Some(Ok(Instruct { pos, labels, operation: Op::Void })); },
                                    _ => { return Some(Err(ProcessingError::InvalidElement { elem: Element::new(pos, ElementValue::ProcessorInstruction(name)), info: InvalidElementInfo::ProcessorInstructName })); }
                                },
                            Element { value: ElementValue::Label(name), .. } => { labels.push(name); },
                            elem => { return self.err(ProcessingError::InvalidElement { elem, info: InvalidElementInfo::Unexpected }); }
                        }
                    Err(err) => { return self.err(ProcessingError::ParsingFailure(err)); }
                };
            }
        }
    }
}


#[derive(Debug)]
pub enum ProcessingShortcutError {
    ProcessingError(ProcessingError<ParsingError<LexingError>>),
    InitializationError(ProcessorInitializationError),
}


impl Display for ProcessingShortcutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProcessingError(_) => write!(f, "while processing an error occurred"),
            Self::InitializationError(_) => write!(f, "failed to initialize processor"),
        }
    }
}


impl Error for ProcessingShortcutError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ProcessingError(err) => Some(err),
            Self::InitializationError(err) => Some(err),
        }
    }
}


impl Processor<Parser<Lexer<Chars<'_>>, LexingError>, ParsingError<LexingError>> {
    pub fn process(src: &str) -> Result<Vec<Instruct>, ProcessingShortcutError> {
        Ok(Processor::new(Parser::new(Lexer::new(src.chars())), None, None, false).map_err(ProcessingShortcutError::InitializationError)?.try_collect().map_err(ProcessingShortcutError::ProcessingError)?)
    }

    pub fn process_custom(src: &str, lib_path: Option<PathBuf>, rel_path: Option<PathBuf>, allow_abs_paths: bool) -> Result<Vec<Instruct>, ProcessingShortcutError> {
        Ok(Processor::new(Parser::new(Lexer::new(src.chars())), lib_path, rel_path, allow_abs_paths).map_err(ProcessingShortcutError::InitializationError)?.try_collect().map_err(ProcessingShortcutError::ProcessingError)?)
    }
}

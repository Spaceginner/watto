#![feature(iterator_try_collect)]
#![feature(ascii_char)]
#![feature(let_chains)]
#![feature(string_into_chars)]

use std::error::Error;
use std::io::{Read, Write};
use clap::Parser;
use crate::argparser::Format;
use crate::assembler::Assembler;
use crate::lexer::Lexer;
use crate::processor::Processor;

mod argparser;
mod lexer;
mod parser;
mod processor;
mod assembler;

fn handle_error(context: &'static str, mut err: &dyn Error) -> ! {
    eprintln!("while {context}, an error occurred: {err}");
    
    while let Some(source) = err.source() {
        err = source;
        eprintln!("source of which: {err}");
    };
    
    std::process::exit(1)
}

fn main() {
    let args: argparser::AsmArgs = argparser::AsmArgs::parse();

    let rel_path = args.source.is_local().then(|| args.source.parent().unwrap().to_path_buf());
    
    let source = {
        let mut buf = String::new();
        args.source.read_all().unwrap().read_to_string(&mut buf).unwrap_or_else(|err| handle_error("reading source", &err));
        buf
    };
    
    match args.format {
        Format::Binary => {
            let prog = 
                Assembler::new(Processor::new(parser::Parser::new(Lexer::new(source.chars())), args.lib_path.map(|p| p.to_path_buf()), rel_path, !args.forbid_abs_includes).unwrap_or_else(|err| handle_error("initializing processor", &err)))
                .assemble()
                .unwrap_or_else(|err| handle_error("assembling program", &err));

            let mut out = args.out.create_with_len(prog.len() as u64).unwrap_or_else(|err| handle_error("creating output stream", &err));

            if !args.dry {
                out.write_all(&prog).unwrap_or_else(|err| handle_error("writing to output", &err));
            };
        },
        Format::Words => {
            let words = Lexer::lex(&source).unwrap_or_else(|err| handle_error("lexing program", &err));
            
            let mut out = args.out.create().unwrap_or_else(|err| handle_error("creating output stream", &err));

            if !args.dry {
                for word in words {
                    writeln!(out, "{word}").unwrap_or_else(|err| handle_error("writing to output", &err));
                };
            };
        },
        Format::Elements => {
            let elements = parser::Parser::parse(&source).unwrap_or_else(|err| handle_error("parsing program", &err));

            let mut out = args.out.create().unwrap_or_else(|err| handle_error("creating output stream", &err));

            if !args.dry {
                for elem in elements {
                    writeln!(out, "{elem}").unwrap_or_else(|err| handle_error("writing to output", &err));
                };
            };
        },
        Format::Instructs => {
            let instructs = Processor::process_custom(&source, args.lib_path.map(|p| p.to_path_buf()), rel_path, !args.forbid_abs_includes).unwrap_or_else(|err| handle_error("processing program", &err));

            let mut out = args.out.create().unwrap_or_else(|err| handle_error("creating output stream", &err));

            if !args.dry {
                for instr in instructs {
                    writeln!(out, "{instr}").unwrap_or_else(|err| handle_error("writing to output", &err));
                };
            };
        },
    }

    
}

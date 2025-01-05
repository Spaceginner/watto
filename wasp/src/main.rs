#![feature(iterator_try_collect)]
#![feature(ascii_char)]
#![feature(let_chains)]

use std::error::Error;
use std::io::{Read, Write};
use clap::Parser;
use crate::assembler::Assembler;
use crate::lexer::Lexer;
use crate::processor::Processor;

mod argparser;
mod lexer;
mod parser;
mod processor;
mod assembler;

fn handle_error(mut err: &dyn Error) -> ! {
    println!("error occurred: {err}");
    
    while let Some(source) = err.source() {
        err = source;
        println!("source of which: {err}");
    };
    
    std::process::exit(1)
}

fn main() {
    let args: argparser::AsmArgs = argparser::AsmArgs::parse();

    let source = {
        let mut buf = String::new();
        args.source.read_all().unwrap().read_to_string(&mut buf).unwrap();
        buf
    };

    let prog = Assembler::new(Processor::new(parser::Parser::new(Lexer::new(source.chars())), None, None, false)).assemble().unwrap_or_else(|err| handle_error(&err));
    
    let mut out = args.out.create_with_len(prog.len() as u64).unwrap();
    
    if !args.dry {
        out.write_all(&prog).unwrap();
    };
}

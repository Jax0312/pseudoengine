#![feature(iter_advance_by)]

use std::{fs::File, io::Read};

mod executor;
mod tokens;
mod parser;
mod enums;
mod lexer;
mod utils;

const FILEPATH: &str = "inputs/expression.txt";

fn main() {
    // Read input

    let mut file = File::open(FILEPATH).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let mut lex = lexer::lexer(&mut buf.chars()).into_iter().peekable();
    parser::parse_file(&mut lex);
    // executor::run(parser::parse(&mut lexer::lexer(&mut buf)));
    
}


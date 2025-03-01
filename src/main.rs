#![allow(warnings)]
use std::{fs::File, io::Read};

mod tokens;
mod parser;
mod enums;
mod lexer;
mod utils;
mod executor;

const FILEPATH: &str = "inputs/func_test.txt";

fn main() {
    // Read input
    println!("Executing {}", FILEPATH);
    let mut file = File::open(FILEPATH).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    // Trim and end a newline for better error reporting
    buf = buf.trim().parse().unwrap();
    buf.push('\n');
    let mut lex = lexer::lexer(&mut buf.chars()).into_iter().peekable();
    let ast = parser::parse_file(&mut lex);
    executor::run(ast);
}


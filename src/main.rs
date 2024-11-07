use std::{ fs::File, io::Read};
mod executor;
mod tokens;
mod parser;

use crate::tokens::Token;
use logos::{Logos};

fn main() {
    // Read input
    let mut file = File::open("inputs/expression.txt").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let mut lex = Token::lexer(buf.as_str());
    for result in lex {
        match result {
            Ok(token) => println!("{:#?}", token),
            Err(_) => println!("Undefined Character"),
        }
    }
    // executor::run(parser::parse(&mut lexer::lexer(&mut buf)));
    
}


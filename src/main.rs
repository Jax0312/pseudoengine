use std::{fs::File, io::Read};
mod lexer;
mod parser;
mod executor;

fn main() {
    // Read input
    // let mut file = File::open("inputs/expression.txt").unwrap();
    // let mut buf = String::new();
    // file.read_to_string(&mut buf).unwrap();
    // let mut buf = buf.chars();
    //
    // executor::run(parser::parse(&mut lexer::lexer(&mut buf)));

    let mut file = File::open("inputs/demo.txt").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let mut buf = buf.chars();

    let mut tokens = lexer::lexer(&mut buf);
    // let mut ast = parser::parse(&mut tokens);
    // executor::run(ast);

    
}


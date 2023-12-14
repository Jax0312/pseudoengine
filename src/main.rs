use std::{fs::File, io::Read};
mod lexer;
mod parser;

fn main() {
    // Read input
    let mut file = File::open("inputs/Q01.txt").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf);
    let mut buf = buf.chars();
    
    parser::parse(&mut lexer::lexer(&mut buf));
    
    
    
}

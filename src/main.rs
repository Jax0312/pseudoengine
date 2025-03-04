#![allow(warnings)]
use std::{env, fs::File, io::Read};
use std::fmt::format;

mod tokens;
mod parser;
mod enums;
mod lexer;
mod utils;
mod executor;

const DEBUG_FILEPATH: &str = "tests/classes_test.txt";

fn main() {
    // Read input
    println!("The current directory is {}",  env::current_dir().unwrap().display());
    let filepath = if cfg!(debug_assertions) {
        DEBUG_FILEPATH
    } else {
        let args: Vec<String> = env::args().collect();
        &*match args.get(1) {
            Some(arg) => arg.clone(),
            None => panic!("Missing filepath argument"),
        }
    };
    execute(filepath);
}

fn execute(filepath: &str) {
    println!("Executing {}", filepath);
    let mut file = File::open(filepath).expect(format!("File {} not found", filepath).as_str());
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    // Trim and end a newline for better error reporting
    buf = buf.trim().parse().unwrap();
    buf.push('\n');
    let mut lex = lexer::lexer(&mut buf.chars()).into_iter().peekable();
    let ast = parser::parse_file(&mut lex);
    executor::run(ast);
}
#[cfg(test)]
mod tests {
    use crate::execute;
    #[test]
    fn builtin_func_test() {
        execute("tests/builtin_func_test.txt");
    }
    // TODO: Uncomment the test when you're done @JinWei
    // #[test]
    // fn func_test() {
    //     execute("tests/func_test.txt");
    // }
    #[test]
    fn file_test() {
        execute("tests/file_test.txt");
    }
    #[test]
    fn classes_test() {
        execute("tests/classes_test.txt");
    }
    #[test]
    fn pointer_test() {
        execute("tests/pointer_test.txt");
    }
    
}


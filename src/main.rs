#![allow(warnings)]
use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;
use std::{env, fs::read_to_string, io::Read};

mod enums;
mod executor;
mod lexer;
mod parser;
mod tokens;
mod utils;

const DEBUG_FILEPATH: &str = "tests/classes_test.txt";

#[derive(Clone)]
struct SourceFile {
    name: String,
    file: Vec<String>,
}

impl SourceFile {
    pub fn new() -> Self {
        SourceFile {
            name: String::new(),
            file: Vec::new(),
        }
    }
}

thread_local! {
    pub static SOURCE_FILE: Rc<RefCell<SourceFile>> = Rc::new(RefCell::new(SourceFile::new()));
}

fn main() {
    // Read input
    println!(
        "The current directory is {}",
        env::current_dir().unwrap().display()
    );
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
    let mut buf = read_to_string(filepath).expect(format!("File {} not found", filepath).as_str());
    let lines = buf.clone().lines().map(|line| line.to_string()).collect::<Vec<String>>();
    SOURCE_FILE.with(|file| {
        file.replace(SourceFile {
            name: filepath.to_string(),
            file: lines.clone(),
        })
    });
    // Trim and end a newline for better error reporting
    buf = buf.parse().unwrap();
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

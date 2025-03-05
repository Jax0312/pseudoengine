#![allow(warnings)]
use std::cell::RefCell;
use std::fmt::format;
use std::rc::Rc;
use std::{env, fs::read_to_string, io::Read};

use clap::{Arg, Command};

mod enums;
mod executor;
mod lexer;
mod parser;
mod tokens;
mod utils;

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

const HEADER: &str = r#"
██████╗ ███████╗███████╗██╗   ██╗██████╗  ██████╗ ███████╗███╗   ██╗ ██████╗ ██╗███╗   ██╗███████╗
██╔══██╗██╔════╝██╔════╝██║   ██║██╔══██╗██╔═══██╗██╔════╝████╗  ██║██╔════╝ ██║████╗  ██║██╔════╝
██████╔╝███████╗█████╗  ██║   ██║██║  ██║██║   ██║█████╗  ██╔██╗ ██║██║  ███╗██║██╔██╗ ██║█████╗  
██╔═══╝ ╚════██║██╔══╝  ██║   ██║██║  ██║██║   ██║██╔══╝  ██║╚██╗██║██║   ██║██║██║╚██╗██║██╔══╝  
██║     ███████║███████╗╚██████╔╝██████╔╝╚██████╔╝███████╗██║ ╚████║╚██████╔╝██║██║ ╚████║███████╗
╚═╝     ╚══════╝╚══════╝ ╚═════╝ ╚═════╝  ╚═════╝ ╚══════╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝╚═╝  ╚═══╝╚══════╝

An interpreter for the A-Level pseudocode syntax.
Created by Jaxon Moh & Jin Wei Tan."#;

thread_local! {
    pub static SOURCE_FILE: Rc<RefCell<SourceFile>> = Rc::new(RefCell::new(SourceFile::new()));
}

fn main() {
    let cli = Command::new("pseudoengine")
        .about(HEADER)
        .version("0.0.1")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run the program.")
                .arg(Arg::new("file").help("Filepath of the program")),
        )
        .get_matches();

    if let Some(command) = cli.subcommand_name() {
        if let Some(args) = cli.subcommand_matches(command) {
            let file: &String = args
                .get_one("file")
                .expect("File name not provided");
            match command {
                "run" => execute(file),
                _ => unreachable!(),
            };
        }
    }
}

fn execute(filepath: &str) {
    let mut buf = read_to_string(filepath).expect(format!("File {} not found", filepath).as_str());
    let lines = buf
        .clone()
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<String>>();
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
    #[test]
    fn func_test() {
        execute("tests/func_test.txt");
    }
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

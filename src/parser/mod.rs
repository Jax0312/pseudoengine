use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_class::parse_class;
use crate::parser::parse_declare::parse_declare;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_func::{parse_function, parse_procedure, parse_return};
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::parse_io::*;
use crate::parser::parse_loop::{parse_for, parse_repeat, parse_while};
use crate::parser::parse_selection::{parse_if};
use crate::tokens::TToken;
use crate::utils::err;

mod parse_declare;
pub mod parse_expr;
mod parse_identifier;
mod parse_loop;
mod parse_io;
mod parse_func;
mod parse_selection;
mod parse_class;

pub fn parse_file(lexer: &mut Lexer) -> Vec<Box<Node>> {
    let mut nodes = Vec::new();
    let mut main_children = Vec::<Box<Node>>::new();

    while lexer.peek().is_some() {
        match lexer.peek().unwrap().t {
            TToken::Procedure => main_children.push(parse_procedure(lexer)),
            TToken::Function => main_children.push(parse_function(lexer)),
            TToken::Class => main_children.push(parse_class(lexer)),
            _ => main_children.push(parse_line(lexer)),
        }
    }

    nodes.push(Box::from(Node::Main {
        children: main_children,
    }));

    println!("\nAST\n{:?}", nodes);

    nodes
}

pub fn parse_line(lexer: &mut Lexer) -> Box<Node> {
    match lexer.peek().unwrap().t {
        TToken::Declare => parse_declare(lexer),
        TToken::Identifier(_) => parse_assign(lexer),
        TToken::While => parse_while(lexer),
        TToken::For => parse_for(lexer),
        TToken::Repeat => parse_repeat(lexer),
        TToken::Input => parse_input(lexer),
        TToken::Output => parse_output(lexer),
        TToken::OpenFile => parse_open_file(lexer),
        TToken::CloseFile => parse_close_file(lexer),
        TToken::ReadFile => parse_read_file(lexer),
        TToken::WriteFile => parse_write_file(lexer),
        TToken::Seek => parse_seek_file(lexer),
        TToken::GetRecord => parse_get_record(lexer),
        TToken::PutRecord => parse_put_record(lexer),
        TToken::If => parse_if(lexer),
        // TToken::Case => parse_case(lexer),
        TToken::Return => parse_return(lexer),
        TToken::Newline | TToken::EOF => {
            lexer.next();
            Box::new(Node::Null)
        },
        TToken::Procedure | TToken::Function => err("Procedure and Function can only be declared in the global scope", &lexer.peek().unwrap().pos),
        TToken::Class => err("Class can only be declared in the global scope", &lexer.peek().unwrap().pos),
        _ => err("Invalid syntax", &lexer.peek().unwrap().pos),
    }
}

fn parse_assign(lexer: &mut Lexer) -> Box<Node> {

    let lhs = parse_identifier(lexer);
    match lexer.next().unwrap() {
        Token { t: TToken::Assignment, pos: _ } => {
            Box::from(Node::Assignment {
                lhs,
                rhs: parse_expression(lexer, &[]).0,
            })
        },
        _ => {
            // Skip the rest of the expression as it is meaningless without assignment
            while let Some(t) = lexer.next() {
                if t.t == TToken::Newline {
                    break;
                }
            }
            Box::from(Node::Null)
        },
    }

}

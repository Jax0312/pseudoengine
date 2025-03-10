use std::ops::Deref;
use std::vec;

use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_class::parse_class;
use crate::parser::parse_declare::{parse_constant, parse_declare, parse_user_defined_data};
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_func::{parse_function, parse_procedure, parse_return};
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::parse_io::*;
use crate::parser::parse_loop::{parse_for, parse_repeat, parse_while};
use crate::parser::parse_selection::{parse_case, parse_if};
use crate::tokens::TToken;
use crate::utils::err;

mod parse_class;
mod parse_declare;
pub mod parse_expr;
mod parse_func;
mod parse_identifier;
mod parse_io;
mod parse_loop;
mod parse_selection;

pub fn parse_file(lexer: &mut Lexer) -> Vec<Box<Node>> {
    let mut nodes = Vec::new();
    let mut main_children = Vec::<Box<Node>>::new();

    while lexer.peek().is_some() {
        match lexer.peek().unwrap().t {
            TToken::Procedure => main_children.push(parse_procedure(lexer, false)),
            TToken::Function => main_children.push(parse_function(lexer, false)),
            TToken::Class => main_children.push(parse_class(lexer)),
            _ => main_children.push(parse_line(lexer)),
        }
    }

    nodes.push(Box::from(Node::Main {
        children: main_children,
    }));

    nodes
}

pub fn parse_line(lexer: &mut Lexer) -> Box<Node> {
    let token = lexer.peek().unwrap();
    match token.t {
        TToken::Declare => parse_declare(lexer),
        TToken::Constant => parse_constant(lexer),
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
        TToken::Type => parse_user_defined_data(lexer),
        TToken::Case => parse_case(lexer),
        TToken::Return => parse_return(lexer),
        TToken::Identifier(_) => {
            let lhs = parse_expression(lexer);
            try_parse_assign(lexer, lhs)
        }
        TToken::Call => parse_call(lexer),
        TToken::Newline | TToken::EOF => {
            lexer.next();
            Box::new(Node::Null)
        }
        TToken::Procedure | TToken::Function => err(
            "Procedure and Function can only be declared in the global scope",
            &lexer.peek().unwrap().pos,
        ),
        TToken::Class => err(
            "Class can only be declared in the global scope",
            &lexer.peek().unwrap().pos,
        ),
        _ => err("Expected statement", &token.pos),
    }
}

fn parse_call(lexer: &mut Lexer) -> Box<Node> {
    lexer.next();
    let func_call = parse_identifier(lexer);
    if let Node::FunctionCall { .. } = func_call.deref() {
        func_call
    } else {
        err("PROCEDURE expected", &lexer.peek().unwrap().pos);
    }
}

fn try_parse_assign(lexer: &mut Lexer, lhs: Box<Node>) -> Box<Node> {
    match lexer.next().unwrap().t {
        TToken::Assignment => {
            let rhs = parse_expression(lexer);
            let pos = Position::range(lhs.pos(), rhs.pos());
            Box::from(Node::Assignment { lhs, rhs, pos })
        }
        _ => lhs,
    }
}

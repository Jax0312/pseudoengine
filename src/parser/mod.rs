use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_declare::parse_declare;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::parse_io::{parse_close_file, parse_get_record, parse_input, parse_open_file, parse_output, parse_put_record, parse_read_file, parse_seek_file, parse_write_file};
use crate::parser::parse_loop::{parse_for, parse_while};
use crate::tokens::TToken;

mod parse_declare;
pub mod parse_expr;
mod parse_identifier;
mod parse_loop;
mod parse_io;

pub fn parse_file(lexer: &mut Lexer) -> Vec<Box<Node>> {
    let mut nodes = Vec::new();
    let mut main_children = Vec::<Box<Node>>::new();

    while lexer.peek().is_some() {
        match lexer.peek().unwrap().t {
            TToken::Procedure => unimplemented!(),
            TToken::Function => unimplemented!(),
            TToken::Class => unimplemented!(),
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
        TToken::Input => parse_input(lexer),
        TToken::Output => parse_output(lexer),
        TToken::OpenFile => parse_open_file(lexer),
        TToken::CloseFile => parse_close_file(lexer),
        TToken::ReadFile => parse_read_file(lexer),
        TToken::WriteFile => parse_write_file(lexer),
        TToken::Seek => parse_seek_file(lexer),
        TToken::GetRecord => parse_get_record(lexer),
        TToken::PutRecord => parse_put_record(lexer),
        _ => {
            lexer.next();
            Box::new(Node::Null)
        }
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

use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_declare::parse_declare;
use crate::parser::parse_expr::parse_expression;
use crate::enums::Node::{Assignment, Var};
use crate::tokens::TToken;

mod parse_declare;
pub mod parse_expr;
mod parse_identifier;

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
        _ => {
            lexer.next();
            Box::new(Node::Null)
        }
    }
}

fn parse_assign(lexer: &mut Lexer) -> Box<Node> {
    let Token {t: TToken::Identifier(name), pos} = lexer.next().unwrap() else { unreachable!() };
    let lhs = Box::from(Var { name, pos });
    
    match lexer.next().unwrap() {
        Token { t: TToken::Assignment, pos: _ } => {
            Box::from(Assignment {
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

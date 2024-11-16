use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_input(lexer: &mut Lexer) -> Box<Node> {
    // skip INPUT token
    lexer.next();
    let Token {t: TToken::Identifier(name), pos} = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier").unwrap() else { unreachable!()};
    Box::from(Node::Input {
        child: Box::new(Node::Var {
            name, pos
        }),
    })
}

pub fn parse_output(lexer: &mut Lexer) -> Box<Node> {
    let mut children = Vec::new();
    
    // skip OUTPUT token
    lexer.next();
    loop {
        let (exp, stop_token) = parse_expression(lexer, &[TToken::Comma]);
        children.push(exp);
        if stop_token.is_none() {
            break;
        }
    }

    Box::from(Node::Output {
        children
    })
}
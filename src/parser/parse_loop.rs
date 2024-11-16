use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::err;

pub fn parse_while(lexer: &mut Lexer) -> Box<Node> {
    // Skip while token
    lexer.next();
    
    let mut body = Vec::new();
    let cond = parse_expression(lexer, &[]).0;
    
    loop {
        match lexer.peek() {
            Some(Token {t: TToken::EOF, pos}) => err("'ENDWHILE' expected", pos),
            Some(Token { t: TToken::EndWhile, pos}) => break, 
            _ => body.push(parse_line(lexer))
        }
    }
    Box::from(Node::While {
        cond,
        body,
    })
}
use std::fmt::format;
use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_while(lexer: &mut Lexer) -> Box<Node> {
    // Skip while token
    lexer.next();
    
    let mut body = Vec::new();
    let cond = parse_expression(lexer, &[]).0;
    
    loop {
        match lexer.peek() {
            Some(Token {t: TToken::EOF, pos}) => err("'ENDWHILE' expected", pos),
            Some(Token { t: TToken::EndWhile, pos}) => {
                lexer.next();
                break;
            }, 
            _ => body.push(parse_line(lexer))
        }
    }
    Box::from(Node::While {
        cond,
        body,
    })
}

pub fn parse_for(lexer: &mut Lexer) -> Box<Node> {
    // skip for token
    lexer.next();
    let iter;
    
    if let Some(Token {t: TToken::Identifier(name), pos}) = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier") {
        iter = Box::from(Node::Var {name, pos})
    } else { unreachable!()}
    expect_token(lexer, &[TToken::Assignment], "<- expected");
    
    let start = parse_expression(lexer, &[TToken::To]).0;
    let (end, end_token) = parse_expression(lexer, &[TToken::Step]);
    
    let mut body = Vec::new();
    let step;
    if let Some(TToken::Step) =  end_token {
        step = parse_expression(lexer, &[]).0;
    } else {
        step = Box::from(Node::Null)
    }

    loop {
        match lexer.peek() {
            Some(Token {t: TToken::EOF, pos}) => err("'NEXT' expected", pos),
            Some(Token { t: TToken::Next, pos}) => {
                lexer.next();
                break;
            },
            _ => body.push(parse_line(lexer))
        }
    }
    
    // Check for optional appearance of iter variable
    if let Some(Token {t: TToken::Identifier(name), pos}) = lexer.peek() {
        match *iter {
            Node::Var { name: ref _name, .. } => {
                // Identifier must match the earlier specified one
                if *name != *_name {
                    err(format!("Identifier must be '{}' ", _name).as_str(), pos);
                }
                lexer.next();
            },
            _ => unreachable!()
        }   
    }
    
    Box::from(Node::For {
        iter,
        range: Box::new(Node::Range {start, end}),
        step,
        body
    })
}
use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_while(lexer: &mut Lexer) -> Box<Node> {
    // Skip while token
    let token = lexer.next().unwrap();

    let mut body = Vec::new();
    let cond = parse_expression(lexer);

    let end = loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("'ENDWHILE' expected", pos),
            Some(Token {
                t: TToken::EndWhile,
                pos,
            }) => {
                let pos = pos.clone();
                lexer.next();
                break pos;
            }
            _ => body.push(parse_line(lexer)),
        }
    };
    let pos = Position::range(token.pos, end);
    Box::from(Node::While { cond, body, pos })
}

pub fn parse_repeat(lexer: &mut Lexer) -> Box<Node> {
    // skip REPEAT token
    let token = lexer.next().unwrap();

    let mut body = vec![];
    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("'UNTIL' expected", pos),
            Some(Token {
                t: TToken::Until,
                pos,
            }) => {
                lexer.next();
                break;
            }
            _ => body.push(parse_line(lexer)),
        }
    }

    let cond = parse_expression(lexer);
    let pos = Position::range(token.pos, cond.pos());
    Box::from(Node::Repeat { cond, body, pos })
}

pub fn parse_for(lexer: &mut Lexer) -> Box<Node> {
    // skip for token
    let token = lexer.next().unwrap();
    let iter;

    if let Token {
        t: TToken::Identifier(name),
        pos,
    } = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier")
    {
        iter = Box::from(Node::Var { name, pos })
    } else {
        unreachable!()
    }
    expect_token(lexer, &[TToken::Assignment], "<-");

    let start = parse_expression(lexer);
    expect_token(lexer, &[TToken::To], "'TO'");
    let end = parse_expression(lexer);

    let mut body = Vec::new();
    let step;
    if lexer.peek().unwrap().t == TToken::Step {
        lexer.next();
        step = parse_expression(lexer);
    } else {
        step = Box::from(Node::Null)
    }

    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("'NEXT' expected", pos),
            Some(Token {
                t: TToken::Next,
                pos: _,
            }) => {
                lexer.next();
                break;
            }
            _ => body.push(parse_line(lexer)),
        }
    }

    // Check for optional appearance of iter variable
    let ident = lexer.peek().unwrap().clone();
    if let Token {
        t: TToken::Identifier(name),
        pos,
    } = ident
    {
        match *iter {
            Node::Var {
                name: ref _name, ..
            } => {
                // Identifier must match the earlier specified one
                if *name != *_name {
                    err(format!("Identifier must be '{}' ", _name).as_str(), &pos);
                }
                lexer.next();
            }
            _ => unreachable!(),
        }
    } else {
        err("Identifier expected", &ident.pos)
    };
    let pos = Position::range(start.pos(), end.pos());
    let range = Box::new(Node::Range { start, end, pos });

    let pos = Position::range(token.pos, ident.pos);
    Box::from(Node::For {
        iter,
        range,
        step,
        body,
        pos,
    })
}

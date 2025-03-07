use crate::enums::{Node, Position, Token};
use crate::lexer::{self, Lexer};
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::{parse_line, try_parse_assign};
use crate::tokens::TToken;
use crate::utils::{err, expect_token};
use std::ops::Deref;

pub fn parse_if(lexer: &mut Lexer) -> Box<Node> {
    // skip IF token
    let token = lexer.next().unwrap();
    let cond = parse_expression(lexer);
    expect_token(lexer, &[TToken::Then], "'THEN'");

    let mut true_body = vec![];
    let mut else_encountered = false;
    let mut false_body = vec![];

    let end = loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("'ENDIF' expected", pos),
            Some(Token {
                t: TToken::EndIf,
                pos,
            }) => {
                let pos = pos.clone();
                lexer.next();
                break pos;
            }
            Some(Token {
                t: TToken::Else,
                pos: _,
            }) => {
                else_encountered = true;
                lexer.next();
            }
            _ => {
                if else_encountered {
                    false_body.push(parse_line(lexer))
                } else {
                    true_body.push(parse_line(lexer))
                }
            }
        }
    };
    let pos = Position::range(token.pos, end.clone());
    Box::from(Node::If {
        cond,
        true_body,
        false_body,
        pos,
    })
}

pub fn parse_case(lexer: &mut Lexer) -> Box<Node> {
    // skip CASE token
    let token = lexer.next().unwrap();
    expect_token(lexer, &[TToken::Of], "'Of'");
    let cmp = parse_identifier(lexer);
    // skip NEWLINE token
    lexer.next();
    let mut cases = vec![];
    let mut otherwise = vec![];

    let end = loop {
        let token = lexer.peek().unwrap().clone();
        match token.t {
            TToken::EndCase => break token.pos,
            TToken::EOF => err("ENDCASE expected", &token.pos),
            TToken::Otherwise => {
                lexer.next();
                expect_token(lexer, &[TToken::Colon], "':'");
                loop {
                    match lexer.peek().unwrap() {
                        Token {
                            t: TToken::EndCase, ..
                        } => break,
                        Token { t: TToken::EOF, .. } => break,
                        _ => otherwise.push(parse_line(lexer)),
                    }
                }
            }
            _ => {
                let mut children = Vec::new();
                let mut start = Box::new(parse_literal(lexer));
                if lexer.peek().unwrap().t == TToken::To {
                    lexer.next();
                    let end = Box::new(parse_literal(lexer));
                    let pos = Position::range(token.pos, end.pos());
                    start = Box::new(Node::Range { start, end, pos });
                }
                expect_token(lexer, &[TToken::Colon], "':'");
                loop {
                    if try_parse_literal(lexer).is_some() {
                        break;
                    }
                    match lexer.peek().unwrap().t {
                        TToken::EndCase | TToken::Otherwise => break,
                        _ => children.push(parse_line(lexer)),
                    }
                }
                while Node::Null == **children.last().unwrap() {
                    children.pop();
                }
                let pos = Position::range(token.pos, children.last().unwrap().pos());
                cases.push(Box::from(Node::Case {
                    expr: start,
                    children,
                    pos,
                }));
            }
        }
    };
    lexer.next();

    let pos = Position::range(token.pos, end);
    Box::from(Node::Switch {
        cmp,
        cases,
        otherwise,
        pos,
    })
}

fn parse_literal(lexer: &mut Lexer) -> Node {
    let token = lexer.peek().unwrap().clone();
    if let Some(literal) = try_parse_literal(lexer) {
        lexer.next();
        return literal;
    }
    err("Expected literal", &token.pos);
}

fn try_parse_literal(lexer: &mut Lexer) -> Option<Node> {
    let literal = lexer.peek().unwrap();
    let pos = literal.pos;
    match literal.t.clone() {
        TToken::StringLit(val) => Some(Node::String { val, pos }),
        TToken::IntegerLit(val) => Some(Node::Int { val, pos }),
        TToken::RealLit(val) => Some(Node::Real { val, pos }),
        TToken::BoolLit(val) => Some(Node::Boolean { val, pos }),
        TToken::DateLit(val) => Some(Node::Date { val, pos }),
        _ => None,
    }
}

use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::{parse_line, try_parse_assign};
use crate::tokens::TToken;
use crate::utils::{err, expect_token};
use std::ops::Deref;

pub fn parse_if(lexer: &mut Lexer) -> Box<Node> {
    // skip IF token
    lexer.next();
    let (cond, stop_token) = parse_expression(lexer, &[TToken::Then]);
    // Then can be on the next line
    if stop_token.t != TToken::Then {
        expect_token(lexer, &[TToken::Then], "'THEN'");
    }

    let mut true_body = vec![];
    let mut else_encountered = false;
    let mut false_body = vec![];

    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("'ENDIF' expected", pos),
            Some(Token {
                t: TToken::EndIf,
                pos: _,
            }) => {
                lexer.next();
                break;
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
    }

    Box::from(Node::If {
        cond,
        true_body,
        false_body,
    })
}

pub fn parse_case(lexer: &mut Lexer) -> Box<Node> {
    // skip CASE token
    lexer.next();
    expect_token(lexer, &[TToken::Of], "'Of'");
    let cmp = parse_identifier(lexer);
    let mut cases = vec![];
    // skip NEWLINE token
    lexer.next();

    let mut temp = vec![];
    let mut is_range = false;
    let mut expr = Node::Null;
    let mut statements = vec![];
    let mut otherwise = vec![];

    let token_to_lit = |token: &Token| -> Node {
        match &token.t {
            TToken::StringLit(val) => Node::String {
                val: val.clone(),
                pos: Position::invalid(),
            },
            TToken::IntegerLit(val) => Node::Int {
                val: val.clone(),
                pos: Position::invalid(),
            },
            TToken::RealLit(val) => Node::Real {
                val: val.clone(),
                pos: Position::invalid(),
            },
            TToken::BoolLit(val) => Node::Boolean {
                val: val.clone(),
                pos: Position::invalid(),
            },
            _ => err("Invalid value", &token.pos),
        }
    };

    loop {
        let token = lexer.next().unwrap();
        match token {
            Token { t: TToken::To, .. } => is_range = true,
            Token {
                t: TToken::EndCase, ..
            } => break,
            Token {
                t: TToken::EOF,
                pos,
            } => err("ENDCASE expected", &pos),
            Token {
                t: TToken::Otherwise,
                ..
            } => {
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
            Token {
                t: TToken::Newline,
                pos,
            } => {
                temp.push(token);
                statements.push(parse_line(&mut temp.clone().into_iter().peekable()));
                temp.clear();
            }
            Token {
                t: TToken::Colon,
                pos,
            } => {
                cases.push(Box::from(Node::Case {
                    expr: Box::from(expr),
                    children: statements.clone(),
                }));
                expr = match temp.len() {
                    1 => token_to_lit(temp.first().unwrap()),
                    2 => Node::Range {
                        start: Box::from(token_to_lit(&temp[0])),
                        end: Box::from(token_to_lit(&temp[1])),
                    },
                    _ => err("Invalid expression", &pos),
                };
                temp.clear();
                statements.clear();
            }
            _ => temp.push(token),
        }
    }

    cases.push(Box::from(Node::Case {
        expr: Box::from(expr),
        children: statements.clone(),
    }));

    // first case is null node
    cases.remove(0);

    Box::from(Node::Switch {
        cmp: Box::from(Node::Expression(vec![cmp])),
        cases,
        otherwise,
    })
}

use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_declare::parse_declaration;
use crate::parser::parse_func::{parse_function, parse_procedure};
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_class(lexer: &mut Lexer) -> Box<Node> {
    // skip CLASS token
    let token = lexer.next().unwrap();
    let Token {
        t: TToken::Identifier(name),
        pos: ident_pos,
    } = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier")
    else {
        unreachable!()
    };
    let mut base = Box::from(Node::Null);
    let mut children = vec![];
    // Check for inheritance pattern
    if let Some(Token {
        t: TToken::Inherits,
        pos: _,
    }) = lexer.peek()
    {
        lexer.next();
        let Token {
            t: TToken::Identifier(val),
            pos,
        } = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier")
        else {
            unreachable!()
        };
        base = Box::from(Node::String { val, pos })
    }

    let end = loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("ENDCLASS expected", &pos),
            Some(Token {
                t: TToken::EndClass,
                pos,
            }) => {
                let pos = pos.clone();
                lexer.next();
                break pos;
            }
            Some(Token {
                t: TToken::Private,
                pos: _,
            }) => {
                lexer.next();
                children.push(match lexer.peek() {
                    Some(Token {
                        t: TToken::Procedure,
                        ..
                    }) => Box::from(parse_procedure(lexer, true)),
                    Some(Token {
                        t: TToken::Function,
                        ..
                    }) => Box::from(parse_function(lexer, true)),
                    _ => Box::from(parse_declaration(lexer, false, true)),
                });
            }
            Some(Token {
                t: TToken::Public,
                pos: _,
            }) => {
                lexer.next();
                children.push(match lexer.peek() {
                    Some(Token {
                        t: TToken::Procedure,
                        ..
                    }) => parse_procedure(lexer, false),
                    Some(Token {
                        t: TToken::Function,
                        ..
                    }) => parse_function(lexer, false),
                    _ => parse_declaration(lexer, false, false),
                });
            }
            _ => children.push(parse_line(lexer)),
        }
    };
    let pos = Position::range(token.pos, end);
    Box::from(Node::Class {
        name: Box::from(Node::String {
            val: name,
            pos: ident_pos,
        }),
        base,
        children,
        pos,
    })
}

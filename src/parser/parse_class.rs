use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_declare::parse_declaration;
use crate::parser::parse_func::{parse_function, parse_procedure};
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_class(lexer: &mut Lexer) -> Box<Node> {
    // skip CLASS token
    lexer.next();
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

    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => err("ENDCLASS expected", &pos),
            Some(Token {
                t: TToken::EndClass,
                pos: _,
            }) => {
                lexer.next();
                break;
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
                    }) => Box::from(Node::Private(parse_procedure(lexer))),
                    Some(Token {
                        t: TToken::Function,
                        ..
                    }) => Box::from(Node::Private(parse_function(lexer))),
                    _ => Box::from(Node::Private(parse_declaration(lexer))),
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
                    }) => parse_procedure(lexer),
                    Some(Token {
                        t: TToken::Function,
                        ..
                    }) => parse_function(lexer),
                    _ => parse_declaration(lexer),
                });
            }
            _ => children.push(parse_line(lexer)),
        }
    }

    Box::from(Node::Class {
        name: Box::from(Node::String {
            val: name,
            pos: ident_pos,
        }),
        base,
        children,
    })
}

use crate::enums::{Node, Token, VariableType};
use crate::lexer::Lexer;
use crate::parser::parse_declare::{parse_array, parse_declaration};
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_function(lexer: &mut Lexer) -> Box<Node> {
    // skip FUNCTION token
    lexer.next();
    let name = match expect_token(
        lexer,
        &[TToken::Identifier("".to_string()), TToken::New],
        "Function name",
    ) {
        Token {
            t: TToken::Identifier(val),
            pos,
        } => Box::from(Node::String { val, pos }),
        Token {
            t: TToken::New,
            pos,
        } => Box::from(Node::String {
            val: "new".to_string(),
            pos,
        }),
        _ => unreachable!(),
    };
    let params = parse_params(lexer);
    expect_token(lexer, &[TToken::ReturnType], "'RETURNS' expected");

    let return_type = match expect_token(
        lexer,
        &[
            TToken::VarType(VariableType::Integer),
            TToken::Identifier("".to_string()),
            TToken::Array,
        ],
        "TYPE expected",
    )
    .t
    {
        TToken::VarType(vt) => Box::from(vt),
        TToken::Identifier(vt) => Box::from(VariableType::Composite(vt)),
        TToken::Array => parse_array(lexer),
        _ => unreachable!(),
    };

    let mut children = vec![];
    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EndFunction,
                pos: _,
            }) => {
                lexer.next();
                break;
            }
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => {
                err("ENDFUNCTION expected", pos);
            }
            _ => children.push(parse_line(lexer)),
        }
    }

    Box::from(Node::Function {
        name,
        params,
        return_type,
        children,
    })
}

pub fn parse_procedure(lexer: &mut Lexer) -> Box<Node> {
    // skip PROCEDURE token
    lexer.next();
    let name = match expect_token(
        lexer,
        &[TToken::Identifier("".to_string()), TToken::New],
        "Procedure name",
    ) {
        Token {
            t: TToken::Identifier(val),
            pos,
        } => Box::from(Node::String { val, pos }),
        Token {
            t: TToken::New,
            pos,
        } => Box::from(Node::String {
            val: "new".to_string(),
            pos,
        }),
        _ => unreachable!(),
    };

    let params = parse_params(lexer);

    let mut children = vec![];
    loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EndProcedure,
                pos: _,
            }) => {
                lexer.next();
                break;
            }
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => {
                err("ENDPROCEDURE expected", pos);
            }
            _ => children.push(parse_line(lexer)),
        }
    }

    Box::from(Node::Procedure {
        name,
        params,
        children,
    })
}

fn parse_params(lexer: &mut Lexer) -> Vec<Box<Node>> {
    let mut params = vec![];
    expect_token(lexer, &[TToken::LParen], "'('");
    loop {
        match lexer.peek() {
            Some(&Token {
                t: TToken::RParen,
                pos: _,
            }) => {
                lexer.next();
                break;
            }
            Some(&Token {
                t: TToken::PassBy(ref by),
                pos: _,
            }) => {
                let by = by.clone();
                lexer.next();
                if by == "BYREF" {
                    params.push(Box::from(Node::Reference(parse_declaration(lexer))));
                } else {
                    params.push(parse_declaration(lexer));
                }
            }
            Some(&Token {
                t: TToken::Comma,
                pos: _,
            }) => {
                lexer.next();
            }
            _ => params.push(parse_declaration(lexer)),
        }
    }

    params
}

pub fn parse_return(lexer: &mut Lexer) -> Box<Node> {
    lexer.next();
    Box::from(Node::Return(parse_expression(lexer, &[]).0))
}

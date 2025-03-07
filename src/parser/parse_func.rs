use crate::enums::{Node, Position, Token, VariableType};
use crate::lexer::Lexer;
use crate::parser::parse_declare::{parse_array, parse_declaration};
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_function(lexer: &mut Lexer, private: bool) -> Box<Node> {
    // skip FUNCTION token
    let token = lexer.next().unwrap();
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
        TToken::Identifier(vt) => Box::from(VariableType::Custom(vt)),
        TToken::Array => parse_array(lexer),
        _ => unreachable!(),
    };

    let mut children = vec![];
    let end = loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EndFunction,
                pos,
            }) => {
                let pos = pos.clone();
                lexer.next();
                break pos;
            }
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => {
                err("ENDFUNCTION expected", pos);
            }
            _ => children.push(parse_line(lexer)),
        }
    };

    let pos = Position::range(token.pos, end);
    Box::from(Node::Function {
        name,
        params,
        return_type,
        children,
        pos,
        private
    })
}

pub fn parse_procedure(lexer: &mut Lexer, private: bool) -> Box<Node> {
    // skip PROCEDURE token
    let token = lexer.next().unwrap();
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
    let end = loop {
        match lexer.peek() {
            Some(Token {
                t: TToken::EndProcedure,
                pos,
            }) => {
                let pos = pos.clone();
                lexer.next();
                break pos;
            }
            Some(Token {
                t: TToken::EOF,
                pos,
            }) => {
                err("ENDPROCEDURE expected", pos);
            }
            _ => children.push(parse_line(lexer)),
        }
    };

    let pos = Position::range(token.pos, end);
    Box::from(Node::Procedure {
        name,
        params,
        children,
        pos,
        private
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
                params.push(parse_declaration(lexer, by == "BYREF", false));
            }
            Some(&Token {
                t: TToken::Comma,
                pos: _,
            }) => {
                lexer.next();
            }
            _ => params.push(parse_declaration(lexer, false, false)),
        }
    }

    params
}

pub fn parse_return(lexer: &mut Lexer) -> Box<Node> {
    let token = lexer.next().unwrap();
    let expr = parse_expression(lexer);
    let pos = Position::range(token.pos, expr.pos());
    Box::from(Node::Return { expr, pos })
}

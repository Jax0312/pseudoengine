use crate::enums::{Array, Node, Token, VariableType};
use crate::lexer::Lexer;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_declare(lexer: &mut Lexer) -> Box<Node> {
    // Skip Declare token
    lexer.next();
    parse_declaration(lexer)
}

// Actual parsing of declaration content
pub fn parse_declaration(lexer: &mut Lexer) -> Box<Node> {

    let mut expect_ident = false;
    let mut vars = Vec::<String>::new();
    let mut current;
    
    // Handle one or more identifier

    loop {
        current = lexer.next();
        match current.clone().unwrap() {
            Token {t: TToken::Identifier(ident), pos: _ } => {
                vars.push(ident);
                expect_ident = false;
            }
            Token {t: TToken::Comma, pos } => {
                if expect_ident {
                    err("Identifier expected", &pos);
                }
                expect_ident = true;
            }
            _ => break,
        }
    }
    let current = current.unwrap();

    if expect_ident {
        err("Identifier expected", &current.pos);
    } else if current.t != TToken::Colon {
        err(": expected", &current.pos);
    }

    // Handle variable type
    match lexer.next().unwrap() {
        Token { t: TToken::VarType(vt), pos: _ } => Box::new(Node::Declare {
            t: Box::from(vt),
            children: vars,
        }),
        Token { t: TToken::Array, pos: _ }=> Box::new(Node::Declare {
            t: parse_array(lexer),
            children: vars,
        }),
        Token { t: _, pos} => err("Type expected", &pos),
    }
}

pub fn parse_array(lexer: &mut Lexer) -> Box<VariableType> {

    expect_token(lexer, &[TToken::LSqrBracket], "[");

    parse_array_dimension(lexer)
}

fn parse_array_dimension(lexer: &mut Lexer) -> Box<VariableType> {

    // The function calls itself recursively to create a nested array structure

    let mut v= Box::new(Array{t: Box::from(VariableType::Integer), lower: 0, upper: 0});

    if let TToken::IntegerLit(val) = expect_token(lexer, &[TToken::IntegerLit(0)], "Integer").t { v.lower = val}

    expect_token(lexer, &[TToken::Colon], ":");

    if let TToken::IntegerLit(val) = expect_token(lexer, &[TToken::IntegerLit(0)], "Integer").t { v.upper = val}

    match lexer.next().unwrap() {
        Token {t: TToken::Comma, pos: _} => {
            Box::from(VariableType::Array( Box::from(Array {
                t: parse_array_dimension(lexer),
                lower: v.lower,
                upper: v.upper,
            })))
        }
        Token {t: TToken::RSqrBracket, pos} => {
            expect_token(lexer, &[TToken::Of], "'Of'");
            if let Some(Token {t: TToken::VarType(vt), pos: _}) = lexer.next() {
                Box::from(VariableType::Array( Box::from(Array {
                    t: Box::from(vt),
                    lower: v.lower,
                    upper: v.upper,
                })))
            } else {
                err("Type expected", &pos);
            }
        },
        Token {t: _, pos} => {err(" ']' or ',' expected", &pos)}
    }
}

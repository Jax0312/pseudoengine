use crate::enums::{Array, Node, Position, Token, VariableType};
use crate::lexer::Lexer;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_user_defined_data(lexer: &mut Lexer) -> Box<Node> {
    // Skip Type token
    lexer.next();
    let name;
    if let TToken::Identifier(_name) = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier").t {name = _name;} else {unreachable!()}
    // clear newline
    let next_token = expect_token(lexer, &[TToken::Newline, TToken::Operator("=".to_string())], "DECLARE or '='");
    match next_token.t  {
        TToken::Newline => {
            // This is a record data type pattern
            expect_token(lexer, &[TToken::Declare], "DECLARE");
            let mut fields = Vec::new();
            loop {
                fields.push(parse_declaration(lexer));
                // parse_declaration does not consume the trailing newline token
                if lexer.peek().unwrap().t == TToken::Newline {
                    lexer.next();   
                }
                match expect_token(lexer, &[TToken::Declare, TToken::EndType], "EndType").t {
                    TToken::EndType => break, 
                    TToken::Declare => continue,
                    _ => unreachable!(),
                }
            }
            Box::from(Node::Record {name, children: fields})
        },
        TToken::Operator(op) => {
            if op != "=" {
                err("Expected '='", &next_token.pos)
            }
            match expect_token(lexer, &[TToken::LParen, TToken::Caret], "'(' or '^'").t {
                TToken::LParen => parse_enum(lexer, name),
                TToken::Caret => {todo!()},
                _ => unreachable!(),
            }
        },
        _ => unreachable!()
    }
    
}

fn parse_enum(lexer: &mut Lexer, name: String) -> Box<Node> {
    let mut expect_ident = false;
    let mut variants = Vec::<String>::new();
    let mut current;
    
    loop {
        current = lexer.next();
        match current.clone().unwrap() {
            Token {t: TToken::Identifier(ident), pos: _ } => {
                variants.push(ident);
                expect_ident = false;
            }
            Token {t: TToken::Comma, pos } => {
                if expect_ident {
                    err("Enum value expected", &pos);
                }
                expect_ident = true;
            }
            _ => break,
        }
    }
    let current = current.unwrap();
    if expect_ident {
        err("Enum value expected", &current.pos);
    } else if current.t != TToken::RParen {
        err(") expected", &current.pos);
    }
    Box::from(Node::Enum { name, variants: variants.into_iter().map(|variant| Box::from(Node::String { val: variant, pos: Position::invalid() })).collect() })
}

pub fn parse_constant(lexer: &mut Lexer) -> Box<Node> {
    // Skip Constant token
    lexer.next();
    let name;
    let mut val= Box::from(Node::Null);
    if let TToken::Identifier(_name) = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier").t {
        name = _name
    } else {unreachable!()}
    if let Token{ t, pos } = expect_token(lexer, &[TToken::Operator("".to_string())], "'='") {
        if let TToken::Operator(_op) = t {
            if _op != "=" {
                err("'=' expected", &pos)
            }
        } else {unreachable!()}   
    }
    
    if let token = lexer.next().unwrap() {
        val = Box::from(match token.t {
            TToken::IntegerLit(val) => Node::Int{ val, pos: token.pos },
            TToken::RealLit(val) => Node::Real{ val, pos: token.pos },
            TToken::StringLit(val) => Node::String{ val, pos: token.pos },
            TToken::DateLit(val) => Node::Date{ val, pos: token.pos },
            TToken::BoolLit(val) => Node::Boolean{ val, pos: token.pos },
            _ => err("Literal value expected", &token.pos)    
        })
    }
    
    Box::from(Node::Const { name, val, pos: Position::invalid() })
}

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
        Token { t: TToken::Identifier(name), pos: _ }=> Box::new(Node::Declare {
            t: Box::from(VariableType::Custom(name)),
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

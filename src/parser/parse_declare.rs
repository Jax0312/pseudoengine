use crate::enums::{Array, Index, Node, Position, Token, VariableType};
use crate::lexer::Lexer;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_user_defined_data(lexer: &mut Lexer) -> Box<Node> {
    // Skip Type token
    let token = lexer.next().unwrap();
    let name_token = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier");
    let name = match name_token.t {
        TToken::Identifier(name) => name,
        _ => unreachable!(),
    };
    // clear newline
    let next_token = expect_token(
        lexer,
        &[TToken::Newline, TToken::Operator("=".to_string())],
        "DECLARE or '='",
    );
    match next_token.t {
        TToken::Newline => {
            // This is a record data type pattern
            expect_token(lexer, &[TToken::Declare], "DECLARE");
            let mut fields = Vec::new();
            let end = loop {
                fields.push(parse_declaration(lexer, false, false));
                // parse_declaration does not consume the trailing newline token
                if lexer.peek().unwrap().t == TToken::Newline {
                    lexer.next();
                }
                let end = expect_token(lexer, &[TToken::Declare, TToken::EndType], "EndType");
                match end.t {
                    TToken::EndType => break end.pos,
                    TToken::Declare => continue,
                    _ => unreachable!(),
                }
            };
            let pos = Position::range(token.pos, end);
            Box::from(Node::Record {
                name: Box::new(Node::String {
                    val: name,
                    pos: name_token.pos,
                }),
                children: fields,
                pos,
            })
        }
        TToken::Operator(op) => {
            if op != "=" {
                err("Expected '='", &next_token.pos)
            }
            match expect_token(lexer, &[TToken::LParen, TToken::Caret], "'(' or '^'").t {
                TToken::LParen => parse_enum(lexer, token, name),
                TToken::Caret => parse_pointer(lexer, token, &name),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

fn parse_pointer(lexer: &mut Lexer, start: Token, name: &String) -> Box<Node> {
    let vtype = expect_token(
        lexer,
        &[
            TToken::Identifier("".to_string()),
            TToken::VarType(VariableType::String),
        ],
        "Data type",
    );
    let ref_to = Box::from(match vtype.t {
        TToken::Identifier(name) => VariableType::Custom(name),
        TToken::VarType(vt) => vt,
        _ => unreachable!(),
    });
    let pos = Position::range(start.pos, vtype.pos);
    Box::from(Node::PointerDef {
        name: name.clone(),
        ref_to,
        pos,
    })
}

fn parse_enum(lexer: &mut Lexer, start: Token, name: String) -> Box<Node> {
    let mut expect_ident = false;
    let mut variants = Vec::<String>::new();
    let mut current;

    loop {
        current = lexer.next();
        match current.clone().unwrap() {
            Token {
                t: TToken::Identifier(ident),
                pos: _,
            } => {
                variants.push(ident);
                expect_ident = false;
            }
            Token {
                t: TToken::Comma,
                pos,
            } => {
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
    let pos = Position::range(start.pos, current.pos);
    Box::from(Node::Enum {
        name,
        variants: variants
            .into_iter()
            .map(|variant| {
                Box::from(Node::String {
                    val: variant,
                    pos: Position::invalid(),
                })
            })
            .collect(),
        pos,
    })
}

pub fn parse_constant(lexer: &mut Lexer) -> Box<Node> {
    // Skip Constant token
    lexer.next();
    let name;
    let mut val = Box::from(Node::Null);
    if let TToken::Identifier(_name) =
        expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier").t
    {
        name = _name
    } else {
        unreachable!()
    }
    if let Token { t, pos } = expect_token(lexer, &[TToken::Operator("".to_string())], "'='") {
        if let TToken::Operator(_op) = t {
            if _op != "=" {
                err("'=' expected", &pos)
            }
        } else {
            unreachable!()
        }
    }

    if let token = lexer.next().unwrap() {
        val = Box::from(match token.t {
            TToken::IntegerLit(val) => Node::Int {
                val,
                pos: token.pos,
            },
            TToken::RealLit(val) => Node::Real {
                val,
                pos: token.pos,
            },
            TToken::StringLit(val) => Node::String {
                val,
                pos: token.pos,
            },
            TToken::DateLit(val) => Node::Date {
                val,
                pos: token.pos,
            },
            TToken::BoolLit(val) => Node::Boolean {
                val,
                pos: token.pos,
            },
            _ => err("Literal value expected", &token.pos),
        })
    }

    Box::from(Node::Const {
        name,
        val,
        pos: Position::invalid(),
    })
}

pub fn parse_declare(lexer: &mut Lexer) -> Box<Node> {
    // Skip Declare token
    let token = lexer.next().unwrap();
    let mut node = parse_declaration(lexer, false, false);
    let new_pos = Position::range(token.pos, node.pos());
    if let Node::Declare { pos, .. } = &mut *node {
        *pos = new_pos;
    }
    node
}

// Actual parsing of declaration content
pub fn parse_declaration(lexer: &mut Lexer, byref: bool, private: bool) -> Box<Node> {
    let mut start = Position::invalid();
    let mut expect_ident = false;
    let mut vars = Vec::<String>::new();
    let mut current;

    // Handle one or more identifier

    loop {
        current = lexer.next();
        match current.clone().unwrap() {
            Token {
                t: TToken::Identifier(ident),
                pos,
            } => {
                if start == Position::invalid() {
                    start = pos;
                }
                vars.push(ident);
                expect_ident = false;
            }
            Token {
                t: TToken::Comma,
                pos,
            } => {
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
    let vtype = lexer.next().unwrap();
    let t = match vtype.t {
        TToken::VarType(vt) => Box::from(vt),
        TToken::Array => parse_array(lexer),
        TToken::Identifier(name) => Box::from(VariableType::Custom(name)),
        _ => err("Type expected", &vtype.pos),
    };

    let pos = Position::range(start, vtype.pos);
    Box::new(Node::Declare {
        t,
        byref,
        private,
        children: vars,
        pos,
    })
}

pub fn parse_array(lexer: &mut Lexer) -> Box<VariableType> {
    expect_token(lexer, &[TToken::LSqrBracket], "[");

    parse_array_dimension(lexer)
}

fn parse_array_dimension(lexer: &mut Lexer) -> Box<VariableType> {
    let mut shape = Vec::new();
    loop {
        let mut index = Index { lower: 0, upper: 0 };
        if let TToken::IntegerLit(val) = expect_token(lexer, &[TToken::IntegerLit(0)], "Integer").t
        {
            index.lower = val
        }
        expect_token(lexer, &[TToken::Colon], ":");
        if let TToken::IntegerLit(val) = expect_token(lexer, &[TToken::IntegerLit(0)], "Integer").t
        {
            index.upper = val
        }
        shape.push(index);

        let token = lexer.next().unwrap();
        match token.t {
            TToken::Comma => {}
            TToken::RSqrBracket => break,
            _ => err(" ']' or ',' expected", &token.pos),
        }
    }
    expect_token(lexer, &[TToken::Of], "'Of'");
    let token = lexer.next().unwrap();
    if let TToken::VarType(t) = token.t {
        Box::from(VariableType::Array {
            shape,
            t: Box::new(t),
        })
    } else {
        err("Type expected", &token.pos);
    }
}

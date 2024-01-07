use crate::lexer::*;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &mut Vec<Token>) -> Vec<Box<Node>> {
    let mut nodes = Vec::<Box<Node>>::new();

    let mut tokens = tokens.iter().peekable();

    while tokens.peek().is_some() {
        match &tokens.next().unwrap().t_type {
            TokenType::Declare => nodes.push(parse_declare(&mut tokens)),
            TokenType::Output => nodes.push(parse_output(&mut tokens)),
            TokenType::Input => nodes.push(parse_input(&mut tokens)),
            TokenType::Identifier(identifier) => {
                nodes.push(parse_assignment(&mut tokens, identifier.clone()))
            }
            _ => (),
        }
    }

    println!("{:#?}", nodes);

    nodes
}

fn parse_assignment(tokens: &mut Peekable<Iter<Token>>, identifier: String) -> Box<Node> {
    let token = tokens.next();
    if token.is_none() || token.unwrap().t_type != TokenType::Assignment {
        ()
    }

    // TODO
    // So currently the op pred parsing can only be triggered by an expression starting with num lit.
    // It should also trigger if the leading token of rhs is a identifier
    // More robust handling required

    Box::new(Node::Assignment {
        lhs: Box::new(Node::Var(identifier.clone())),
        rhs: Box::new(parse_expr(tokens)),
    })
}

fn parse_expr(tokens: &mut Peekable<Iter<Token>>) -> Node {
    let lhs = parse_primary(tokens);
    parse_expr_precedence(tokens, lhs, 0)
}

fn parse_expr_precedence(tokens: &mut Peekable<Iter<Token>>, mut lhs: Node, precedence: i8) -> Node {
    if let Some(token) = tokens.peek() {
        let mut lookahead = (*token).clone();
        while tokens.peek().is_some() && op_precedence(&lookahead.t_type) >= precedence {
            let op = lookahead;
            if tokens.next().is_some() {
                let mut rhs = parse_primary(tokens);
                if let Some(token) = tokens.peek() {
                    lookahead = (*token).clone();
                    while tokens.peek().is_some() && op_precedence(&lookahead.t_type) > op_precedence(&op.t_type) {
                        rhs = parse_expr_precedence(tokens, rhs, op_precedence(&lookahead.t_type));
                        if let Some(token) = tokens.peek() {
                            lookahead = (*token).clone();
                        }
                    }
                }
                lhs = Node::BinaryExpr { 
                    op: Operator::from(&op.t_type),
                    lhs: Box::new(lhs), 
                    rhs: Box::new(rhs)
                };
            }
        }
    }
    lhs
}

fn parse_primary(tokens: &mut Peekable<Iter<Token>>) -> Node {
    let token = tokens.next();

    if token.is_none() {
        eprintln!(
            "Expected expr at line {:?}:{:?}",
            &token.unwrap().line_c,
            &token.unwrap().col_s
        );
        panic!();
    }

    match &token.unwrap().t_type {
        TokenType::NumLit(lit) => Node::Int(*lit),
        TokenType::StringLit(lit) => Node::String(lit.clone()),
        TokenType::Identifier(name) => Node::Var(name.clone()),
        _ => unimplemented!()
    }
}

fn parse_input(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
    let token = tokens.next();
    if token.is_none() {
        eprintln!(
            "Expected identifier at line {:?}:{:?}",
            &token.unwrap().line_c,
            &token.unwrap().col_s
        );
        panic!();
    }

    if let TokenType::Identifier(name) = &token.unwrap().t_type {
        Box::new(Node::Input {
            child: Box::new(Node::Var(name.clone())),
        })
    } else {
        eprintln!(
            "Expected identifier at line {:?}:{:?}",
            &token.unwrap().line_c,
            &token.unwrap().col_s
        );
        panic!();
    }
}

fn parse_output(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
    let mut children = Vec::<Box<Node>>::new();
    let mut expect_separator = false;
    loop {
        let token = tokens.peek();
        if token.is_none() || token.unwrap().t_type == TokenType::LineEnd {
            break;
        }
        if expect_separator {
            if token.unwrap().t_type != TokenType::Comma {
                break;
            }
            tokens.next();
            expect_separator = false;
        } else {
            children.push(Box::new(parse_expr(tokens)));
            expect_separator = true;
        }
    }
    Box::new(Node::Output { children })
}

fn parse_declare(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
    let mut identifiers = Vec::<String>::new();

    loop {
        let mut token = tokens.next();
        if token.is_none() {
            break;
        }
        match &token.unwrap().t_type {
            TokenType::Identifier(name) => {
                identifiers.push(name.clone());
                token = tokens.next();
                match &token.unwrap().t_type {
                    TokenType::Comma => (),
                    TokenType::Colon => break,
                    TokenType::Identifier(_) => eprintln!("Expected comma"),
                    _ => eprintln!("Error declaring"),
                }
            }
            _ => eprintln!("Expected identifier"),
        }
    }

    let token = tokens.next();
    match &token.unwrap().t_type {
        TokenType::Integer => Box::new(Node::Declare {
            t: VariableType::Integer,
            children: identifiers,
        }),
        TokenType::Real => Box::new(Node::Declare {
            t: VariableType::Real,
            children: identifiers,
        }),
        TokenType::Char => Box::new(Node::Declare {
            t: VariableType::Char,
            children: identifiers,
        }),
        TokenType::String => Box::new(Node::Declare {
            t: VariableType::String,
            children: identifiers,
        }),
        TokenType::Date => Box::new(Node::Declare {
            t: VariableType::Date,
            children: identifiers,
        }),
        _ => {
            eprintln!("Expected a valid primitive type");
            panic!()
        }
    }
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Equal,
    NotEqual,
    Greater,
    Lesser,
    GreaterEqual,
    LesserEqual,
    And,
    Or,
    Not,
}

impl From<&TokenType> for Operator {
    fn from(t_type: &TokenType) -> Operator {
        match t_type {
            TokenType::Plus => Operator::Plus,
            TokenType::Minus => Operator::Minus,
            TokenType::Star => Operator::Star,
            TokenType::Slash => Operator::Slash,
            TokenType::Equal => Operator::Equal,
            TokenType::NotEqual => Operator::NotEqual,
            TokenType::Greater => Operator::Greater,
            TokenType::Lesser => Operator::Lesser,
            TokenType::GreaterEqual => Operator::GreaterEqual,
            TokenType::LesserEqual => Operator::LesserEqual,
            TokenType::And => Operator::And,
            TokenType::Or => Operator::Or,
            TokenType::Not => Operator::Not,
            _ => unreachable!()
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Node {
    Main {
        children: Vec<Box<Node>>,
    },
    Var(String),
    Int(i64),
    String(String),
    Declare {
        t: VariableType,
        // Identifiers
        children: Vec<String>,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    BinaryExpr {
        op: Operator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Output {
        children: Vec<Box<Node>>,
    },
    Input {
        child: Box<Node>,
    },
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum VariableType {
    Integer,
    Real,
    Char,
    String,
    Date,
    Array(Box<VariableType>),
    Composite,
}

fn op_precedence(t_type: &TokenType) -> i8 {
    match t_type {
        TokenType::Plus => 5,
        TokenType::Minus => 5,
        TokenType::Star => 6,
        TokenType::Slash => 6,
        TokenType::Equal => 3,
        TokenType::NotEqual => 3,
        TokenType::Greater => 4,
        TokenType::Lesser => 4,
        TokenType::GreaterEqual => 4,
        TokenType::LesserEqual => 4,
        TokenType::And => 2,
        TokenType::Or => 1,
        TokenType::Not => 1,
        _ => -1
    }
}
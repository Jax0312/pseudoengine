use crate::lexer::*;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &mut Vec<Token>) {
    let mut nodes = Vec::<Box<Node>>::new();
    
    let mut tokens = tokens.iter().peekable();
    
    while tokens.peek().is_some() {
        match tokens.next().unwrap().t_type {
            TokenType::Declare => nodes.push(parse_declare(&mut tokens)),
            TokenType::Output => nodes.push(parse_output(&mut tokens)),
            _ => (),
        }
    }
    
    
}

fn parse_output(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {

    let mut children = Vec::<Box<Node>>::new();
    
    loop {
        let mut token = tokens.next();
        if token.is_none() { break; }
        match &token.unwrap().t_type {
            TokenType::StringLit(value) => children.push(Box::new(Node::String(value.clone()))),
            // parse number expression TokenType::NumLit()
            TokenType::Identifier(name) => children.push(Box::new(Node::Var(name.clone()))),
            // possible function as well
            _ => { eprintln!("Invalid expresssion"); panic!(); },
        }
    }
    
    Box::new(Node::Output { children: children })
    
}

fn parse_declare(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
    let mut identifiers = Vec::<String>::new();

    loop {
        let mut token = tokens.next();
        if token.is_none() { break; }
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
            },
            _ => eprintln!("Expected identifier"),
        }
    }

    let mut token = tokens.next();
    match &token.unwrap().t_type {
        TokenType::Integer => Box::new(Node::Declare { t: TokenType::Integer, children: identifiers }),
        TokenType::Real => Box::new(Node::Declare { t: TokenType::Real, children: identifiers }),
        TokenType::Char => Box::new(Node::Declare { t: TokenType::Char, children: identifiers }),
        TokenType::String => Box::new(Node::Declare { t: TokenType::String, children: identifiers }),
        TokenType::Date => Box::new(Node::Declare { t: TokenType::Date, children: identifiers }),
        _ => {
            eprintln!("Expected a valid type");
            panic!()
        },
    }
    
    
}

#[derive(Debug)]
pub enum Operator {
    Plus,
    Minus
}

#[derive(Debug)]
pub enum Node {
    Main {
        children: Vec<Box<Node>>,
    },
    Var(String),
    Int(i64),
    String(String),
    Declare {
        t: TokenType,
        // Identifiers
        children: Vec<String>
    },
    UnaryExpr {
        op: Operator,
        child: Box<Node>,
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
    }
}
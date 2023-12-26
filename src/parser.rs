use crate::lexer::*;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse(tokens: &mut Vec<Token>) -> Vec<Box<Node>> {
    let mut nodes = Vec::<Box<Node>>::new();
    
    let mut tokens = tokens.iter().peekable();
    
    while tokens.peek().is_some() {
        match tokens.next().unwrap().t_type {
            TokenType::Declare => nodes.push(parse_declare(&mut tokens)),
            TokenType::Output => nodes.push(parse_output(&mut tokens)),
            TokenType::Input => nodes.push(parse_input(&mut tokens)),
            _ => (),
        }
    }
    
    nodes

}

fn parse_input(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {

    let token = tokens.next();
    if token.is_none() {
        eprintln!("Expected identifier at line {:?}:{:?}", &token.unwrap().line_c, &token.unwrap().col_s);
        panic!();
    }

    if let TokenType::Identifier(name) = &token.unwrap().t_type {
        Box::new(Node::Input { child: Box::new(Node::Var(name.clone()))})
    } else {
        eprintln!("Expected identifier at line {:?}:{:?}", &token.unwrap().line_c, &token.unwrap().col_s);
        panic!();
    }

}

fn parse_output(tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {

    let mut children = Vec::<Box<Node>>::new();
    let mut expectSeperator  = false;
    loop {
        let mut token = tokens.next();
        if token.is_none() || token.unwrap().t_type == TokenType::LineEnd { break; }
        if (expectSeperator) {
            if (token.unwrap().t_type != TokenType::Comma) {
                eprintln!("Expected comma at line {:?}:{:?}", &token.unwrap().line_c, &token.unwrap().col_s);
                panic!();
            }
            expectSeperator = false;
        } else {
            match &token.unwrap().t_type {
                TokenType::StringLit(value) => children.push(Box::new(Node::String(value.clone()))),
                TokenType::NumLit(value) => children.push(Box::new(Node::Int(value.clone()))),
                TokenType::Identifier(name) => children.push(Box::new(Node::Var(name.clone()))),
                // TODO
                // Parse function as well
                // Parse lambda expression
                _ => { eprintln!("Invalid expresssion {:?} at line {:?}:{:?}", &token.unwrap().t_type, &token.unwrap().line_c, &token.unwrap().col_s); panic!(); },
            }
            expectSeperator = true;
        }
    }
    println!("{:?}", children);
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
        TokenType::Integer => Box::new(Node::Declare { t: VariableType::Integer, children: identifiers }),
        TokenType::Real => Box::new(Node::Declare { t: VariableType::Real, children: identifiers }),
        TokenType::Char => Box::new(Node::Declare { t: VariableType::Char, children: identifiers }),
        TokenType::String => Box::new(Node::Declare { t: VariableType::String, children: identifiers }),
        TokenType::Date => Box::new(Node::Declare { t: VariableType::Date, children: identifiers }),
        _ => {
            eprintln!("Expected a valid primitive type");
            panic!()
        },
    }
    
    
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus
}

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

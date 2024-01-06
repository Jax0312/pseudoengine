use crate::lexer::*;
use std::iter::Peekable;
use std::slice::Iter;
use crate::parser::Node::{Var};

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

    nodes
}

fn parse_assignment(tokens: &mut Peekable<Iter<Token>>, identifier: String) -> Box<Node> {

    let token = tokens.next();
    if token.is_none() || token.unwrap().t_type != TokenType::Assignment {
        ()
    }

    Box::new(Node::Assignment {
        lhs: Box::new(Node::Var(identifier.clone())),

        rhs: {
            let token = tokens.next();
            match token {
                Some(token) => match &token.t_type {
                    TokenType::StringLit(value) => Box::new(Node::String(value.clone())),
                    TokenType::Identifier(identifier) => Box::new(Node::Var(identifier.clone())),
                    TokenType::NumLit(value) => {
                        // Start Operator Precedence Parsing
                        op_pred_parsing(*value, tokens)
                    }
                    _ => {
                        eprintln!(
                            "Unexpected {:?} at line {:?}:{:?}",
                            token.t_type,
                            token.line_c,
                            token.col_s
                        );
                        panic!();
                    }
                },
                None => {
                    eprintln!(
                        "Expected a value at line {:?}:{:?}",
                        token.unwrap().line_c,
                        &token.unwrap().col_s
                    );
                    panic!();
                }
            }
        },
    })
}

fn op_pred_parsing(current_num : i64, tokens: &mut Peekable<Iter<Token>>) -> Box<Node> {
    let mut operands = Vec::<Box<Node>>::new();
    operands.push(Box::new(Node::Int(current_num)));

    loop {
        let token = tokens.next();
        if token.is_none() || &token.unwrap().t_type == &TokenType::LineEnd {
            break;
        }

        let token = token.unwrap().clone();

        match &token.t_type {
            TokenType::NumLit(num) => operands.push(Box::new(Node::Int(*num))),
            TokenType::Plus => operands.push(Box::new(Node::Operator {op: Operator::Plus})),
            TokenType::Minus => operands.push(Box::new(Node::Operator {op: Operator::Minus})),
            TokenType::Identifier(name) => operands.push(Box::new(Var(name.clone()))),
            TokenType::Star | TokenType::Slash => {
                // Proceed with next token
                let next_token = tokens.next();
                if next_token.is_none() {
                        eprintln!(
                            "Invalid operand at line {:?}:{:?}",
                            token.line_c,
                            token.col_s
                        );
                        panic!();

                }
                if let Some(node) = operands.pop() {
                    match *node {
                        Node::Var(..) | Node::Int(..) | Node::BinaryExpr {..} => {
                            let operation : Operator = match &token.t_type {
                                TokenType::Star => Operator::Multiply,
                                _ => Operator::Divide,
                            };
                            operands.push(Box::new(
                                Node::BinaryExpr {
                                    op: operation,
                                    lhs: node,
                                    rhs: Box::new(match &next_token.unwrap().t_type {
                                        TokenType::NumLit(value) => Node::Int(*value),
                                        TokenType::Identifier(name) => Node::Var(name.clone()),
                                        _ => {
                                            eprintln!(
                                                "Illegal operation at line {:?}:{:?}",
                                                token.line_c,
                                                token.col_s
                                            );
                                            panic!();
                                        }
                                    })
                                }
                            ))
                        },
                        _ => {
                            eprintln!(
                                "Illegal operation at line {:?}:{:?}",
                                token.line_c,
                                token.col_s
                            );
                            panic!();
                        }
                    }
                }
            },
            _ => {
                eprintln!(
                    "Expected value at line {:?}:{:?}",
                    token.line_c,
                    token.col_s
                );
                panic!();
            }
        }

    }

    println!("{:?}", operands);

    Box::new(Node::Var("temp".to_string()))
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
    let mut expect_seperator = false;
    loop {
        let mut token = tokens.next();
        if token.is_none() || token.unwrap().t_type == TokenType::LineEnd {
            break;
        }
        if expect_seperator {
            if token.unwrap().t_type != TokenType::Comma {
                eprintln!(
                    "Expected comma at line {:?}:{:?}",
                    &token.unwrap().line_c,
                    &token.unwrap().col_s
                );
                panic!();
            }
            expect_seperator = false;
        } else {
            match &token.unwrap().t_type {
                TokenType::StringLit(value) => children.push(Box::new(Node::String(value.clone()))),
                TokenType::NumLit(value) => children.push(Box::new(Node::Int(value.clone()))),
                TokenType::Identifier(name) => children.push(Box::new(Node::Var(name.clone()))),
                // TODO
                // Parse function as well
                // Parse lambda expression
                _ => {
                    eprintln!(
                        "Invalid expresssion {:?} at line {:?}:{:?}",
                        &token.unwrap().t_type,
                        &token.unwrap().line_c,
                        &token.unwrap().col_s
                    );
                    panic!();
                }
            }
            expect_seperator = true;
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

    let mut token = tokens.next();
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
    Multiply,
    Divide,
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
        children: Vec<String>,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    Operator {
        op: Operator,
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

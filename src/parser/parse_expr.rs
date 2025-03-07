use crate::enums::{Node, Position, Token, VariableType};
use crate::lexer::Lexer;
use crate::parser::parse_identifier::parse_identifier;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

#[derive(Debug, PartialEq, Clone)]
enum Associativity {
    Left,
    Right,
}

pub fn parse_expression(lexer: &mut Lexer) -> Box<Node> {
    let lhs = parse_unary(lexer);
    parse_expr_precedence(lexer, lhs, 0)
}

fn parse_unary(lexer: &mut Lexer) -> Box<Node> {
    let token = lexer.peek().unwrap().clone();
    if is_unary(&token) {
        let op = get_op(&lexer.next().unwrap());
        let expr = parse_unary(lexer);
        let pos = Position::range(token.pos, expr.pos());
        return Box::new(Node::Unary { op, expr, pos });
    }
    if TToken::Caret == token.t {
        lexer.next();
        let expr = parse_unary(lexer);
        let pos = Position::range(token.pos, expr.pos());
        return Box::new(Node::Reference { expr, pos });
    }
    let mut expr = parse_primary(lexer);
    while TToken::Caret == lexer.peek().unwrap().t {
        let token = lexer.next().unwrap();
        let pos = Position::range(expr.pos(), token.pos);
        expr = Box::new(Node::Dereference { expr, pos });
    }
    expr
}

fn parse_expr_precedence(lexer: &mut Lexer, mut lhs: Box<Node>, precedence: i8) -> Box<Node> {
    if let Some(token) = lexer.peek() {
        let mut lookahead = token.clone();
        while lexer.peek().is_some() && op_precedence(&lookahead) >= precedence {
            let op = lookahead.clone();
            if lexer.next().is_some() {
                let mut rhs = parse_unary(lexer);
                if let Some(token) = lexer.peek() {
                    lookahead = (*token).clone();
                    while lexer.peek().is_some() && op_precedence(&lookahead) > precedence {
                        rhs = parse_expr_precedence(lexer, rhs, op_precedence(&lookahead));
                        if let Some(token) = lexer.peek() {
                            lookahead = (*token).clone();
                        }
                    }
                }
                let op = get_op(&op);
                let pos = Position::range(lhs.pos(), rhs.pos());
                lhs = Box::new(Node::Binary { op, lhs, rhs, pos });
            }
        }
    }
    lhs
}

fn parse_primary(lexer: &mut Lexer) -> Box<Node> {
    let token = lexer.peek().clone().unwrap();
    let pos = token.pos;
    match token.t.clone() {
        TToken::StringLit(val) => {
            lexer.next();
            Box::new(Node::String { val, pos })
        }
        TToken::IntegerLit(val) => {
            lexer.next();
            Box::new(Node::Int { val, pos })
        }
        TToken::RealLit(val) => {
            lexer.next();
            Box::new(Node::Real { val, pos })
        }
        TToken::BoolLit(val) => {
            lexer.next();
            Box::new(Node::Boolean { val, pos })
        }
        TToken::DateLit(val) => {
            lexer.next();
            Box::new(Node::Date { val, pos })
        }
        TToken::LParen => parse_group(lexer),
        TToken::New => parse_new(lexer),
        TToken::Identifier(_) => parse_identifier(lexer),
        _ => err("Expected espression", &token.pos),
    }
}

fn parse_group(lexer: &mut Lexer) -> Box<Node> {
    let lparen = lexer.next().unwrap();
    let expr = parse_expression(lexer);
    let rparen = expect_token(lexer, &[TToken::RParen], "')'");
    let pos = Position::range(lparen.pos, rparen.pos);
    expr
}

fn parse_new(lexer: &mut Lexer) -> Box<Node> {
    let new = lexer.next().unwrap();
    let call = parse_identifier(lexer);
    match *call {
        Node::FunctionCall { .. } => (),
        _ => err("Class constructor call expected", &call.pos()),
    }
    let pos = Position::range(new.pos, call.pos());
    Box::from(Node::CreateObject { call, pos })
}

fn op_precedence(op: &Token) -> i8 {
    if let TToken::Operator(op) = &op.t {
        return match op.as_str() {
            "&&" | "||" => 1,
            "=" | "!=" | ">=" | ">" | "<" | "<=" => 2,
            "+" | "-" => 3,
            "*" | "/" | "//" | "%" | "&" => 4,
            _ => -1,
        };
    }
    -1
}

fn is_unary(op: &Token) -> bool {
    if let TToken::Operator(op) = &op.t {
        match op.as_str() {
            "_+" | "_-" | "!" => return true,
            _ => return false,
        }
    }
    false
}

fn get_op(op: &Token) -> String {
    if let TToken::Operator(op) = &op.t {
        return op.clone();
    }
    unreachable!()
}

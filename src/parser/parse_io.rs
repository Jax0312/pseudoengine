use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_input(lexer: &mut Lexer) -> Box<Node> {
    // skip INPUT token
    let token = lexer.next().unwrap();
    let Token {
        t: TToken::Identifier(name),
        pos,
    } = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier")
    else {
        unreachable!()
    };
    let child = Box::new(Node::Var { name, pos });
    let pos = Position::range(token.pos, pos);
    Box::from(Node::Input { child, pos })
}

pub fn parse_output(lexer: &mut Lexer) -> Box<Node> {
    let mut children = Vec::new();

    // skip OUTPUT token
    let token = lexer.next().unwrap();
    loop {
        children.push(parse_expression(lexer));
        if lexer.peek().unwrap().t != TToken::Comma {
            break;
        }
        lexer.next();
    }
    let pos = Position::range(token.pos, children.last().unwrap().pos());
    Box::from(Node::Output { children, pos })
}

pub fn parse_open_file(lexer: &mut Lexer) -> Box<Node> {
    // skip OPENFILE token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::For], "'FOR'");
    let Token { t: mode, pos } = expect_token(
        lexer,
        &[TToken::FileMode("".to_string())],
        "'APPEND', 'READ', 'WRITE'",
    );
    let pos = Position::range(token.pos, pos);
    Box::from(Node::OpenFile {
        filename,
        mode,
        pos,
    })
}

pub fn parse_close_file(lexer: &mut Lexer) -> Box<Node> {
    // skip CLOSEFILE token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    let pos = Position::range(token.pos, filename.pos());
    Box::from(Node::CloseFile { filename, pos })
}

pub fn parse_read_file(lexer: &mut Lexer) -> Box<Node> {
    // skip READFILE token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::Comma], "','");
    let var = parse_identifier(lexer);
    match *var {
        Node::Var { .. } | Node::ArrayVar { .. } => (),
        _ => err("Identifier expected", &var.pos()),
    }
    let pos = Position::range(token.pos, var.pos());
    Box::from(Node::ReadFile { filename, var, pos })
}

pub fn parse_write_file(lexer: &mut Lexer) -> Box<Node> {
    // skip WRITEFILE token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::Comma], "','");
    let expr = parse_expression(lexer);
    let pos = Position::range(token.pos, expr.pos());
    Box::from(Node::WriteFile {
        filename,
        expr,
        pos,
    })
}

pub fn parse_seek_file(lexer: &mut Lexer) -> Box<Node> {
    // skip SEEK token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::Comma], "','");
    let expr = parse_expression(lexer);
    let pos = Position::range(token.pos, expr.pos());
    Box::from(Node::SeekFile {
        filename,
        expr,
        pos,
    })
}

pub fn parse_get_record(lexer: &mut Lexer) -> Box<Node> {
    // skip GETRECORD token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::Comma], "','");
    let var = parse_identifier(lexer);
    match *var {
        Node::Var { .. } | Node::ArrayVar { .. } => (),
        _ => err("Identifier expected", &var.pos()),
    }
    let pos = Position::range(token.pos, var.pos());
    Box::from(Node::GetRecord { filename, var, pos })
}

pub fn parse_put_record(lexer: &mut Lexer) -> Box<Node> {
    // skip PUTRECORD token
    let token = lexer.next().unwrap();
    let filename = parse_expression(lexer);
    expect_token(lexer, &[TToken::Comma], "','");
    let var = parse_identifier(lexer);
    match *var {
        Node::Var { .. } | Node::ArrayVar { .. } => (),
        _ => err("Identifier expected", &var.pos()),
    }
    let pos = Position::range(token.pos, var.pos());
    Box::from(Node::PutRecord { filename, var, pos })
}

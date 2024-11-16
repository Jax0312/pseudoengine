use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_input(lexer: &mut Lexer) -> Box<Node> {
    // skip INPUT token
    lexer.next();
    let Token {t: TToken::Identifier(name), pos} = expect_token(lexer, &[TToken::Identifier("".to_string())], "Identifier") else { unreachable!() };
    Box::from(Node::Input {
        child: Box::new(Node::Var {
            name, pos
        }),
    })
}

pub fn parse_output(lexer: &mut Lexer) -> Box<Node> {
    let mut children = Vec::new();
    
    // skip OUTPUT token
    lexer.next();
    loop {
        let (exp, stop_token) = parse_expression(lexer, &[TToken::Comma]);
        children.push(exp);
        if stop_token.t != TToken::Comma {
            break;
        }
    }

    Box::from(Node::Output {
        children
    })
}

pub fn parse_open_file(lexer: &mut Lexer) -> Box<Node> {
    // skip OPENFILE token
    lexer.next();
    let (filename, stop_token) = parse_expression(lexer, &[TToken::For]);
    if stop_token.t != TToken::For {
        err("'FOR' expected", &stop_token.pos);
    }
    let Token {t: mode, pos: _} = expect_token(lexer, &[TToken::FileMode("".to_string())], "'APPEND', 'READ', 'WRITE' expected");
    Box::from(Node::OpenFile {
        filename,
        mode
    })
}

pub fn parse_close_file(lexer: &mut Lexer) -> Box<Node> {
    // skip CLOSEFILE token
    lexer.next();
    Box::from(Node::CloseFile(parse_expression(lexer, &[]).0))
}

pub fn parse_read_file(lexer: &mut Lexer) -> Box<Node> {
    // skip READFILE token
    lexer.next();
    let (filename, stop_token) = parse_expression(lexer, &[TToken::Comma]);
    if stop_token.t != TToken::Comma {
        err("',' expected", &stop_token.pos);
    }
    let var= parse_identifier(lexer);
    match *var {
        Node::Var {..} | Node::ArrayVar {..} => (),
        _ => err("Identifier expected", &stop_token.pos),
    }
    Box::from(Node::ReadFile {
        filename,
        var,
    })
}

pub fn parse_write_file(lexer: &mut Lexer) -> Box<Node> {
    // skip WRITEFILE token
    lexer.next();
    let (filename, stop_token) = parse_expression(lexer, &[TToken::Comma]);
    if stop_token.t != TToken::Comma {
        err("',' expected", &stop_token.pos);
    }
    Box::from(Node::WriteFile {
        filename,
        expr: parse_expression(lexer, &[]).0,
    })
}
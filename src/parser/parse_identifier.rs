use crate::enums::{Node, Position, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::tokens::TToken;
use crate::utils::err;

// Function handles variable, array and function call
pub fn parse_identifier(lexer: &mut Lexer) -> Box<Node> {
    let mut children = Vec::new();
    loop {
        let ident = lexer.next().unwrap();
        let name = match ident.t {
            TToken::Identifier(name) => name,
            _ => err("Expected identifier", &ident.pos),
        };

        let token = lexer.peek().unwrap().clone();
        match token.t {
            TToken::LSqrBracket => {
                lexer.next();
                let mut indices = Vec::new();
                loop {
                    indices.push(parse_expression(lexer));
                    if lexer.peek().unwrap().t == TToken::RSqrBracket {
                        break;
                    }
                    let token = lexer.next().unwrap();
                    if token.t != TToken::Comma {
                        err("Expected comma", &token.pos);
                    }
                }
                let end = lexer.next().unwrap();
                let pos = Position::range(ident.pos, end.pos);
                children.push(Box::from(Node::ArrayVar { name, indices, pos }))
            }
            TToken::LParen => {
                lexer.next();
                let mut params = Vec::new();
                if TToken::RParen != lexer.peek().unwrap().t {
                    loop {
                        params.push(parse_expression(lexer));
                        if lexer.peek().unwrap().t == TToken::RParen {
                            break;
                        }
                        let token = lexer.next().unwrap();
                        if token.t != TToken::Comma {
                            err("Expected comma", &token.pos);
                        }
                    }
                }
                let end = lexer.next().unwrap();
                let pos = Position::range(ident.pos, end.pos);
                children.push(Box::from(Node::FunctionCall { name, params, pos }))
            }
            _ => {
                let pos = ident.pos.clone();
                children.push(Box::from(Node::Var { name, pos }));
            }
        }

        let token = lexer.peek().unwrap();
        let name = match token.t {
            TToken::Period => lexer.next(),
            _ => break,
        };
    }

    if children.len() > 1 {
        let pos = Position::range(
            children.first().unwrap().pos(),
            children.last().unwrap().pos(),
        );
        Box::new(Node::Composite { children, pos })
    } else {
        children[0].clone()
    }
}

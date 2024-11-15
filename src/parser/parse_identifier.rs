use crate::enums::{Node, Token};
use crate::enums::Node::FunctionCall;
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::tokens::TToken;
use crate::utils::{err};

// Function handles variable, array and function call
pub fn parse_identifier(lexer: &mut Lexer) -> Box<Node> {

    match lexer.next() {
        Some(Token {t: TToken::Identifier(name), pos}) => {
            match lexer.peek() {
                Some(&Token { t: TToken::LSqrBracket, pos }) => {
                    // Handle array
                    // Skip '['
                    lexer.next();
                    let mut indices = Vec::new();
                    
                    loop {
                        let (exp, res) = parse_expression(lexer, &[TToken::Comma, TToken::RSqrBracket]);
                        indices.push(exp);
                        if res.is_some() && res.unwrap() == TToken::RSqrBracket {
                            break
                        }
                    }
                    
                    Box::from(Node::ArrayVar {
                        name,
                        indices,
                        pos,
                    })
                },
                Some(&Token { t: TToken::LParen, pos: _ }) => {
                    // Handle function
                    // Skip '('
                    lexer.next();
                    let mut params = Vec::new();
                    loop {
                        let (exp, res) = parse_expression(lexer, &[TToken::Comma, TToken::RParen]);
                        params.push(exp);
                        if res.is_some() && res.unwrap() == TToken::RParen {
                            break
                        }
                    }
                    Box::from(FunctionCall {
                        name,
                        params,
                    })
                }
                _ => Box::from(Node::Var {name, pos})
            }
        },
        Some(Token {t: _, pos}) => err("Identifier expected", &pos),
        _ => unreachable!()
    } 
    
}

use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::tokens::TToken;
use crate::utils::err;

// Function handles variable, array and function call
pub fn parse_identifier(lexer: &mut Lexer) -> Box<Node> {
    let mut is_comp = false;
    let mut make_ref = false;
    let mut vars = Vec::new();

    loop {
        match lexer.next() {
            Some(Token {
                t: TToken::Identifier(name),
                pos,
            }) => {
                match lexer.peek() {
                    Some(&Token {
                        t: TToken::LSqrBracket,
                        pos,
                    }) => {
                        // Handle array
                        // Skip '['
                        lexer.next();
                        let mut indices = Vec::new();

                        loop {
                            let (exp, res) =
                                parse_expression(lexer, &[TToken::Comma, TToken::RSqrBracket]);
                            indices.push(exp);
                            if res.t == TToken::RSqrBracket {
                                break;
                            }
                        }

                        vars.push(Box::from(Node::ArrayVar { name, indices, pos }))
                    }
                    Some(&Token {
                        t: TToken::LParen,
                        pos: _,
                    }) => {
                        // Handle function
                        // Skip '('
                        lexer.next();
                        let mut params = Vec::new();
                        
                        if let Some(Token {t: TToken::RParen, pos: _}) = lexer.peek() {
                            // No need to parse expr
                            lexer.next();
                        } else {
                            loop {
                                let (exp, res) =
                                    parse_expression(lexer, &[TToken::Comma, TToken::RParen]);
                                params.push(exp);
                                if res.t == TToken::RParen {
                                    break;
                                }
                            }                            
                        }
                        
                        vars.push(Box::from(Node::FunctionCall { name, params }))
                    }
                    _ => {
                        vars.push(Box::from(Node::Var { name, pos }));
                    }
                }

                if make_ref {
                    // make ref
                    let temp = Box::from(Node::Reference(vars.pop().unwrap()));
                    vars.push(temp);
                    make_ref = false;
                }

                if let Some(&Token {
                    t: TToken::Caret,
                    pos: _,
                }) = lexer.peek()
                {
                    // make deref
                    let temp = Box::from(Node::Dereference(vars.pop().unwrap()));
                    vars.push(temp);
                    lexer.next();
                }
                
                // repeat if period is found
                match lexer.peek() {
                    Some(&Token {
                        t: TToken::Period,
                        pos: _,
                    }) => {
                        is_comp = true;
                        lexer.next();
                    }

                    _ => break,
                }
            }
            Some(Token {
                t: TToken::Caret,
                pos: _,
            }) => make_ref = true,
            Some(Token { t: _, pos }) => err("Identifier expected", &pos),
            _ => unreachable!(),
        }
    }

    if is_comp {
        return Box::from(Node::Composite { children: vars });
    }

    vars[0].clone()
}

use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_line;
use crate::tokens::TToken;
use crate::utils::{err, expect_token};

pub fn parse_if(lexer: &mut Lexer) -> Box<Node> {
    
    // skip IF token
    lexer.next();
    let (cond, stop_token) = parse_expression(lexer, &[TToken::Then]);
    // Then can be on the next line
    if stop_token.t != TToken::Then {
        expect_token(lexer, &[TToken::Then], "'THEN'");    
    }
    
    let mut true_body = vec![];
    let mut else_encountered = false;
    let mut false_body = vec![];

    loop {
        match lexer.peek() {
            Some(Token {t: TToken::EOF, pos}) => err("'ENDIF' expected", pos),
            Some(Token { t: TToken::EndIf, pos: _}) => {
                lexer.next();
                break;
            },
            Some(Token {t: TToken::Else, pos: _}) => {
                else_encountered = true;
                lexer.next();
            }
            ,
            _ => {
                if else_encountered {
                    false_body.push(parse_line(lexer))
                } else {
                    true_body.push(parse_line(lexer))    
                }
                   
            }
        }
    }
    
    Box::from(Node::If{
        cond,
        true_body,
        false_body,
    })
    
}

// pub fn parse_case(lexer: &mut Lexer) -> Box<Node> {
//     // skip CASE token
//     lexer.next();
// }
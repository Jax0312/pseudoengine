use crate::enums::{Node, Token};
use crate::lexer::Lexer;
use crate::parser::parse_expr::parse_expression;
use crate::parser::parse_identifier::parse_identifier;
use crate::parser::{parse_line, try_parse_assign};
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

// TODO: finish case and parsing for composite type, enum and set declaration
pub fn parse_case(lexer: &mut Lexer) -> Box<Node> {
    // skip CASE token
    lexer.next();
    expect_token(lexer, &[TToken::Of], "'Of'");
    let cmp = parse_identifier(lexer);
    let mut cases = vec![];
    
    loop {
        let expr;
        // Handle case condition
        let (c1, token) =  parse_expression(lexer, &[TToken::Colon, TToken::To]); 
           match token.t {
               TToken::Colon => expr = c1,
               TToken::To => {
                   let (c2, stop_token) = parse_expression(lexer, &[TToken::Colon]);
                   if stop_token.t != TToken::Colon {
                       err("':' expected", &stop_token.pos)   
                   }
                   expr = Box::from(Node::Range {start:c1, end:c2});
               },
               _ => err("':' expected", &token.pos),
           }
        
        // Handle case body
        let mut children = vec![];
        match lexer.peek() {
            Some(Token {t: TToken::EOF, pos}) => err("ENDCASE expected", pos),
            Some(Token {t: TToken::EndCase, pos: _}) => {
                lexer.next();
                break;
            }
            Some(Token {t: TToken::Identifier(_), pos: _}) => {
                let lhs = parse_identifier(lexer);
                
                children.push(try_parse_assign(lexer, lhs));
            }
            _ => children.push(parse_line(lexer)),
        }
        
    }
    
    Box::from(Node::Switch {
        cmp,
        cases,
        otherwise: vec![],
    })
}
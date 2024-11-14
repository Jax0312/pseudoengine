use std::iter::Peekable;
use crate::enums::{Identifier, Position, Token, VariableType};
use std::str::Chars;
use std::vec::IntoIter;
use crate::tokens::TToken;

pub fn lexer(buf: &mut Chars) -> Vec<Token> {
    let mut c_pos = Position { line: 1, col: 0 };
    let mut tokens = Vec::new();
    let mut buf = buf.peekable();

    while let Some(ch) = buf.next() {
        c_pos.col += 1;
        match ch {
            ' ' | '\t' | '\r' => {
                // Whitespace
            }
            '\n' => {
                c_pos.line += 1;
                c_pos.col = 1;
                tokens.push(Token {
                    t: TToken::Newline,
                    pos: c_pos.clone(),
                })
            }
            '0'..='9' => {
                let mut number = String::new();
                number.push(ch);
                while let Some(f) = buf.peek() {
                    if f.is_numeric() || *f == '.' {
                        number.push(buf.next().unwrap());
                        c_pos.col += 1;
                    } else {
                        break;
                    }
                }

                let dot_count = number.chars().filter(|c| *c == '.').count();
                match dot_count {
                    0 => tokens.push(Token {
                        t: TToken::IntegerLit(number.parse::<i64>().unwrap()),
                        pos: c_pos.clone(),
                    }),
                    1 => tokens.push(Token {
                        t: TToken::RealLit(number.parse::<f64>().unwrap()),
                        pos: c_pos.clone(),
                    }),
                    _ => {
                        println!("{:?}", tokens);
                        err("Invalid value", &c_pos)   
                    },
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut word = String::new();
                word.push(ch);
                while let Some(f) = buf.peek() {
                    if !f.is_alphanumeric() && *f != '_' {
                        break;
                    }
                    word.push(buf.next().unwrap());
                    c_pos.col += 1;
                }
                
                tokens.push(Token {
                    t: match_word(word),
                    pos: c_pos.clone(),
                })
            },
            '/' => {
                if let Some('/') = buf.peek() {
                    // Comment detected, ignore until \n
                    while buf.peek() != Some(&'\n') && buf.peek().is_some() {
                        buf.next();
                    }
                } else {
                    tokens.push(Token {t: TToken::Operator("/".to_string()), pos: c_pos.clone()})
                }
            },
            '"' => {
               // String literals
                let mut lit = String::new();
                while let Some(c) = buf.next()  {
                    c_pos.col += 1;
                    if c == '"' {
                        break
                    }
                    lit.push(c);
                }
                
                tokens.push(Token {t: TToken::StringLit(lit), pos: c_pos.clone()});
                
            }
            _ => {
                // Symbols
                let mut sym = String::new();
                sym.push(ch);
                while let Some(f) = buf.peek() {
                    match f {
                        '-' | '=' | '>' => {
                            sym.push(buf.next().unwrap());
                            c_pos.col += 1;                            
                        }
                        _ => break
                    }
                }
                tokens.push(Token {
                    t: match_symbol(sym),
                    pos: c_pos.clone(),
                })
                
            },
        }
    }
    
    // Second pass to reduce multiple newline and identify array 
    let mut temp_tokens = tokens.into_iter().peekable();
    tokens = vec![];
    while let Some(token ) = temp_tokens.peek() {
        tokens.push(temp_tokens.next().unwrap());
        match tokens.last().unwrap().clone().t {
            TToken::Newline => {
                while temp_tokens.peek().is_some() && temp_tokens.peek().unwrap().t == TToken::Newline {
                    temp_tokens.next();
                }
            },
            TToken::Identifier(ident) => {
                if temp_tokens.peek().is_some() && temp_tokens.peek().unwrap().t == TToken::LSqrBracket {
                    let mut indices = vec![];
                    temp_tokens.next();
                    loop {
                        if let TToken::IntegerLit(val) = expect_token(&mut temp_tokens, TToken::IntegerLit(0), "Integer").unwrap().t {
                            if let Ok(index) = usize::try_from(val) {
                               indices.push(index);
                            } else {
                                err("Unsigned integer expected", &tokens.last().unwrap().pos);
                            }

                            if temp_tokens.peek().is_none() {
                                err("] expected", &tokens.last().unwrap().pos);
                            }

                            match temp_tokens.next().unwrap().t {
                                TToken::RSqrBracket => break,
                                TToken::Comma => (),
                                _ => err("] or , expected", &tokens.last().unwrap().pos),  
                            };
                            
                        }
                    }
                    let token = Token {
                        t: TToken::Identifier(Identifier {name: ident.name, indices: Some(indices)}),
                        pos: tokens.pop().unwrap().pos
                    };
                    
                    tokens.push(token);
                    
                }
            },
            _ => ()
        }
    }

    for token in tokens.clone() {
        print!("{:?}, ", token.t);
    }
    println!();

    tokens
}

fn expect_token(
    lexer: &mut Peekable<IntoIter<Token>>,
    token: TToken,
    message: &str,
) -> Option<Token> {
    let next = lexer.next().unwrap();
    if std::mem::discriminant(&next.t) == std::mem::discriminant(&token) {
        Some(next)
    } else {
        println!("{:?}", next);
        err(&format!("{} expected", message), &next.pos);
    }
}

fn match_symbol(sym: String) -> TToken {
    
    match sym.as_str() {
        "+" | "-" | "*" | "/" => TToken::Operator(sym.clone()),
        "<>" => TToken::Operator("!=".to_string()),
        "=" | ">=" | ">" | "<" | "<=" => TToken::Operator(sym.clone()),
        "(" => TToken::LParen,
        ")" => TToken::RParen,
        "[" => TToken::LSqrBracket,
        "]" => TToken::RSqrBracket,
        "&" => TToken::Ampersand,
        "<-" => TToken::Assignment,
        ":" => TToken::Colon,
        "," => TToken::Comma,
        "^" => TToken::Caret,
        "." => TToken::Period,
        _ => TToken::Unknown
        
    }
}

fn match_word(word: String) -> TToken {
    match word.as_str() {
        "APPEND" | "WRITE" | "READ" | "RANDOM" => TToken::FileMode(word),
        "BYREF" | "BYVAL" => TToken::PassBy(word),
        "ARRAY" => TToken::Array,
        "CALL" => TToken::Call,
        "CASE" => TToken::Case,
        "CLASS" => TToken::Class,
        "CLOSEFILE" => TToken::CloseFile,
        "CONSTANT" => TToken::Constant,
        "DECLARE" => TToken::Declare,
        "DEFINE" => TToken::Define,
        "ELSE" => TToken::Else,
        "ENDCASE" => TToken::EndCase,
        "ENDCLASS" => TToken::EndClass,
        "ENDFUNCTION" => TToken::EndFunction,
        "ENDIF" => TToken::EndIf,
        "ENDPROCEDURE" => TToken::EndProcedure,
        "ENDTYPE" => TToken::EndType,
        "ENDWHILE" => TToken::EndWhile,
        "FOR" => TToken::For,
        "FUNCTION" => TToken::Function,
        "GETRECORD" => TToken::GetRecord,
        "IF" => TToken::If,
        "INHERITS" => TToken::Inherits,
        "INPUT" => TToken::Input,
        "NEXT" => TToken::Next,
        "NEW" => TToken::New,
        "OPENFILE" => TToken::OpenFile,
        "OTHERWISE" => TToken::Otherwise,
        "OUTPUT" => TToken::Output,
        "OF" => TToken::Of,
        "PROCEDURE" => TToken::Procedure,
        "PRIVATE" => TToken::Private,
        "PUBLIC" => TToken::Public,
        "PUTRECORD" => TToken::PutRecord,
        "READFILE" => TToken::ReadFile,
        "REPEAT" => TToken::Repeat,
        "RETURN" => TToken::Return,
        "RETURNS" => TToken::ReturnType,
        "TO" => TToken::To,
        "SEEK" => TToken::Seek,
        "STEP" => TToken::Step,
        "SUPER" => TToken::Super,
        "THEN" => TToken::Then,
        "TYPE" => TToken::Type,
        "UNTIL" => TToken::Until,
        "WHILE" => TToken::While,
        "WRITEFILE" => TToken::WriteFile,
        "TRUE" => TToken::BoolLit(true),
        "FALSE" => TToken::BoolLit(false),
        "DIV" => TToken::Operator("//".to_string()),
        "MOD" => TToken::Operator("%".to_string()),
        "AND" => TToken::Operator("&&".to_string()),
        "OR" => TToken::Operator("||".to_string()),
        "NOT" => TToken::Operator("!".to_string()),
        "INTEGER" => TToken::VarType(VariableType::Integer),
        "REAL" => TToken::VarType(VariableType::Real),
        "STRING" => TToken::VarType(VariableType::String),
        "DATE" => TToken::VarType(VariableType::Date),
        _ => TToken::Identifier(Identifier {
            name: word,
            indices: None,
        }),
    }
    
}

fn err(message: &str, pos: &Position) -> ! {
    println!("{} at line {} col {}", message, pos.line, pos.col);
    panic!()
}

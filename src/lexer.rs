use std::iter::Peekable;
use crate::enums::{Position, Token, VariableType};
use std::str::Chars;
use std::vec::IntoIter;
use crate::tokens::TToken;
use crate::utils::{err};

pub type Lexer = Peekable<IntoIter<Token>>;

// TODO: incorrect pos col reporting

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
                // Minus 1 because newline is invisible to user
                c_pos.col -= 1;
                tokens.push(Token {
                    t: TToken::Newline,
                    pos: c_pos,
                });
                c_pos.line += 1;
                c_pos.col = 0;
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
                        pos: c_pos,
                    }),
                    1 => tokens.push(Token {
                        t: TToken::RealLit(number.parse::<f64>().unwrap()),
                        pos: c_pos,
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
                    t: match_word(word.clone()),
                    // Point back to start of word
                    pos: Position {line: c_pos.line, col: c_pos.col - word.len() + 1},
                })
            },
            '/' => {
                if let Some('/') = buf.peek() {
                    // Comment detected, ignore until \n
                    while buf.peek() != Some(&'\n') && buf.peek().is_some() {
                        buf.next();
                    }
                } else {
                    tokens.push(Token {t: TToken::Operator("/".to_string()), pos: c_pos})
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
                
                tokens.push(Token {t: TToken::StringLit(lit), pos: c_pos});
                
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
    
    // EOF token added to signify source file end for error reporting purpose
    c_pos.col += 1;
    tokens.push( Token { t: TToken::EOF, pos: c_pos } );
    
    // Second pass to reduce multiple newline and identify array 
    let mut temp_tokens = tokens.into_iter().peekable();
    tokens = vec![];
    while temp_tokens.peek().is_some() {
        tokens.push(temp_tokens.next().unwrap());
        match tokens.last().unwrap().clone().t {
            TToken::Newline => {
                while temp_tokens.peek().is_some() && temp_tokens.peek().unwrap().t == TToken::Newline {
                    temp_tokens.next();
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



fn match_symbol(sym: String) -> TToken {
    
    match sym.as_str() {
        "+" | "-" | "*" | "/" => TToken::Operator(sym.clone()),
        "<>" => TToken::Operator("!=".to_string()),
        "=" | ">=" | ">" | "<" | "<=" | "&" => TToken::Operator(sym.clone()),
        "(" => TToken::LParen,
        ")" => TToken::RParen,
        "[" => TToken::LSqrBracket,
        "]" => TToken::RSqrBracket,
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
        _ => TToken::Identifier(word),
    }
    
}
use crate::enums::{Position, Token, VariableType};
use crate::tokens::TToken;
use crate::utils::err;
use chrono::NaiveDate;
use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;

pub type Lexer = Peekable<IntoIter<Token>>;

// TODO: incorrect pos start reporting

pub fn lexer(buf: &mut Chars) -> Vec<Token> {
    let mut c_line = 1;
    let mut c_pos = 0;
    let mut tokens = Vec::new();
    let mut buf = buf.peekable();

    while let Some(ch) = buf.next() {
        match ch {
            ' ' | '\t' | '\r' => {
                c_pos += 1;
            }
            '\n' => {
                // Minus 1 because newline is invisible to user
                tokens.push(Token {
                    t: TToken::Newline,
                    pos: Position::from(c_line, c_pos, 1),
                });
                c_line += 1;
                c_pos = 0;
            }
            '0'..='9' => {
                let mut number = String::new();
                let mut dot_count = 0;
                let mut slash_count = 0;
                // flag for possibility literal is a date
                let mut possible_date = false;
                // save start position cuz it might be used
                number.push(ch);

                while let Some(f) = &buf.peek() {
                    match f {
                        '0'..='9' => {}
                        '/' => {
                            slash_count += 1;
                            possible_date = slash_count == 2 && dot_count == 0;
                        }
                        '.' => {
                            dot_count += 1;
                        }
                        _ => break,
                    }
                    number.push(buf.next().unwrap());
                }

                if possible_date {
                    let pos = Position::from(c_line, c_pos, number.len());
                    if let Ok(date) = NaiveDate::parse_from_str(&*number, "%d/%m/%Y") {
                        tokens.push(Token {
                            t: TToken::DateLit(date),
                            pos: pos,
                        })
                    } else {
                        err("Invalid DATE format", &pos)
                    }
                    c_pos += number.len();
                } else {
                    for temp in number.split('/').into_iter() {
                        let pos = Position::from(c_line, c_pos, temp.len());
                        match temp.chars().filter(|c| *c == '.').count() {
                            0 => {
                                tokens.push(Token {
                                    t: TToken::IntegerLit(temp.parse::<i64>().unwrap()),
                                    pos,
                                });
                            }
                            1 => {
                                tokens.push(Token {
                                    t: TToken::RealLit(temp.parse::<f64>().unwrap()),
                                    pos,
                                });
                            }
                            _ => err("Multiple decimal points are not allowed", &pos),
                        }
                        c_pos += temp.len();
                        tokens.push(Token {
                            t: TToken::Operator("/".to_string()),
                            pos: Position::from(c_line, c_pos, 1),
                        });
                        c_pos += 1;
                    }
                    if number.chars().last().unwrap() != '/' {
                        // remove the extra '/' operator
                        tokens.pop();
                    }
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
                }

                tokens.push(Token {
                    t: match_word(word.clone()),
                    // Point back to start of word
                    pos: Position::from(c_line, c_pos, word.len()),
                });
                c_pos += word.len();
            }
            '/' => {
                if let Some('/') = buf.peek() {
                    // Comment detected, ignore until \n
                    while buf.peek() != Some(&'\n') && buf.peek().is_some() {
                        buf.next();
                    }
                } else {
                    tokens.push(Token {
                        t: TToken::Operator("/".to_string()),
                        pos: Position::from(c_line, c_pos, 1),
                    });
                    c_pos += 1;
                }
            }
            '"' => {
                // String literals
                let line_start = c_line;
                let pos_start = c_pos;
                let mut lit = String::new();
                c_pos += 1;
                while let Some(c) = buf.next() {
                    match c {
                        '"' => break,
                        '\n' => {
                            c_pos = 0;
                            c_line += 1
                        }
                        _ => c_pos += 1,
                    }
                    lit.push(c);
                }
                c_pos += 1;
                tokens.push(Token {
                    t: TToken::StringLit(lit.clone()),
                    pos: Position::new(line_start, c_line, pos_start, c_pos),
                });
            }
            _ => {
                // Symbols
                let mut sym = String::new();
                sym.push(ch);
                // This is to handle symbol "<-, <>, <= and >="
                if ch == '<' || ch == '>' {
                    while let Some(f) = buf.peek() {
                        match f {
                            '-' | '=' | '>' => {
                                sym.push(buf.next().unwrap());
                            }
                            _ => break,
                        }
                    }
                }
                tokens.push(Token {
                    t: match_symbol(sym.clone()),
                    pos: Position::from(c_line, c_pos, sym.len()),
                });
                c_pos += sym.len();
            }
        }
    }

    // EOF token added to signify source file end for error reporting purpose
    tokens.push(Token {
        t: TToken::EOF,
        pos: Position::from(c_line, c_pos, 1),
    });
    c_pos += 1;

    // Second pass to reduce multiple newline and identify array
    let mut temp_tokens = tokens.into_iter().peekable();
    tokens = vec![];
    while temp_tokens.peek().is_some() {
        tokens.push(temp_tokens.next().unwrap());
        match tokens.last().unwrap().clone().t {
            TToken::Newline => {
                while temp_tokens.peek().is_some()
                    && temp_tokens.peek().unwrap().t == TToken::Newline
                {
                    temp_tokens.next();
                }
            }
            _ => (),
        }
    }

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
        _ => TToken::Unknown,
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
        "BOOLEAN" => TToken::VarType(VariableType::Boolean),
        "DATE" => TToken::VarType(VariableType::Date),
        _ => TToken::Identifier(word.to_lowercase()),
    }
}

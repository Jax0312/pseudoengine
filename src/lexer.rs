use std::str::Chars;

pub fn lexer(buf: &mut Chars) -> Vec<Token> {
    let mut temp = String::new();
    let mut line_c = 1;
    let mut col_c = 1;
    let mut tokens = Vec::new();
    let mut ch;
    
    let mut buf = buf.peekable();
    
    while buf.peek().is_some() {
        ch = buf.next().unwrap();
        if ch.is_alphabetic() || ch == '_' {
            // Advance function
            while !ch.is_whitespace() && ch != ',' {
                temp.push(ch);
                match buf.next() {
                    Some(x) => {
                        ch = x;
                        col_c += 1;
                    },
                    None => break
                }
            }
            tokens.push(match_word(temp.clone(), col_c - temp.chars().count(), col_c, line_c));
            if ch == ',' {
                tokens.push(Token {t_type: TokenType::Comma, col_s: col_c, col_e: col_c, line_c: line_c})
            }
            temp.clear();
        } else if ch.is_digit(10) {

            // Handle numeric literal
            while !ch.is_whitespace() {
                temp.push(ch);
                match buf.next() {
                    Some(x) => {
                        ch = x;
                        col_c += 1;
                    },
                    None => break
                }
            }

            match temp.parse::<i64>() {
                Ok(value) => {
                    tokens.push(Token {t_type: TokenType::NumLit(value), col_s: col_c - temp.chars().count(), col_e: col_c, line_c: line_c});
                    temp.clear(); },
                Err(_) => {
                    eprintln!("Error Parsing Number literal");
                    panic!()
                    // Handle Expection
                }
            }

        } else if ch == 0xA as char {
            // Newline
            tokens.push(Token {t_type: TokenType::LineEnd, col_s: col_c, col_e: col_c, line_c: line_c});
            col_c = 1;
            line_c += 1;
        } else if ch == '"' {
            // Handle string literal
            // Handle syntax error exception here
            ch = buf.next().unwrap();
            col_c += 1;
            while ch != '"' {
                temp.push(ch);
                match buf.next() {
                    Some(x) => {
                        ch = x;
                        col_c += 1;
                    },
                    None => break
                }
            }
            tokens.push(Token {t_type: TokenType::StringLit(temp.clone()), col_s: col_c - temp.chars().count(), col_e: col_c, line_c: line_c});
            temp.clear();
        } else if !ch.is_whitespace() {
            // In this case it can only be special characters
            // Advance function
            while !ch.is_whitespace() {
                temp.push(ch);
                match buf.next() {
                    Some(x) => {
                        ch = x;
                        col_c += 1;
                    },
                    None => break
                }
            }
            tokens.push(match_symbol(temp.clone(), col_c, line_c));
            temp.clear();
        }
        col_c += 1;
    }
    
    println!("Token Dump Start\nToken length {}", tokens.len());
    println!("{:?}\nToken Dump End", tokens);
    
    tokens
    
}

fn match_symbol(sign: String, col_s: usize, line_c: usize) -> Token {
    match sign.as_str() {
        "(" => Token {t_type: TokenType::LParen, col_s: col_s, col_e: col_s, line_c: line_c},
        ")" => Token {t_type: TokenType::RParen, col_s: col_s, col_e: col_s, line_c: line_c},
        "+" => Token {t_type: TokenType::Plus, col_s: col_s, col_e: col_s, line_c: line_c},
        "-" => Token {t_type: TokenType::Minus, col_s: col_s, col_e: col_s, line_c: line_c},
        "*" => Token {t_type: TokenType::Star, col_s: col_s, col_e: col_s, line_c: line_c},
        "/" => Token {t_type: TokenType::Slash, col_s: col_s, col_e: col_s, line_c: line_c},
        "&" => Token {t_type: TokenType::Ampersand, col_s: col_s, col_e: col_s, line_c: line_c},
        "<-" => Token {t_type: TokenType::Assignment, col_s: col_s - 1, col_e: col_s, line_c: line_c},
        ":" => Token {t_type: TokenType::Colon, col_s: col_s, col_e: col_s, line_c: line_c},
        "," => Token {t_type: TokenType::Comma, col_s: col_s, col_e: col_s, line_c: line_c},
        "=" => Token {t_type: TokenType::Equal, col_s: col_s, col_e: col_s, line_c: line_c},
        "<>" => Token {t_type: TokenType::NotEqual,col_s: col_s - 1, col_e: col_s, line_c: line_c},
        ">" => Token {t_type: TokenType::Greater, col_s: col_s, col_e: col_s, line_c: line_c},
        "<" => Token {t_type: TokenType::Lesser,  col_s: col_s, col_e: col_s, line_c: line_c},
        ">=" => Token {t_type: TokenType::GreaterEqual,  col_s: col_s - 1, col_e: col_s, line_c: line_c},
        "<=" => Token {t_type: TokenType::LesserEqual,  col_s: col_s - 1, col_e: col_s, line_c: line_c},
        "[" => Token {t_type: TokenType::LSqrBracket,  col_s: col_s, col_e: col_s, line_c: line_c},
        "]" => Token {t_type: TokenType::RSqrBracket, col_s: col_s, col_e: col_s, line_c: line_c},
        _ => Token {t_type: TokenType::Undefined,  col_s: col_s, col_e: col_s, line_c: line_c},
    }
}

fn match_word(word: String, col_s: usize, col_e: usize, line_c: usize) -> Token {
    match word.as_str() {
        "INTEGER" => Token {t_type: TokenType::Integer,  col_s: col_s, col_e: col_e, line_c: line_c},
        "REAL" => Token {t_type: TokenType::Real,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CHAR" => Token {t_type: TokenType::Char,  col_s: col_s, col_e: col_e, line_c: line_c},
        "STRING" => Token {t_type: TokenType::String,  col_s: col_s, col_e: col_e, line_c: line_c},
        "DATE" => Token {t_type: TokenType::Date,  col_s: col_s, col_e: col_e, line_c: line_c},
        "DIV" => Token {t_type: TokenType::Div,  col_s: col_s, col_e: col_e, line_c: line_c},
        "MOD" => Token {t_type: TokenType::Mod,  col_s: col_s, col_e: col_e, line_c: line_c},
        "AND" => Token {t_type: TokenType::And,  col_s: col_s, col_e: col_e, line_c: line_c},
        "OR" => Token {t_type: TokenType::Or,  col_s: col_s, col_e: col_e, line_c: line_c},
        "NOT" => Token {t_type: TokenType::Not,  col_s: col_s, col_e: col_e, line_c: line_c},
        "TRUE" => Token {t_type: TokenType::True,  col_s: col_s, col_e: col_e, line_c: line_c},
        "FALSE" => Token {t_type: TokenType::False,  col_s: col_s, col_e: col_e, line_c: line_c},
        "DECLARE" => Token {t_type: TokenType::Declare,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CONSTANT" => Token {t_type: TokenType::Constant,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ARRAY" => Token {t_type: TokenType::Array,  col_s: col_s, col_e: col_e, line_c: line_c},
        "IF" => Token {t_type: TokenType::If,  col_s: col_s, col_e: col_e, line_c: line_c},
        "THEN" => Token {t_type: TokenType::Then,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ELSE" => Token {t_type: TokenType::Else,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ENDIF" => Token {t_type: TokenType::EndIf,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CASE" => Token {t_type: TokenType::Case,  col_s: col_s, col_e: col_e, line_c: line_c},
        "OF" => Token {t_type: TokenType::Of,  col_s: col_s, col_e: col_e, line_c: line_c},
        "OTHERWISE" => Token {t_type: TokenType::Otherwise,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ENDCASE" => Token {t_type: TokenType::EndCase,  col_s: col_s, col_e: col_e, line_c: line_c},
        "WHILE" => Token {t_type: TokenType::While,  col_s: col_s, col_e: col_e, line_c: line_c},
        "DO" => Token {t_type: TokenType::Do,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ENDWHILE" => Token {t_type: TokenType::EndWhile,  col_s: col_s, col_e: col_e, line_c: line_c},
        "REPEAT" => Token {t_type: TokenType::Repeat,  col_s: col_s, col_e: col_e, line_c: line_c},
        "UNTIL" => Token {t_type: TokenType::Until,  col_s: col_s, col_e: col_e, line_c: line_c},
        "FOR" => Token {t_type: TokenType::For,  col_s: col_s, col_e: col_e, line_c: line_c},
        "TO" => Token {t_type: TokenType::To,  col_s: col_s, col_e: col_e, line_c: line_c},
        "STEP" => Token {t_type: TokenType::Step,  col_s: col_s, col_e: col_e, line_c: line_c},
        "NEXT" => Token {t_type: TokenType::Next,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CONTINUE" => Token {t_type: TokenType::Continue,  col_s: col_s, col_e: col_e, line_c: line_c},
        "PROCEDURE" => Token {t_type: TokenType::Procedure,  col_s: col_s, col_e: col_e, line_c: line_c},
        "BYREF" => Token {t_type: TokenType::ByRef,  col_s: col_s, col_e: col_e, line_c: line_c},
        "BYVAL" => Token {t_type: TokenType::ByValue,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ENDPROCEDURE" => Token {t_type: TokenType::EndProcedure,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CALL" => Token {t_type: TokenType::Call,  col_s: col_s, col_e: col_e, line_c: line_c},
        "FUNCTION" => Token {t_type: TokenType::Function,  col_s: col_s, col_e: col_e, line_c: line_c},
        "ENDFUNCTION" => Token {t_type: TokenType::EndFunction,  col_s: col_s, col_e: col_e, line_c: line_c},
        "RETURNS" => Token {t_type: TokenType::Returns,  col_s: col_s, col_e: col_e, line_c: line_c},
        "RETURN" => Token {t_type: TokenType::Return,  col_s: col_s, col_e: col_e, line_c: line_c},
        "OUTPUT" => Token {t_type: TokenType::Output,  col_s: col_s, col_e: col_e, line_c: line_c},
        "INPUT" => Token {t_type: TokenType::Input,  col_s: col_s, col_e: col_e, line_c: line_c},
        "OPENFILE" => Token {t_type: TokenType::OpenFile,  col_s: col_s, col_e: col_e, line_c: line_c},
        "READFILE" => Token {t_type: TokenType::ReadFile,  col_s: col_s, col_e: col_e, line_c: line_c},
        "WRITEFILE" => Token {t_type: TokenType::WriteFile,  col_s: col_s, col_e: col_e, line_c: line_c},
        "CLOSEFILE" => Token {t_type: TokenType::CloseFile,  col_s: col_s, col_e: col_e, line_c: line_c},
        "READ" => Token {t_type: TokenType::Read,  col_s: col_s, col_e: col_e, line_c: line_c},
        "WRTIE" => Token {t_type: TokenType::Write,  col_s: col_s, col_e: col_e, line_c: line_c},
        "APPEND" => Token {t_type: TokenType::Append,  col_s: col_s, col_e: col_e, line_c: line_c},
        
        _ => Token {t_type: TokenType::Identifier(word), col_s: col_s, col_e: col_e, line_c: line_c}
    }
}

#[derive(Debug)]
pub struct Token {
    pub t_type: TokenType,
    pub col_s: usize,
    pub col_e: usize,
    pub line_c: usize,
}
#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    // DataType
    Integer,
    Real,
    Char,
    String,
    Date,
    // Parenthesese
    LParen,
    RParen,
    // Arithmetic Operators
    Plus,
    Minus,
    Star,
    Slash,
    Div,
    Mod,
    // String Operator
    Ampersand,
    // Assignment Operators
    Assignment,
    Colon,
    Comma,
    // Comparison Operators
    Equal,
    NotEqual,
    Greater,
    Lesser,
    GreaterEqual,
    LesserEqual,
    // Logical Operators
    And,
    Or,
    Not,
    // Boolean Values
    True,
    False,
    // Variables
    Declare,
    Constant,
    Identifier(String),
    // Array
    DataType,
    Array,
    LSqrBracket,
    RSqrBracket,
    // Unknown
    Type,
    EndType,
    Caret,
    Period,
    // Selection Syntax
    If,
    Then,
    Else,
    EndIf,
    Case,
    Of,
    Otherwise,
    EndCase,
    // Loop Syntax
    While,
    Do,
    EndWhile,
    Repeat,
    Until,
    For,
    To,
    Step,
    Next,
    Continue,
    // Procedure Syntax
    Procedure,
    ByRef,
    ByValue,
    EndProcedure,
    Call,
    // Function Syntax
    Function,
    EndFunction,
    Returns,
    Return,
    // I/O
    Output,
    Input,
    // File Access
    OpenFile,
    ReadFile,
    WriteFile,
    CloseFile,
    // Array Operation
    Read,
    Write,
    Append,
    // Unknown
    LineEnd,
    ExpressionEnd,
    
    // Literals
    StringLit(String),
    NumLit(i64),
    
    // Debug Token
    Undefined
    
}
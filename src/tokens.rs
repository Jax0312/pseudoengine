use crate::enums::{Identifier, VariableType};

#[derive(Clone, Debug, PartialEq)]
pub enum TToken {
    // Literals
    BoolLit(bool),
    StringLit(String),
    IntegerLit(i64),
    RealLit(f64),

    // Symbols
    LParen,
    RParen,
    LSqrBracket,
    RSqrBracket,
    Ampersand,
    Assignment,
    Colon,
    Comma,
    Caret,
    Period,

    // Primitive Type
    VarType(VariableType),
    
    // Operators and comparators
    Operator(String),
    
    // Append, write, read, random
    FileMode(String),
    
    PassBy(String),
    
    Array,
    Call,
    Case,
    Class,
    CloseFile,
    Constant,
    Declare,
    Define,
    Else,
    EndCase,
    EndClass,
    EndFunction,
    EndIf,
    EndProcedure,
    EndType,
    EndWhile,
    For,
    Function,
    GetRecord,
    If,
    Inherits,
    Input,
    Next,
    New,
    OpenFile,
    Otherwise,
    Output,
    Of,
    Procedure,
    Private,
    Public,
    PutRecord,
    ReadFile,
    Repeat,
    Return,
    ReturnType,
    To,
    Seek,
    Step,
    Super,
    Then,
    Type,
    Until,
    While,
    WriteFile,
    Newline,
    
    Identifier(Identifier),
    
    Unknown
    
}


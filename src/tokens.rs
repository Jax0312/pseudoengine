use crate::enums::{Node, Token, VariableType};
use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq)]
pub enum TToken {
    // Literals
    BoolLit(bool),
    StringLit(String),
    IntegerLit(i64),
    RealLit(f64),
    DateLit(NaiveDate),

    // Symbols
    LParen,
    RParen,
    LSqrBracket,
    RSqrBracket,
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
    Then,
    Type,
    Until,
    While,
    WriteFile,
    Newline,

    Identifier(String),

    EOF,

    Unknown,
}
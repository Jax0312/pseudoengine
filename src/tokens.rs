use logos::Logos;
use crate::enums::{Comparator, MOperator, VariableType};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\f]+")] // Ignore whitespace and tabs
#[logos(skip r"//.*")] // Skip comment
pub enum Token {
    // Literals
    #[token("FALSE", |_| false)]
    #[token("TRUE", |_| true)]
    BoolLit(bool),
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| lex.slice().to_owned())]
    StringLit(String),
    #[regex("[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    IntegerLit(i64),
    #[regex(r"([0-9]*\.[0-9]+|[0-9]+\.[0-9]*)", |lex| lex.slice().parse::<f64>().unwrap())]
    RealLit(f64),

    // Symbols
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LSqrBracket,
    #[token("]")]
    RSqrBracket,

    #[token("&")]
    Ampersand,
    #[token("<-")]
    Assignment,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,

    #[token("=", |_| Comparator::Eq)]
    #[token("<>", |_| Comparator::Neq)]
    #[token(">=", |_| Comparator::Gte)]
    #[token(">", |_| Comparator::Gt)]
    #[token("<", |_| Comparator::Lt)]
    #[token("<=", |_| Comparator::Lte)]
    Comparator(Comparator),

    #[token("^")]
    Caret,
    #[token(".")]
    Period,

    // Type
    #[token("INTEGER", |_| VariableType::Integer)]
    #[token("REAL", |_| VariableType::Real)]
    #[token("STRING", |_| VariableType::String)]
    #[token("Date", |_| VariableType::Date)]
    VarType(VariableType),
    // Math

    #[token("+", |_| MOperator::Plus)]
    #[token("-", |_| MOperator::Minus)]
    #[token("*", |_| MOperator::Mul)]
    #[token("/", |_| MOperator::Div)]
    #[token("DIV", |_| MOperator::IntDiv)]
    #[token("MOD", |_| MOperator::Mod)]
    MathOp(MOperator),

    // KEYWORDS
    #[token("AND", |_| "and".to_owned())]
    #[token("OR", |_| "or".to_owned())]
    #[token("NOT", |_| "not".to_owned())]
    LogicalCmp(String),

    //  Read write append
    #[token("APPEND", |_| "a".to_owned())]
    #[token("WRITE", |_| "w".to_owned())]
    #[token("READ", |_| "r".to_owned())]
    #[token("RANDOM", |_| "rnd".to_owned())]
    FileMode(String),

    #[token("BYREF", |_| "ref".to_owned())]
    #[token("BYVAL", |_| "val".to_owned())]
    PassBy(String),

    #[token("ARRAY")]
    Array,
    #[token("CALL")]
    Call,
    #[token("CASE OF")]
    CaseOf,
    #[token("CLASS")]
    Class,
    #[token("CLOSEFILE")]
    CloseFile,
    #[token("CONSTANT")]
    Constant,
    #[token("DECLARE")]
    Declare,
    #[token("DEFINE")]
    Define,
    #[token("ELSE")]
    Else,
    #[token("ENDCASE")]
    EndCase,
    #[token("ENDCLASS")]
    EndClass,
    #[token("ENDFUNCTION")]
    EndFunction,
    #[token("ENDIF")]
    EndIf,
    #[token("ENDPROCEDURE")]
    EndProcedure,
    #[token("ENDTYPE")]
    EndType,
    #[token("ENDWHILE")]
    EndWhile,
    #[token("FOR")]
    For,
    #[token("FUNCTION")]
    Function,
    #[token("GETRECORD")]
    GetRecord,
    #[token("IF")]
    If,
    #[token("INHERITS")]
    Inherits,
    #[token("INPUT")]
    Input,
    #[token("NEXT")]
    Next,
    #[token("NEW")]
    New,
    #[token("OPENFILE")]
    OpenFile,
    #[token("OTHERWISE")]
    Otherwise,
    #[token("OUTPUT")]
    Output,
    #[token("OF")]
    Of,
    #[token("PROCEDURE")]
    Procedure,
    #[token("PRIVATE")]
    Private,
    #[token("PUBLIC")]
    Public,
    #[token("PUTRECORD")]
    PutRecord,
    #[token("READFILE")]
    ReadFile,
    #[token("REPEAT")]
    Repeat,
    #[token("RETURN")]
    Return,
    #[token("RETURNS")]
    ReturnType,
    #[token("TO")]
    To,
    #[token("SEEK")]
    Seek,
    #[token("STEP")]
    Step,
    #[token("SUPER")]
    Super,
    #[token("THEN")]
    Then,
    #[token("TYPE")]
    Type,
    #[token("UNTIL")]
    Until,
    #[token("WHILE")]
    While,
    #[token("WRITEFILE")]
    WriteFile,
    #[regex("[\r\n]+")]
    Newline,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
}


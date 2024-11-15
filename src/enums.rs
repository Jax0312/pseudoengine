use crate::tokens::TToken;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Integer,
    Real,
    Char,
    String,
    Date,
    Array(Box<Array>),
    // Composite,
    // ENUM
    // Pointers
}

#[derive(Clone, Debug, PartialEq)]
pub struct Array {
    pub t: Box<VariableType>,
    pub lower: i64,
    pub upper: i64,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub t: TToken,
    pub pos: Position,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Node {
    Main {
        children: Vec<Box<Node>>,
    },
    Int {
        val: i64,
        pos: Position,
    },
    String {
        val: String,
        pos: Position,
    },
    Boolean {
        val: bool,
        pos: Position,
    },
    Real{
        val: f64,
        pos: Position,
    },
    Var {
        name: String,
        pos: Position,
    },
    Reference(Box<Node>),
    Dereference(Box<Node>),
    Composite {
        children: Vec<Box<Node>>,
    },
    Op {
        op: String,
        pos: Position,
    },
    ArrayVar {
        name: String,
        indices: Vec<Box<Node>>,
        pos: Position,
    },
    Declare {
        t: Box<VariableType>,
        // Identifiers
        children: Vec<String>,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    FunctionCall {
        name: String,
        params: Vec<Box<Node>>
    },
    Expression(Vec<Box<Node>>),
    Output {
        children: Vec<Box<Node>>,
    },
    Input {
        child: Box<Node>,
    },
    Null,
}

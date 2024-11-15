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

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub indices: Option<Vec<usize>>,
}

#[derive(Clone, Debug, PartialEq)]
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
    Var(Token),
    // Int(i64),
    // String(String),
    Declare {
        t: Box<VariableType>,
        // Identifiers
        children: Vec<String>,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
    // BinaryExpr {
    //     op: MOperator,
    //     lhs: Box<Node>,
    //     rhs: Box<Node>,
    // },
    Expression(Vec<Token>),
    Output {
        children: Vec<Box<Node>>,
    },
    Input {
        child: Box<Node>,
    },
    Null,
}

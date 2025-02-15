use std::collections::HashMap;

use crate::{executor::Property, tokens::TToken};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Boolean,
    Integer,
    Real,
    Char,
    String,
    Date,
    Array(Box<Array>),
    Custom(String),
    // Pointers
}

#[derive(Clone, Debug, PartialEq)]
pub struct Index {
    pub lower: i64,
    pub upper: i64,
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

impl Position {
    pub fn invalid() -> Position {
        Position {
            line: usize::MAX,
            col: usize::MAX,
        }
    }
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
    Class {
        name: Box<Node>,
        base: Box<Node>,
        children: Vec<Box<Node>>,
    },
    Object {
        // name: String,
        props: HashMap<String, Property>,
    },
    Function {
        name: Box<Node>,
        params: Vec<Box<Node>>,
        return_type: Box<VariableType>,
        children: Vec<Box<Node>>,
    },
    Procedure {
        name: Box<Node>,
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
    },
    Private(Box<Node>),
    CreateObject(Box<Node>),
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
    Real {
        val: f64,
        pos: Position,
    },
    Var {
        name: String,
        pos: Position,
    },
    RefVar(*mut Box<Node>),
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
    Array {
        values: Vec<Box<Node>>,
        shape: Vec<Index>,
        pos: Position,
    },
    Range {
        start: Box<Node>,
        end: Box<Node>,
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
        params: Vec<Box<Node>>,
    },
    While {
        cond: Box<Node>,
        body: Vec<Box<Node>>,
    },
    Repeat {
        cond: Box<Node>,
        body: Vec<Box<Node>>,
    },
    For {
        iter: Box<Node>,
        range: Box<Node>,
        step: Box<Node>,
        body: Vec<Box<Node>>,
    },
    Expression(Vec<Box<Node>>),
    Output {
        children: Vec<Box<Node>>,
    },
    Input {
        child: Box<Node>,
    },
    OpenFile {
        filename: Box<Node>,
        mode: TToken,
    },
    CloseFile(Box<Node>),
    ReadFile {
        filename: Box<Node>,
        var: Box<Node>,
    },
    WriteFile {
        filename: Box<Node>,
        expr: Box<Node>,
    },
    SeekFile {
        filename: Box<Node>,
        expr: Box<Node>,
    },
    PutRecord {
        filename: Box<Node>,
        var: Box<Node>,
    },
    GetRecord {
        filename: Box<Node>,
        var: Box<Node>,
    },
    Return(Box<Node>),
    If {
        cond: Box<Node>,
        true_body: Vec<Box<Node>>,
        false_body: Vec<Box<Node>>,
    },
    Switch {
        cmp: Box<Node>,
        cases: Vec<Box<Node>>,
        otherwise: Vec<Box<Node>>,
    },
    Case {
        expr: Box<Node>,
        children: Vec<Box<Node>>,
    },
    Null,
}

impl Node {
    pub fn val_as_str(&self) -> String {
        match self {
            Node::Int {val, .. } => val.to_string(),
            Node::String { val, .. } => val.clone(),
            Node::Boolean { val, .. } => val.to_string(),
            Node::Real { val, .. } => val.to_string(),
            _ => unimplemented!(),
        }
    }
}
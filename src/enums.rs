use crate::{executor::Property, tokens::TToken};
use chrono::NaiveDate;
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

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
    Pointer(Box<VariableType>),
    Custom(String),
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
    pub line_start: usize,
    pub line_end: usize,
    pub pos_start: usize,
    pub pos_end: usize,
}

impl Position {
    pub fn new(line_start: usize, line_end: usize, pos_start: usize, pos_end: usize) -> Position {
        Position {
            line_start,
            line_end,
            pos_start,
            pos_end,
        }
    }

    pub fn from(line: usize, pos: usize, len: usize) -> Position {
        Position::new(line, line, pos, pos + len)
    }

    pub fn invalid() -> Position {
        Position::new(0, 0, 0, 0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub t: TToken,
    pub pos: Position,
}

pub type NodeRef = Rc<RefCell<Box<Node>>>;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
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
        name: String,
        base: Box<Node>,
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
    Date {
        val: NaiveDate,
        pos: Position,
    },
    EnumVal {
        family: String,
        val: String,
    },
    Var {
        name: String,
        pos: Position,
    },
    Const {
        name: String,
        val: Box<Node>,
        pos: Position,
    },
    RefType {
        name: String,
        ref_to: Box<VariableType>,
    },
    Pointer(NodeRef),
    RefVar(NodeRef),
    Reference(Box<Node>),
    Dereference(Box<Node>),
    Enum {
        name: String,
        variants: Vec<Box<Node>>,
    },
    // Composite type 'Record'
    Record {
        name: String,
        children: Vec<Box<Node>>,
    },
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
        values: Vec<NodeRef>,
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
    NullObject(VariableType),
}

impl Node {
    pub fn val_as_str(&self) -> String {
        match self {
            Node::Int { val, .. } => val.to_string(),
            Node::String { val, .. } => val.clone(),
            Node::Boolean { val, .. } => val.to_string(),
            Node::Real { val, .. } => val.to_string(),
            Node::Date { val, .. } => val.to_string(),
            Node::EnumVal { val, .. } => val.to_string(),
            _ => unimplemented!(),
        }
    }
}

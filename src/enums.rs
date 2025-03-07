use crate::{executor::Property, tokens::TToken};
use chrono::NaiveDate;
use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{format, Display},
    ops::Deref,
    rc::Rc,
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Boolean,
    Integer,
    Real,
    Char,
    String,
    Date,
    Array {
        shape: Vec<Index>,
        t: Box<VariableType>,
    },
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

    pub fn range(start: Position, end: Position) -> Position {
        Position::new(start.line_start, end.line_end, start.pos_start, end.pos_end)
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
        pos: Position,
    },
    Function {
        name: Box<Node>,
        private: bool,
        params: Vec<Box<Node>>,
        return_type: Box<VariableType>,
        children: Vec<Box<Node>>,
        pos: Position,
    },
    Procedure {
        name: Box<Node>,
        private: bool,
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
        pos: Position,
    },
    CreateObject {
        call: Box<Node>,
        pos: Position,
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
    Real {
        val: f64,
        pos: Position,
    },
    Date {
        val: NaiveDate,
        pos: Position,
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
    PointerDef {
        name: String,
        ref_to: Box<VariableType>,
        pos: Position,
    },
    Enum {
        name: String,
        variants: Vec<Box<Node>>,
        pos: Position,
    },
    Record {
        name: Box<Node>,
        children: Vec<Box<Node>>,
        pos: Position,
    },
    Composite {
        children: Vec<Box<Node>>,
        pos: Position,
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
    Range {
        start: Box<Node>,
        end: Box<Node>,
        pos: Position,
    },
    Declare {
        t: Box<VariableType>,
        // Identifiers
        children: Vec<String>,
        byref: bool,
        private: bool,
        pos: Position,
    },
    Assignment {
        lhs: Box<Node>,
        rhs: Box<Node>,
        pos: Position,
    },
    FunctionCall {
        name: String,
        params: Vec<Box<Node>>,
        pos: Position,
    },
    While {
        cond: Box<Node>,
        body: Vec<Box<Node>>,
        pos: Position,
    },
    Repeat {
        cond: Box<Node>,
        body: Vec<Box<Node>>,
        pos: Position,
    },
    For {
        iter: Box<Node>,
        range: Box<Node>,
        step: Box<Node>,
        body: Vec<Box<Node>>,
        pos: Position,
    },
    Unary {
        op: String,
        expr: Box<Node>,
        pos: Position,
    },
    Binary {
        op: String,
        lhs: Box<Node>,
        rhs: Box<Node>,
        pos: Position,
    },
    Output {
        children: Vec<Box<Node>>,
        pos: Position,
    },
    Input {
        child: Box<Node>,
        pos: Position,
    },
    OpenFile {
        filename: Box<Node>,
        mode: TToken,
        pos: Position,
    },
    CloseFile {
        filename: Box<Node>,
        pos: Position,
    },
    ReadFile {
        filename: Box<Node>,
        var: Box<Node>,
        pos: Position,
    },
    WriteFile {
        filename: Box<Node>,
        expr: Box<Node>,
        pos: Position,
    },
    SeekFile {
        filename: Box<Node>,
        expr: Box<Node>,
        pos: Position,
    },
    PutRecord {
        filename: Box<Node>,
        var: Box<Node>,
        pos: Position,
    },
    GetRecord {
        filename: Box<Node>,
        var: Box<Node>,
        pos: Position,
    },
    Return {
        expr: Box<Node>,
        pos: Position,
    },
    Reference {
        expr: Box<Node>,
        pos: Position,
    },
    Dereference {
        expr: Box<Node>,
        pos: Position,
    },
    If {
        cond: Box<Node>,
        true_body: Vec<Box<Node>>,
        false_body: Vec<Box<Node>>,
        pos: Position,
    },
    Switch {
        cmp: Box<Node>,
        cases: Vec<Box<Node>>,
        otherwise: Vec<Box<Node>>,
        pos: Position,
    },
    Case {
        expr: Box<Node>,
        children: Vec<Box<Node>>,
        pos: Position,
    },

    // Interpreter only
    Array {
        t: Box<VariableType>,
        values: Vec<NodeRef>,
        shape: Vec<Index>,
    },
    EnumVal {
        family: String,
        val: String,
    },
    Object {
        name: String,
        base: Box<Node>,
        props: HashMap<String, Property>,
    },
    Pointer(NodeRef),
    RefVar(NodeRef),
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

    pub fn pos(&self) -> Position {
        match &self {
            Node::Unary { pos, .. } => pos.clone(),
            Node::Binary { pos, .. } => pos.clone(),
            Node::Class { pos, .. } => pos.clone(),
            Node::Object { base, .. } => base.pos(),
            Node::Pointer(node_ref) => node_ref.borrow().pos(),
            Node::RefVar(node_ref) => node_ref.borrow().pos(),
            Node::Int { pos, .. } => pos.clone(),
            Node::String { pos, .. } => pos.clone(),
            Node::Boolean { pos, .. } => pos.clone(),
            Node::Real { pos, .. } => pos.clone(),
            Node::Date { pos, .. } => pos.clone(),
            Node::Unary { pos, .. } => pos.clone(),
            Node::Binary { pos, .. } => pos.clone(),
            Node::Function { pos, .. } => pos.clone(),
            Node::Procedure { pos, .. } => pos.clone(),
            Node::CreateObject { pos, .. } => pos.clone(),
            Node::Var { pos, .. } => pos.clone(),
            Node::Const { pos, .. } => pos.clone(),
            Node::PointerDef { pos, .. } => pos.clone(),
            Node::Declare { pos, .. } => pos.clone(),
            Node::Range { pos, .. } => pos.clone(),
            Node::If { pos, .. } => pos.clone(),
            Node::While { pos, .. } => pos.clone(),
            Node::Repeat { pos, .. } => pos.clone(),
            Node::For { pos, .. } => pos.clone(),
            Node::Switch { pos, .. } => pos.clone(),
            Node::Case { pos, .. } => pos.clone(),
            Node::Output { pos, .. } => pos.clone(),
            Node::Input { pos, .. } => pos.clone(),
            Node::OpenFile { pos, .. } => pos.clone(),
            Node::CloseFile { pos, .. } => pos.clone(),
            Node::ReadFile { pos, .. } => pos.clone(),
            Node::WriteFile { pos, .. } => pos.clone(),
            Node::PutRecord { pos, .. } => pos.clone(),
            Node::GetRecord { pos, .. } => pos.clone(),
            Node::Reference { pos, .. } => pos.clone(),
            Node::Dereference { pos, .. } => pos.clone(),
            Node::SeekFile { pos, .. } => pos.clone(),
            Node::Return { pos, .. } => pos.clone(),
            Node::Composite { pos, .. } => pos.clone(),
            Node::ArrayVar { pos, .. } => pos.clone(),
            Node::FunctionCall { pos, .. } => pos.clone(),
            _ => unimplemented!("{:?}", self),
        }
    }
}

impl VariableType {
    pub fn str(&self) -> String {
        match self {
            VariableType::Boolean => "BOOLEAN".to_string(),
            VariableType::Integer => "INTEGER".to_string(),
            VariableType::Real => "REAL".to_string(),
            VariableType::Char => "CHAR".to_string(),
            VariableType::String => "STRING".to_string(),
            VariableType::Date => "DATE".to_string(),
            VariableType::Array { t, .. } => format!("{}[]", t.str()).to_string(),
            VariableType::Pointer(node) => format!("^{}", node.str()).to_string(),
            VariableType::Custom(name) => name.to_string(),
        }
    }
}

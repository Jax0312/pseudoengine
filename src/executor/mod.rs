mod run_expr;
mod run_io;
mod run_stmt;
mod variable;

use std::ops::Deref;

use crate::enums::{Node, VariableType};
use crate::executor::run_stmt::run_stmts;
use crate::executor::variable::Executor;

pub fn run(nodes: Vec<Box<Node>>) {
    let mut executor = Executor::new();

    for node in nodes {
        match *node {
            Node::Main { children } => run_stmts(&mut executor, &children),
            _ => unimplemented!(),
        }
    }
}

pub fn runtime_err(message: String) -> ! {
    println!("Runtime error: {}", message);
    panic!()
}

pub fn var_type_of(node: &Box<Node>) -> VariableType {
    match node.deref() {
        Node::Boolean { .. } => VariableType::Boolean,
        Node::Int { .. } => VariableType::Integer,
        Node::Real { .. } => VariableType::Real,
        Node::String { .. } => VariableType::String,
        _ => unimplemented!(),
    }
}


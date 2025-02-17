mod run_expr;
mod run_io;
mod run_stmt;
mod variable;
mod builtin_func_def;

use std::ops::Deref;

use crate::enums::{Index, Node, Position, VariableType};
use crate::executor::run_stmt::run_stmts;
use crate::executor::variable::Executor;
use variable::Definition;
pub use variable::Property;

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

// Get the VariableType of primitive node
pub fn var_type_of(node: &Box<Node>) -> VariableType {
    match node.deref() {
        Node::Boolean { .. } => VariableType::Boolean,
        Node::Int { .. } => VariableType::Integer,
        Node::Real { .. } => VariableType::Real,
        Node::String { .. } => VariableType::String,
        _ => unimplemented!(),
    }
}

pub fn default_var(executor: &mut Executor, t: &Box<VariableType>) -> Box<Node> {
    Box::new(match t.deref() {
        VariableType::Integer => Node::Int {
            val: 0,
            pos: Position::invalid(),
        },
        VariableType::Real => Node::Real {
            val: 0.0,
            pos: Position::invalid(),
        },
        VariableType::String => Node::String {
            val: String::new(),
            pos: Position::invalid(),
        },
        VariableType::Boolean => Node::Boolean {
            val: false,
            pos: Position::invalid(),
        },
        VariableType::Array(_) => {
            let mut shape = Vec::new();
            let mut capacity = 1;
            let mut inner_t = t.clone();
            while let VariableType::Array(array) = inner_t.deref() {
                shape.push(Index {
                    upper: array.upper,
                    lower: array.lower,
                });
                capacity = capacity * (array.upper - array.lower);
                inner_t = array.t.clone();
            }
            Node::Array {
                values: vec![default_var(executor, &inner_t); capacity as usize],
                shape,
                pos: Position::invalid(),
            }
        }
        VariableType::Custom(name) => match executor.get_def(name) {
            Definition::Class { name, props } => {
                return Box::new(Node::Object { props });
            }
            _ => runtime_err("Invalid type".to_string()),
        },
        _ => unimplemented!(),
    })
}

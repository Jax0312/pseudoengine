mod run_builtins;
mod run_class;
mod run_expr;
mod run_file;
mod run_io;
mod run_stmt;
mod variable;

use crate::enums::{Index, Node, NodeRef, Position, VariableType};
use crate::executor::run_stmt::run_stmts;
pub use crate::executor::variable::Property;
use crate::executor::variable::{Definition, Executor, NodeDeref};
use chrono::NaiveDate;
use std::collections::HashMap;
use std::ops::Deref;

pub fn run(nodes: Vec<Box<Node>>) {
    let mut executor = Executor::new();

    for node in nodes {
        match *node {
            Node::Main { mut children } => run_stmts(&mut executor, &mut children),
            _ => unimplemented!(),
        };
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
        Node::Date { .. } => VariableType::Date,
        Node::EnumVal { family, .. } => VariableType::Custom(family.clone()),
        Node::Pointer(value) => {
            let inner_type = var_type_of(value.borrow().deref());
            VariableType::Pointer(Box::new(inner_type))
        }
        Node::Object { name, .. } => VariableType::Custom(name.clone()),
        Node::NullObject(var_type) => var_type.clone(),
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
        VariableType::Date => Node::Date {
            val: NaiveDate::default(),
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
                // index bounds are inclusive
                capacity = capacity * (array.upper - array.lower + 1);
                inner_t = array.t.clone();
            }
            Node::Array {
                values: vec![NodeRef::new_ref(default_var(executor, &inner_t)); capacity as usize],
                shape,
                pos: Position::invalid(),
            }
        }
        VariableType::Custom(name) => match executor.get_def(name) {
            Definition::Class { props, base, name } => def_base_class(props, base, name),
            Definition::Record { props, name } => {
                let base = Box::new(Node::Null);
                return Box::new(Node::Object { props, base, name });
            }
            Definition::Enum { name } => {
                return Box::from(Node::NullObject(VariableType::Custom((name))))
            }
            Definition::Pointer { ref_to, .. } => {
                return Box::from(Node::NullObject(VariableType::Pointer(ref_to.clone())))
            }
            _ => runtime_err("Invalid type".to_string()),
        },
        _ => unimplemented!(),
    })
}

pub fn def_base_class(
    props: HashMap<String, Property>,
    base: Box<Definition>,
    name: String,
) -> Node {
    let base = Box::new(match base.deref().clone() {
        Definition::Class { props, base, name } => def_base_class(props, base, name),
        _ => Node::Null,
    });
    Node::Object { props, base, name }
}

mod run_builtins;
mod run_class;
mod run_expr;
mod run_file;
mod run_io;
mod run_stmt;
mod variable;

use crate::enums::{Index, Node, NodeRef, Position, VariableType};
use crate::executor::run_stmt::run_stmt;
pub use crate::executor::variable::Property;
use crate::executor::variable::{Definition, Executor, NodeDeref};
use crate::utils::err;
use chrono::NaiveDate;
use std::collections::HashMap;
use std::ops::Deref;

pub fn run(nodes: Vec<Box<Node>>) {
    let mut executor = Executor::new();

    for node in nodes {
        match *node {
            Node::Main { mut children } => {
                for node in children {
                    if let Node::Return { pos, .. } = *node {
                        err("Cannot return outside of function or procedure", &pos)
                    }
                    run_stmt(&mut executor, &node);
                }
            }
            _ => unimplemented!(),
        };
    }
}

// Get the VariableType of primitive node
pub fn var_type_of(node: &Box<Node>) -> VariableType {
    match node.deref().clone() {
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
        Node::RefVar(value) => var_type_of(value.borrow().deref()),
        Node::Object { name, .. } => VariableType::Custom(name.clone()),
        Node::Array { t, shape, .. } => VariableType::Array { shape, t },
        Node::NullObject(var_type) => var_type.clone(),
        _ => unimplemented!("{:?}", node),
    }
}

pub fn default_var(executor: &mut Executor, t: &Box<VariableType>, pos: &Position) -> Box<Node> {
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
        VariableType::Array { shape, t } => {
            let mut capacity = 1;
            for index in shape {
                // index bounds are inclusive
                capacity = capacity * (index.upper - index.lower + 1);
            }
            Node::Array {
                values: vec![NodeRef::new_ref(default_var(executor, &t, pos)); capacity as usize],
                shape: shape.clone(),
                t: t.clone(),
            }
        }
        VariableType::Custom(name) => match executor.get_def(name, pos) {
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
            _ => unreachable!(),
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

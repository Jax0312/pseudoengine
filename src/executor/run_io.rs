use std::ops::Deref;

use crate::enums::{Node, Position, VariableType};

use crate::executor::run_expr::run_expr;
use crate::executor::runtime_err;
use crate::executor::variable::Executor;

use super::run_class::run_access_mut;
use super::var_type_of;

pub fn run_output(executor: &mut Executor, exprs: &Vec<Box<Node>>) {
    for expr in exprs {
        let res = run_expr(executor, expr);
        match *res {
            Node::Int { val, .. } => print!("{}", val.to_string()),
            Node::Real { val, .. } => print!("{}", val.to_string()),
            Node::String { val, .. } => print!("{}", val),
            Node::Boolean { val, .. } => print!("{}", val.to_string().to_uppercase()),
            Node::Date { val, .. } => print!("{}", val.format("%d-%m-%Y").to_string()),
            Node::Null => print!("null"),
            Node::EnumVal { val, .. } => print!("{}", val),
            _ => unimplemented!(),
        }
    }
    print!("\n")
}

pub fn run_input(executor: &mut Executor, child: &Box<Node>) {
    match child.deref() {
        Node::Var { .. } | Node::ArrayVar { .. } | Node::Composite { .. } | Node::Dereference(_) => {}
        _ => runtime_err("Invalid input statement".to_string()),
    };
    let node = run_access_mut(executor, child);
    let mut temp = String::new();
    std::io::stdin()
        .read_line(&mut temp)
        .expect("Failed to read input");
    temp = temp
        .strip_suffix("\r\n")
        .or(temp.strip_suffix("\n"))
        .unwrap()
        .parse()
        .unwrap();
    let node = match var_type_of(node.borrow().deref()) {
        VariableType::Integer => Box::new(Node::Int {
            val: temp.parse::<i64>().expect("Invalid INTEGER value input"),
            pos: Position::invalid(),
        }),
        VariableType::String => Box::new(Node::String {
            val: temp,
            pos: Position::invalid(),
        }),
        VariableType::Real => Box::new(Node::Real {
            val: temp.parse::<f64>().expect("Invalid REAL value input"),
            pos: Position::invalid(),
        }),
        VariableType::Boolean => Box::new(Node::Boolean {
            val: temp.to_lowercase() == "TRUE",
            pos: Position::invalid(),
        }),
        _ => runtime_err("Invalid type".to_string()),
    };
}

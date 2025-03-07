use std::ops::Deref;

use crate::enums::{Node, Position, VariableType};

use crate::executor::run_expr::run_expr;
use crate::executor::variable::Executor;
use crate::utils::err;

use super::run_class::run_access_mut;
use super::var_type_of;

pub fn run_output(executor: &mut Executor, exprs: &Vec<Box<Node>>) {
    for expr in exprs {
        let pos = expr.pos();
        let res = run_expr(executor, expr);
        match *res {
            Node::Int { val, .. } => print!("{}", val.to_string()),
            Node::Real { val, .. } => print!("{}", val.to_string()),
            Node::String { val, .. } => print!("{}", val),
            Node::Boolean { val, .. } => print!("{}", val.to_string().to_uppercase()),
            Node::Date { val, .. } => print!("{}", val.format("%d-%m-%Y").to_string()),
            Node::Null => print!("null"),
            Node::EnumVal { val, .. } => print!("{}", val),
            _ => err("Value cannot be printed", &pos),
        }
    }
    print!("\n")
}

pub fn run_input(executor: &mut Executor, child: &Box<Node>, pos: &Position) {
    match child.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Reference { .. } => {}
        _ => err("Input cannot be stored in this value", &child.pos()),
    };
    let node = run_access_mut(executor, child);
    let mut temp = String::new();
    std::io::stdin()
        .read_line(&mut temp)
        .unwrap_or_else(|_| err("System error, failed to read input", pos));
    temp = temp
        .strip_suffix("\r\n")
        .or(temp.strip_suffix("\n"))
        .unwrap()
        .parse()
        .unwrap();
    let var_type = var_type_of(node.borrow().deref());
    let node = match var_type {
        VariableType::Integer => Box::new(Node::Int {
            val: temp
                .parse::<i64>()
                .unwrap_or_else(|_| err("Input value is not INTEGER", pos)),
            pos: Position::invalid(),
        }),
        VariableType::String => Box::new(Node::String {
            val: temp,
            pos: Position::invalid(),
        }),
        VariableType::Real => Box::new(Node::Real {
            val: temp
                .parse::<f64>()
                .unwrap_or_else(|_| err("Input value is not REAL", pos)),
            pos: Position::invalid(),
        }),
        VariableType::Boolean => Box::new(Node::Boolean {
            val: temp.to_lowercase() == "TRUE",
            pos: Position::invalid(),
        }),
        _ => err(format!("Input type {} is not allowed", var_type.str()).as_str(), &child.pos()),
    };
}

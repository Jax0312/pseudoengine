use std::ops::Deref;

use super::variable::Executor;
use crate::enums::{Node, Position};
use crate::executor::runtime_err;

pub fn run_expr(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    let mut stack = Vec::<Box<Node>>::new();

    if let Node::Expression(exprs) = node.deref() {
        for expr in exprs {
            match expr.deref() {
                Node::Op { op, .. } => run_op(executor, &mut stack, op),
                Node::Var { name, .. } => run_var(executor, &mut stack, name),
                _ => stack.push(expr.clone()),
            }
        }
    }

    return stack.first().unwrap().clone();
}

fn run_var(executor: &mut Executor, stack: &mut Vec<Box<Node>>, name: &String) {
    stack.push(executor.get_var(name).value.clone())
}

fn run_op(executor: &mut Executor, stack: &mut Vec<Box<Node>>, op: &String) {
    let op = op.as_str();
    match op {
        "+" | "-" | "*" | "/" | "%" | "//" | "_+" | "_-" => run_arithmetic_op(executor, stack, op),
        "<" | ">" | "<=" | ">=" => run_comparison_op(executor, stack, op),
        _ => unimplemented!(),
    }
}

fn run_arithmetic_op(executor: &mut Executor, stack: &mut Vec<Box<Node>>, op: &str) {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
    let (lhs_val, lhs_real) = assert_number(&lhs);
    let (rhs_val, rhs_real) = assert_number(&rhs);

    let res = match op {
        "+" => lhs_val + rhs_val,
        "-" => lhs_val - rhs_val,
        "*" => lhs_val * rhs_val,
        "/" => lhs_val / rhs_val,
        "%" => lhs_val % rhs_val,
        "//" => (lhs_val / rhs_val).floor(),
        _ => unreachable!(),
    };

    stack.push(if lhs_real || rhs_real {
        Box::from(Node::Real {
            val: res,
            pos: Position::invalid(),
        })
    } else {
        Box::from(Node::Int {
            val: res.trunc() as i64,
            pos: Position::invalid(),
        })
    })
}

fn run_comparison_op(executor: &mut Executor, stack: &mut Vec<Box<Node>>, op: &str) {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
    let (lhs_val, _) = assert_number(&lhs);
    let (rhs_val, _) = assert_number(&rhs);

    let res = match op {
        ">" => lhs_val > rhs_val,
        "<" => lhs_val < rhs_val,
        ">=" => lhs_val >= rhs_val,
        "<=" => lhs_val <= rhs_val,
        _ => unreachable!(),
    };

    stack.push(Box::from(Node::Boolean {
        val: res,
        pos: Position::invalid(),
    }))
}

fn assert_number(node: &Box<Node>) -> (f64, bool) {
    match *node.deref() {
        Node::Int { val, .. } => (val as f64, false),
        Node::Real { val, .. } => (val, true),
        _ => runtime_err("Invalid type".to_string()),
    }
}

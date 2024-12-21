use std::ops::Deref;

use crate::enums::{Node, Position, VariableType};
use crate::executor::run_expr::run_expr;
use crate::executor::run_io::run_output;
use crate::executor::runtime_err;
use crate::executor::variable::{Executor, Variable};

pub fn run_stmts(executor: &mut Executor, nodes: &Vec<Box<Node>>) {
    for node in nodes {
        run_stmt(executor, &node)
    }
}

fn run_stmt(executor: &mut Executor, node: &Node) {
    match node {
        Node::Declare { t, children } => run_declare(executor, children, t),
        Node::If {
            cond,
            true_body,
            false_body,
        } => run_if(executor, cond, true_body, false_body),
        Node::While { cond, body } => run_while(executor, cond, body),
        Node::For {
            iter,
            range,
            step,
            body,
        } => run_for(executor, iter, range, step, body),
        Node::Output { children } => run_output(executor, children),
        Node::Null => (),
        _ => unimplemented!(),
    }
}

fn run_declare(executor: &mut Executor, identifiers: &[String], vtype: &Box<VariableType>) {
    for identifier in identifiers {
        let value = match vtype.deref() {
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
            _ => unimplemented!(),
        };
        executor.declare(identifier, Box::new(value), vtype);
    }
}

fn run_if(
    executor: &mut Executor,
    cond: &Box<Node>,
    true_body: &Vec<Box<Node>>,
    false_body: &Vec<Box<Node>>,
) {
    if as_boolean_expr(executor, cond) {
        run_stmts(executor, true_body)
    } else {
        run_stmts(executor, false_body)
    };
}

fn run_while(executor: &mut Executor, cond: &Box<Node>, body: &Vec<Box<Node>>) {
    loop {
        if as_boolean_expr(executor, cond) {
            run_stmts(executor, body)
        } else {
            break;
        }
    }
}

fn run_for(
    executor: &mut Executor,
    iter: &Box<Node>,
    range: &Box<Node>,
    step: &Box<Node>,
    body: &Vec<Box<Node>>,
) {
    if let Node::Var { name, .. } = iter.deref() {
        if let Node::Range { start, end, .. } = range.deref() {
            let mut start = as_number_expr(executor, start);
            let end = as_number_expr(executor, end);
            let step = if let Node::Null = *step.deref() {
                1
            } else {
                as_number_expr(executor, step)
            };
            executor.declare(
                name,
                Box::new(Node::Int {
                    val: start,
                    pos: Position::invalid(),
                }),
                &Box::new(VariableType::Integer),
            );
            while start <= end {
                executor.set_var(
                    name,
                    Box::new(Node::Int {
                        val: start,
                        pos: Position::invalid(),
                    }),
                );
                start += step;
                run_stmts(executor, &body);
            }
            return;
        };
    }
    runtime_err("Invalid for statement".to_string())
}

fn as_number_expr(executor: &mut Executor, node: &Box<Node>) -> i64 {
    let expr = run_expr(executor, node);
    return match *expr.deref() {
        Node::Int { val, pos } => val,
        Node::Real { val, pos } => val.trunc() as i64,
        _ => runtime_err("Invalid type".to_string()),
    };
}

fn as_boolean_expr(executor: &mut Executor, node: &Box<Node>) -> bool {
    let expr = run_expr(executor, node);
    if let Node::Boolean { val, .. } = expr.deref() {
        return val.clone();
    };
    runtime_err("Invalid type".to_string())
}

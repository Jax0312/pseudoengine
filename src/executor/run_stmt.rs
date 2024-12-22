use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, Position, VariableType};
use crate::executor::run_expr::run_expr;
use crate::executor::run_io::{run_input, run_output};
use crate::executor::runtime_err;
use crate::executor::variable::Executor;

use super::var_type_of;

pub fn run_stmts(executor: &mut Executor, nodes: &Vec<Box<Node>>) {
    for node in nodes {
        run_stmt(executor, node)
    }
}

pub fn run_stmt(executor: &mut Executor, node: &Box<Node>) {
    match node.deref() {
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
        Node::Input { child } => run_input(executor, child),
        Node::Function { name, .. } => run_function(executor, name, node),
        Node::Expression(_) => {
            run_expr(executor, node);
        }
        Node::Assignment { lhs, rhs } => run_assign(executor, lhs, rhs),
        Node::Null => (),
        _ => unimplemented!(),
    }
}

fn run_function(executor: &mut Executor, identifier: &Box<Node>, node: &Box<Node>) {
    if let Node::String { val, .. } = identifier.deref() {
        return executor.declare_fn(val, node);
    }
    runtime_err("Invalid function declaration".to_string())
}

fn run_declare(executor: &mut Executor, identifiers: &[String], t: &Box<VariableType>) {
    for identifier in identifiers {
        let value = match t.deref() {
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
            VariableType::Array(_) => {
                let mut indices = Vec::new();
                let mut capacity = 1;
                let mut inner_t = t.clone();
                while let VariableType::Array(array) = inner_t.deref() {
                    indices.push(Index {
                        upper: array.upper,
                        lower: array.lower,
                    });
                    capacity = capacity * (array.upper - array.lower);
                    inner_t = array.t.clone();
                }
                Node::Array {
                    values: vec![Box::new(Node::Null); capacity as usize],
                    indices,
                    pos: Position::invalid(),
                }
            }
            _ => unimplemented!(),
        };
        executor.declare_var(identifier, Box::new(value), t);
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

fn run_assign(executor: &mut Executor, lhs: &Box<Node>, rhs: &Box<Node>) {
    if let Node::Var { name, .. } = lhs.deref() {
        let t = executor.get_var(name).t.clone();
        let expr = run_expr(executor, rhs);
        if var_type_of(&expr) == t {
            return executor.set_var(name, expr);
        }
        runtime_err("Mismatched types".to_string());
    } else if let Node::ArrayVar { name, indices, .. } = lhs.deref() {
        let index_exprs = indices
            .iter()
            .map(|index| as_number_expr(executor, index))
            .collect::<Vec<i64>>();
        let expr = run_expr(executor, rhs);
        let node = &mut executor.get_var_mut(name).value;
        if let Node::Array {
            values,
            indices: array_indices,
            ..
        } = node.deref_mut()
        {
            let mut size = 1;
            let mut total_index = 0;
            if indices.len() != array_indices.len() {
                runtime_err("Missing indices".to_string())
            }
            for (array_index, index_expr) in array_indices.iter().zip(index_exprs).rev() {
                total_index += index_expr * size;
                size = size * (array_index.upper - array_index.lower);
            }
            return values[total_index as usize] = expr;
        };
    }
    runtime_err("Invalid assign statement".to_string());
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
            executor.declare_var(
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

pub fn as_number_expr(executor: &mut Executor, node: &Box<Node>) -> i64 {
    let expr = run_expr(executor, node);
    return match *expr.deref() {
        Node::Int { val, .. } => val,
        _ => runtime_err("Invalid type".to_string()),
    };
}

pub fn as_boolean_expr(executor: &mut Executor, node: &Box<Node>) -> bool {
    let expr = run_expr(executor, node);
    if let Node::Boolean { val, .. } = expr.deref() {
        return val.clone();
    };
    runtime_err("Invalid type".to_string())
}

use std::ops::Deref;

use crate::enums::{Node, Position};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::variable::Executor;
use crate::executor::{runtime_err, var_type_of};

pub fn run_expr(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    let mut stack = Vec::<Box<Node>>::new();

    if let Node::Expression(exprs) = node.deref() {
        for expr in exprs {
            match expr.deref() {
                Node::Op { op, .. } => run_op(&mut stack, op),
                Node::Var { name, .. } => run_var(executor, &mut stack, name),
                Node::FunctionCall { name, params } => {
                    run_fn_call(executor, &mut stack, name, params)
                }
                Node::ArrayVar { name, indices, .. } => {
                    run_array_var(executor, &mut stack, name, indices)
                }
                _ => stack.push(expr.clone()),
            }
        }
    }

    return stack.first().unwrap().clone();
}

fn run_array_var(
    executor: &mut Executor,
    stack: &mut Vec<Box<Node>>,
    name: &String,
    indices: &Vec<Box<Node>>,
) {
    let index_exprs = indices
        .iter()
        .map(|index| as_number_expr(executor, index))
        .collect::<Vec<i64>>();
    let node = executor.get_var(name).value.clone();
    if let Node::Array {
        values, indices, ..
    } = node.deref()
    {
        let mut size = 1;
        let mut total_index = 0;
        if indices.len() != index_exprs.len() {
            runtime_err("Missing indices".to_string())
        }
        for (index_expr, index) in index_exprs.iter().zip(indices).rev() {
            total_index += index_expr * size;
            size = size * (index.upper - index.lower);
        }
        return stack.push(values[total_index as usize].clone());
    };
    runtime_err("Invalid array access".to_string())
}

fn run_var(executor: &mut Executor, stack: &mut Vec<Box<Node>>, name: &String) {
    stack.push(executor.get_var(name).value.clone())
}

fn run_op(stack: &mut Vec<Box<Node>>, op: &String) {
    let op = op.as_str();
    match op {
        "+" | "-" | "*" | "/" | "%" | "//" | "_+" | "_-" => run_arithmetic_op(stack, op),
        "<" | ">" | "<=" | ">=" => run_comparison_op(stack, op),
        _ => unimplemented!(),
    }
}

fn run_arithmetic_op(stack: &mut Vec<Box<Node>>, op: &str) {
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

fn run_comparison_op(stack: &mut Vec<Box<Node>>, op: &str) {
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

fn run_fn_call(
    executor: &mut Executor,
    stack: &mut Vec<Box<Node>>,
    name: &String,
    call_params: &Vec<Box<Node>>,
) {
    if let Node::Function {
        params, children, ..
    } = executor.get_fn(name).deref()
    {
        executor.enter_scope();
        if call_params.len() != params.len() {
            runtime_err("Missing parameters".to_string())
        }
        for (param, call_param) in params.iter().zip(call_params) {
            if let Node::Declare { t, children } = param.deref() {
                let expr = run_expr(executor, call_param);
                if var_type_of(&expr) == *t.deref() {
                    executor.declare_var(&children[0], expr, t);
                } else {
                    runtime_err("Mismatched parameters".to_string())
                }
            }
        }
        for child in children {
            match child.deref() {
                Node::Return(expr) => {
                    stack.push(run_expr(executor, expr));
                }
                _ => run_stmt(executor, child),
            }
        }
        return executor.exit_scope();
    }
    runtime_err("Invalid function call".to_string())
}

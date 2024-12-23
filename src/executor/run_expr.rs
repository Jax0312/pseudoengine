use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::enums::{Index, Node, Position};
use crate::executor::run_stmt::{as_number_expr, run_stmt, run_stmts};
use crate::executor::variable::{Definition, Executor, Property, Scope, Variable};
use crate::executor::{default_var, runtime_err, var_type_of};

pub fn run_expr(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::Expression(exprs) = node.deref() {
        let mut stack = Vec::<Box<Node>>::new();
        for expr in exprs {
            let expr = match expr.deref() {
                Node::Op { op, .. } => run_op(&mut stack, op),
                Node::Var { name, .. } => run_var(executor, name),
                Node::FunctionCall { name, params } => run_fn_call(executor, name, params),
                Node::ArrayVar { name, indices, .. } => run_array_var(executor, name, indices),
                Node::CreateObject(object) => run_create_obj(executor, expr),
                Node::RefVar(node) => run_ref_var(executor, node),
                Node::Composite { children } => run_composite(executor, children),
                _ => expr.clone(),
            };
            stack.push(expr);
        }
        return stack.first().unwrap().clone();
    } else if let Node::CreateObject(object) = node.deref() {
        return run_create_obj(executor, object);
    }
    unimplemented!()
}

fn run_array_var(executor: &mut Executor, name: &String, indices: &Vec<Box<Node>>) -> Box<Node> {
    let node = executor.get_var(name).value.clone();
    run_array_var_inner(executor, &node, indices)
}

fn run_array_var_inner(
    executor: &mut Executor,
    node: &Box<Node>,
    indices: &Vec<Box<Node>>,
) -> Box<Node> {
    let indices = indices
        .iter()
        .map(|index| as_number_expr(executor, index))
        .collect::<Vec<i64>>();
    if let Node::Array { values, shape, .. } = node.deref() {
        return values[get_array_index(indices, shape)].clone();
    };
    runtime_err("Invalid array access".to_string())
}

pub fn get_array_index(indices: Vec<i64>, shape: &Vec<Index>) -> usize {
    let mut size = 1;
    let mut total_index = 0;
    if indices.len() != shape.len() {
        runtime_err("Missing indices".to_string())
    }
    for (shape, index) in shape.iter().zip(indices).rev() {
        total_index += index * size;
        size = size * (shape.upper - shape.lower);
    }
    total_index as usize
}

fn run_var(executor: &mut Executor, name: &String) -> Box<Node> {
    executor.get_var(name).value.clone()
}

fn run_ref_var(executor: &mut Executor, node: &*mut Box<Node>) -> Box<Node> {
    unsafe { (**node).clone() }
}

fn run_op(stack: &mut Vec<Box<Node>>, op: &String) -> Box<Node> {
    let op = op.as_str();
    match op {
        "+" | "-" | "*" | "/" | "%" | "//" | "_+" | "_-" => run_arithmetic_op(stack, op),
        "<" | ">" | "<=" | ">=" => run_comparison_op(stack, op),
        _ => unimplemented!(),
    }
}

fn run_arithmetic_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
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

    if lhs_real || rhs_real {
        Box::from(Node::Real {
            val: res,
            pos: Position::invalid(),
        })
    } else {
        Box::from(Node::Int {
            val: res.trunc() as i64,
            pos: Position::invalid(),
        })
    }
}

fn run_comparison_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
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

    Box::from(Node::Boolean {
        val: res,
        pos: Position::invalid(),
    })
}

fn assert_number(node: &Box<Node>) -> (f64, bool) {
    match *node.deref() {
        Node::Int { val, .. } => (val as f64, false),
        Node::Real { val, .. } => (val, true),
        _ => runtime_err("Invalid type".to_string()),
    }
}

fn run_fn_call(executor: &mut Executor, name: &String, call_params: &Vec<Box<Node>>) -> Box<Node> {
    if let Definition::Function { params, children } = executor.get_def(name) {
        return run_fn_call_inner(executor, call_params, &params, &children, true);
    }
    runtime_err("Invalid function call".to_string())
}

fn run_fn_call_inner(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    returns: bool,
) -> Box<Node> {
    executor.enter_scope();
    if fn_params.len() != call_params.len() {
        runtime_err("Missing parameters".to_string())
    }
    for (call_param, fn_param) in call_params.iter().zip(fn_params) {
        if let Node::Declare { t, children } = fn_param.deref() {
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
                let expr = run_expr(executor, expr);
                executor.exit_scope();
                return expr;
            }
            _ => run_stmt(executor, &child),
        }
    }
    if returns {
        runtime_err("Missing return statement".to_string())
    } else {
        executor.exit_scope();
        Box::new(Node::Null)
    }
}

fn run_create_obj(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::FunctionCall { params, name } = node.deref() {
        if let Definition::Class { props, .. } = executor.get_def(name) {
            if let Some(Property::Procedure {
                private,
                params: fn_params,
                children,
            }) = props.get("new")
            {
                if !private {
                    executor.enter_scope();
                    for (name, prop) in props.iter() {
                        if let Property::Var { value, t, .. } = prop {
                            executor.declare_var(name, value.clone(), t);
                        }
                    }
                    run_fn_call_inner(executor, params, fn_params, children, false);
                    let mut props = props.clone();
                    for (name, prop) in props.iter_mut() {
                        if let Property::Var { value, .. } = prop {
                            *value = executor.get_var(name).value.clone()
                        }
                    }
                    executor.exit_scope();
                    return Box::new(Node::Object { props });
                }
                runtime_err("Constructor cannot be private".to_string())
            }
            runtime_err("Constructor is not defined".to_string())
        }
        runtime_err(format!("{} is not a class", name))
    }
    unreachable!()
}

fn run_composite(executor: &mut Executor, children: &Vec<Box<Node>>) -> Box<Node> {
    let mut base = match children[0].deref() {
        Node::Var { name, .. } => run_var(executor, name),
        Node::FunctionCall { name, params } => run_fn_call(executor, name, params),
        Node::RefVar(node) => run_ref_var(executor, node),
        Node::ArrayVar { name, indices, .. } => run_array_var(executor, name, indices),
        _ => runtime_err("Invalid base property access".to_string()),
    };
    for child in children.iter().skip(1) {
        if let Node::Object { props, .. } = base.deref_mut() {
            base = match child.deref() {
                Node::Var { name, .. } => run_prop_access(executor, name, props),
                Node::ArrayVar { name, indices, .. } => {
                    run_prop_arr_access(executor, name, indices, props)
                }
                Node::FunctionCall { name, params } => {
                    run_method_call(executor, name, params, props)
                }
                _ => runtime_err("Invalid property access".to_string()),
            };
        } else {
            runtime_err("Invalid property access".to_string())
        }
    }
    base
}

fn run_prop_access(
    executor: &mut Executor,
    name: &String,
    props: &HashMap<String, Property>,
) -> Box<Node> {
    if let Some(Property::Var { value, .. }) = props.get(name) {
        return value.clone();
    }
    runtime_err("Invalid property access".to_string())
}

fn run_prop_arr_access(
    executor: &mut Executor,
    name: &String,
    indices: &Vec<Box<Node>>,
    props: &HashMap<String, Property>,
) -> Box<Node> {
    if let Some(Property::Var { value, .. }) = props.get(name) {
        return run_array_var_inner(executor, value, indices);
    }
    runtime_err("Invalid property access".to_string())
}

fn run_method_call(
    executor: &mut Executor,
    name: &String,
    call_params: &Vec<Box<Node>>,
    props: &mut HashMap<String, Property>,
) -> Box<Node> {
    executor.enter_scope();
    for (name, prop) in props.iter_mut() {
        if let Property::Var { value, t, .. } = prop {
            let value = Box::new(Node::RefVar(value as *mut Box<Node>));
            executor.declare_var(name, value, t);
        }
    }
    if let Some(Property::Procedure {
        params: fn_params,
        children,
        ..
    }) = props.get(name)
    {
        run_fn_call_inner(executor, call_params, fn_params, children, false);
        executor.exit_scope();
        return Box::new(Node::Null);
    }
    runtime_err("Invalid property access".to_string())
}

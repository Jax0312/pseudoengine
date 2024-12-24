use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, Position, VariableType};
use crate::executor::run_expr::run_expr;
use crate::executor::run_io::{run_input, run_output};
use crate::executor::runtime_err;
use crate::executor::variable::{Definition, Executor, Property};

use super::run_expr::get_array_index;
use super::variable::Variable;
use super::{default_var, var_type_of};

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
        Node::Function {
            name,
            params,
            children,
            ..
        } => run_function(executor, name, params, children),
        Node::Class {
            name,
            base,
            children,
        } => run_class(executor, name, base, children),
        Node::Expression(_) => {
            run_expr(executor, node);
        }
        Node::Assignment { lhs, rhs } => run_assign(executor, lhs, rhs),
        Node::Null => (),
        _ => unimplemented!(),
    }
}

fn run_function(
    executor: &mut Executor,
    identifier: &Box<Node>,
    params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
) {
    if let Node::String { val, .. } = identifier.deref() {
        return executor.declare_def(
            val,
            Definition::Function {
                params: params.clone(),
                children: children.clone(),
            },
        );
    }
    runtime_err("Invalid function declaration".to_string())
}

fn run_class(
    executor: &mut Executor,
    name: &Box<Node>,
    base: &Box<Node>,
    children: &Vec<Box<Node>>,
) {
    let mut props = HashMap::new();
    for node in children.clone() {
        match node.deref() {
            Node::Null => (),
            _ => {
                let (name, prop) = run_class_prop(executor, &node, false);
                props.insert(name, prop);
            }
        }
    }
    if let Node::String { val, .. } = name.deref() {
        return executor.declare_def(
            val,
            Definition::Class {
                name: val.clone(),
                props,
            },
        );
    }
    runtime_err("Invalid function declaration".to_string())
}

fn run_class_prop(executor: &mut Executor, prop: &Box<Node>, private: bool) -> (String, Property) {
    match prop.deref() {
        Node::Procedure {
            name,
            params,
            children,
        } => {
            if let Node::String { val, .. } = name.deref() {
                return (
                    val.clone(),
                    Property::Procedure {
                        private,
                        params: params.clone(),
                        children: children.clone(),
                    },
                );
            }
            unreachable!()
        }
        Node::Declare { children, t } => {
            return (
                children[0].clone(),
                Property::Var {
                    private,
                    value: default_var(executor, t),
                    t: t.clone(),
                },
            );
        }
        Node::Private(node) => return run_class_prop(executor, node, true),
        _ => runtime_err("Invalid class declaration".to_string()),
    }
}

fn run_declare(executor: &mut Executor, identifiers: &[String], t: &Box<VariableType>) {
    for identifier in identifiers {
        let value = default_var(executor, t);
        executor.declare_var(identifier, value, t);
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
    let rhs = run_expr(executor, rhs);
    match lhs.deref() {
        Node::Var { name, .. } => {
            executor.get_var_mut(name).value = rhs;
        }
        Node::ArrayVar { name, indices, .. } => {
            let indices = indices
                .iter()
                .map(|index| as_number_expr(executor, index))
                .collect::<Vec<i64>>();
            let node = &mut executor.get_var_mut(name).value;
            let var = run_array_access(node, indices);
            *var = rhs;
        }
        Node::RefVar(var) => unsafe { **var = rhs },
        Node::Composite { children } => {
            let mut children = children.clone();
            for child in children.iter_mut() {
                if let Node::ArrayVar { name, indices, .. } = child.deref_mut() {
                    for index in indices {
                        *index = Box::new(Node::Int {
                            val: as_number_expr(executor, index),
                            pos: Position::invalid(),
                        });
                    }
                }
            }
            let mut index = 0;
            let mut base = match children[index].deref() {
                Node::Var { name, .. } => &mut executor.get_var_mut(name).value,
                Node::ArrayVar { name, indices, .. } => {
                    let node = &mut executor.get_var_mut(name).value;
                    run_array_access(node, as_int_exprs(indices))
                }
                Node::RefVar(var) => unsafe { &mut *var.clone() as &mut Box<Node> },
                _ => runtime_err("Invalid assign statement".to_string()),
            };
            let var = run_assign_inner(&mut base, &mut children, &mut index);
            *var = rhs;
        }
        _ => runtime_err("Invalid assign statement".to_string()),
    };
}

fn run_assign_inner<'a>(
    base: &'a mut Box<Node>,
    children: &mut Vec<Box<Node>>,
    index: &mut usize,
) -> &'a mut Box<Node> {
    *index += 1;
    if let Node::Object { props, .. } = base.deref_mut() {
        let base = match children[*index].deref() {
            Node::Var { name, .. } => {
                if let Some(Property::Var { value, .. }) = props.get_mut(name) {
                    value
                } else {
                    runtime_err("Invalid property access".to_string());
                }
            }
            Node::ArrayVar { name, indices, .. } => {
                if let Some(Property::Var { value, .. }) = props.get_mut(name) {
                    run_array_access(value, as_int_exprs(indices))
                } else {
                    runtime_err("Invalid property access".to_string());
                }
            }
            _ => runtime_err("Invalid property access".to_string()),
        };
        if *index == children.len() - 1 {
            return base;
        } else {
            return run_assign_inner(base, children, index);
        }
    }
    runtime_err("Invalid property access".to_string());
}

fn run_array_access<'a>(node: &'a mut Box<Node>, indices: Vec<i64>) -> &'a mut Box<Node> {
    if let Node::Array { values, shape, .. } = node.deref_mut() {
        return &mut values[get_array_index(indices, shape)];
    };
    runtime_err("Invalid array access".to_string());
}

fn as_int_exprs(indices: &Vec<Box<Node>>) -> Vec<i64> {
    indices
        .iter()
        .map(|index| match index.deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>()
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

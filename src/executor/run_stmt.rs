use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::enums::{Node, Position, VariableType};
use crate::enums::Node::EnumVal;
use crate::executor::run_expr::run_expr;
use crate::executor::run_io::{run_input, run_output};
use crate::executor::{runtime_err, var_type_of};
use crate::executor::variable::{Definition, Executor, Property};

use super::default_var;
use super::run_expr::get_array_index;

pub fn run_stmts(executor: &mut Executor, nodes: &Vec<Box<Node>>) {
    for node in nodes {
        run_stmt(executor, node)
    }
}

pub fn run_stmt(executor: &mut Executor, node: &Box<Node>) {
    match node.deref() {
        Node::Declare { t, children } => run_declare(executor, children, t),
        Node::Const { name, val, .. } => run_const(executor, name, val),
        Node::Enum { name, variants } => run_enum(executor, name, variants),
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
        Node::Record { name, children } => run_record(executor, name, children),
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

fn run_record(executor: &mut Executor, name: &String, children: &Vec<Box<Node>>) {
    let mut props = HashMap::new();
    for node in children.clone() {
        match node.deref() {
            Node::Null => (),
            _ => {
                for (name, prop) in run_composite_prop(executor, &node, false).into_iter() {
                    props.insert(name, prop);
                }
            }
        }
    }
    executor.declare_def(
        name,
        Definition::Record {
            name: name.clone(),
            props,
        },
    )
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
                for (name, prop) in run_composite_prop(executor, &node, false).into_iter() {
                    props.insert(name, prop);
                }
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

fn run_composite_prop(
    executor: &mut Executor,
    prop: &Box<Node>,
    private: bool,
) -> Vec<(String, Property)> {
    match prop.deref() {
        Node::Procedure {
            name,
            params,
            children,
        } => {
            if let Node::String { val, .. } = name.deref() {
                return vec![(
                    val.clone(),
                    Property::Procedure {
                        private,
                        params: params.clone(),
                        children: children.clone(),
                    },
                )];
            }
            unreachable!()
        }
        Node::Declare { children, t } => children.iter().map(|var_name| {
            (
                var_name.clone(),
                Property::Var {
                    private,
                    value: default_var(executor, t),
                    t: t.clone(),
                },
            )
        }).collect(),
        Node::Private(node) => return run_composite_prop(executor, node, true),
        _ => runtime_err("Invalid class declaration".to_string()),
    }
}

fn run_const(executor: &mut Executor, identifier: &String, val: &Box<Node>) {
    executor.declare_var(identifier, val.clone(), &Box::from(var_type_of(val)), false);
}

fn run_enum(executor: &mut Executor, name: &String, variants: &[Box<Node>]) {
    
    executor.declare_def(name, Definition::Enum {name: name.clone()});
    
    for variant in variants {
        if let Node::String { val, .. } = variant.deref() {
            executor.declare_var(val, Box::from(EnumVal { family: name.clone(), val: val.clone() }), &Box::from(VariableType::Custom(name.clone())), false);
        }
    }
}

fn run_declare(executor: &mut Executor, identifiers: &[String], t: &Box<VariableType>) {
    for identifier in identifiers {
        let value = default_var(executor, t);
        executor.declare_var(identifier, value, t, true);
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
            executor.set_var(name, rhs);
        }
        Node::ArrayVar { name, indices, .. } => {
            *run_array_access(executor, name, indices) = rhs;
        }
        Node::RefVar(var) => unsafe { **var = rhs },
        Node::Composite { children } => {
            let mut index = 0;
            let mut base = match children[index].deref() {
                Node::Var { name, .. } => &mut executor.get_var_mut(name).value,
                Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
                Node::RefVar(var) => unsafe { &mut *var.clone() as &mut Box<Node> },
                _ => runtime_err("Invalid assign statement".to_string()),
            };
            if let Node::Object(mut obj_id) = base.deref_mut() {
                for child in children.iter().skip(1) {
                    base = match child.deref() {
                        Node::Var { name, .. } => run_prop_access(executor, obj_id, name),
                        Node::ArrayVar { name, indices, .. } => {
                            run_array_prop_access(executor, obj_id, name, indices)
                        }
                        _ => runtime_err("Invalid property access".to_string()),
                    };
                    if let Node::Object(id) = base.deref_mut() {
                        obj_id = id.clone();
                    }
                }
            }
            *base = rhs;
        }
        _ => runtime_err("Invalid assign statement".to_string()),
    };
}

fn run_array_access<'a>(
    executor: &'a mut Executor,
    name: &String,
    indices: &Vec<Box<Node>>,
) -> &'a mut Box<Node> {
    let indices = indices
        .iter()
        .map(|index| match run_expr(executor, index).deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>();
    let node = &mut executor.get_var_mut(name).value;
    if let Node::Array { values, shape, .. } = node.deref_mut() {
        return &mut values[get_array_index(indices, shape)];
    };
    runtime_err("Invalid array access".to_string());
}

fn run_array_prop_access<'a>(
    executor: &'a mut Executor,
    obj_id: u64,
    name: &String,
    indices: &Vec<Box<Node>>,
) -> &'a mut Box<Node> {
    let indices = indices
        .iter()
        .map(|index| match run_expr(executor, index).deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>();
    let object = executor.heap.get_mut(&obj_id).unwrap();
    if let Some(Property::Var { value, .. }) = object.props.get_mut(name) {
        if let Node::Array { values, shape, .. } = value.deref_mut() {
            return &mut values[get_array_index(indices, shape)];
        };
    }
    runtime_err("Invalid array property access".to_string());
}

fn run_prop_access<'a>(
    executor: &'a mut Executor,
    obj_id: u64,
    name: &String,
) -> &'a mut Box<Node> {
    let object = executor.heap.get_mut(&obj_id).unwrap();
    if let Some(Property::Var { value, .. }) = object.props.get_mut(name) {
        return value;
    }
    runtime_err("Invalid property access".to_string());
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
                true
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

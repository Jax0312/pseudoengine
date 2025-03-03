use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, NodeRef};
use crate::executor::run_expr::{get_array_index, run_expr, run_fn_call_inner};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::{default_var, runtime_err};
use crate::executor::variable::{declare_def, get_def, Definition, Executor, NodeDeref, Property};

pub fn run_class(
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
        return declare_def(
            &mut executor.defs,
            val,
            Definition::Class {
                name: val.clone(),
                props,
            },
        );
    }
    runtime_err("Invalid function declaration".to_string())
}

pub fn run_record(executor: &mut Executor, name: &String, children: &Vec<Box<Node>>) {
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
    declare_def(
        &mut executor.defs,
        name,
        Definition::Record {
            name: name.clone(),
            props,
        },
    )
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
        Node::Declare { children, t } => children
            .iter()
            .map(|var_name| {
                (
                    var_name.clone(),
                    Property::Var {
                        private,
                        value: NodeRef::new_ref(default_var(executor, t)),
                        t: t.clone(),
                    },
                )
            })
            .collect(),
        Node::Private(node) => return run_composite_prop(executor, node, true),
        _ => runtime_err("Invalid class declaration".to_string()),
    }
}

pub fn run_composite_access(executor: &mut Executor, lhs: &Box<Node>) -> NodeRef {
    match lhs.deref() {
        Node::Var { name, .. } => run_var_access(executor, name),
        Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
        Node::Composite { children } => {
            let mut base = match children[0].deref() {
                Node::Var { name, .. } => run_var_access(executor, name),
                Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
                _ => runtime_err("Invalid assign statement".to_string()),
            };
            for child in children.iter().skip(1) {
                base = match child.deref() {
                    Node::Var { name, .. } => run_prop_access(base, name),
                    Node::ArrayVar { name, indices, .. } => {
                        run_array_prop_access(executor, base, name, indices)
                    }
                    _ => runtime_err("Invalid property access".to_string()),
                };
            }
            base
        }
        _ => runtime_err("Invalid assign statement".to_string()),
    }
}

pub fn run_composite(executor: &mut Executor, children: &Vec<Box<Node>>) -> Box<Node> {
    let mut base = match children[0].deref() {
        Node::Var { name, .. } => run_var_access(executor, name),
        Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
        _ => runtime_err("Invalid assign statement".to_string()),
    };
    for child in children.iter().skip(1) {
        base = match child.deref() {
            Node::Var { name, .. } => run_prop_access(base, name),
            Node::ArrayVar { name, indices, .. } => run_array_prop_access(executor, base, name, indices),
            Node::FunctionCall { name, params, .. } => {
                return run_procedure_call(executor, base, name, params);
            },
            _ => runtime_err("Invalid property access".to_string()),
        };
    }
    base.clone_node()
}

fn run_var_access(executor: &mut Executor, name: &String) -> NodeRef {
    let value = &executor.get_var_mut(name).value;
    if let Node::RefVar(reference) = value.borrow().deref().deref() {
        return reference.clone();
    }
    value.clone()
}

fn run_array_access(executor: &mut Executor, name: &String, indices: &Vec<Box<Node>>) -> NodeRef {
    let indices = indices
        .iter()
        .map(|index| match run_expr(executor, index).deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>();
    let node = &executor.get_var_mut(name).value;
    if let Node::Array { values, shape, .. } = node.borrow().deref().deref() {
        return values[get_array_index(indices, shape)].clone();
    };
    runtime_err("Invalid array access".to_string());
}

fn run_array_prop_access(executor: &mut Executor, base: NodeRef, name: &String, indices: &Vec<Box<Node>>) -> NodeRef {
    if let Node::Object { props, .. } = base.borrow().deref().deref() {
        if let Some(Property::Var { value, .. }) = props.get(name) {
            if let Node::Array { values, shape, .. } = value.borrow().deref().deref() {
                let indices = indices
                    .iter()
                    .map(|index| match run_expr(executor, index).deref() {
                        Node::Int { val, .. } => val.clone(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<i64>>();
                return values[get_array_index(indices, shape)].clone();
            };
        }
    }
    runtime_err("Invalid array property access".to_string());
}

fn run_prop_access(base: NodeRef, name: &String) -> NodeRef {
    if let Node::Object { props, .. } = base.borrow().deref().deref() {
        if let Some(Property::Var { value, .. }) = props.get(name) {
            return value.clone();
        }
    }
    runtime_err("Invalid property access".to_string());
}

fn run_procedure_call(
    executor: &mut Executor,
    base: NodeRef,
    name: &String,
    call_params: &Vec<Box<Node>>,
) -> Box<Node> {
    if let Node::Object { props, .. } = base.borrow().deref().deref() {
        executor.enter_scope();
        for (name, prop) in props.iter() {
            if let Property::Var { value, t, .. } = prop {
                let value = Box::new(Node::RefVar(value.clone()));
                executor.declare_var(name, value, &t, true);
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
    }
    runtime_err("Invalid property access".to_string())
}

pub fn run_create_obj(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::FunctionCall { params, name } = node.deref() {
        if let Definition::Class { props, .. } = get_def(&mut executor.defs, name) {
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
                            executor.declare_var(name, value.clone_node(), t, true);
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
                    return Box::new(Node::Object{ name: name.clone(), props });
                }
                runtime_err("Constructor cannot be private".to_string())
            }
            runtime_err("Constructor is not defined".to_string())
        }
        runtime_err(format!("{} is not a class", name))
    }
    unreachable!()
}


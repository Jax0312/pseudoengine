use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, NodeRef, VariableType};
use crate::executor::run_expr::{get_array_index, run_expr, run_fn_call, run_fn_call_inner};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::variable::{declare_def, get_def, Definition, Executor, NodeDeref, Property};
use crate::executor::{def_base_class, default_var, runtime_err};

use super::var_type_of;

pub fn run_class(
    executor: &mut Executor,
    name: &Box<Node>,
    base: &Box<Node>,
    children: &Vec<Box<Node>>,
) {
    let mut class_props = HashMap::new();
    for node in children.clone() {
        match node.deref() {
            Node::Null => (),
            _ => {
                for (name, prop) in run_prop_decl(executor, &node, false).into_iter() {
                    class_props.insert(name, prop);
                }
            }
        }
    }
    if let Node::String { val, .. } = name.deref() {
        let base = match base.deref() {
            Node::String { val, .. } => get_def(&mut executor.defs, val),
            _ => Definition::Null,
        };
        return declare_def(
            &mut executor.defs,
            val,
            Definition::Class {
                name: val.clone(),
                base: Box::new(base),
                props: class_props,
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
                for (name, prop) in run_prop_decl(executor, &node, false).into_iter() {
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

fn run_prop_decl(
    executor: &mut Executor,
    prop: &Box<Node>,
    private: bool,
) -> Vec<(String, Property)> {
    match prop.deref() {
        Node::Procedure {
            name,
            params,
            children,
        } => run_method_decl(name, params, children, private, false),
        Node::Function {
            name,
            params,
            children,
            ..
        } => run_method_decl(name, params, children, private, true),
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
        Node::Private(node) => return run_prop_decl(executor, node, true),
        _ => runtime_err("Invalid class declaration".to_string()),
    }
}

fn run_method_decl(
    name: &Box<Node>,
    params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    private: bool,
    returns: bool,
) -> Vec<(String, Property)> {
    if let Node::String { val, .. } = name.deref() {
        return vec![(
            val.clone(),
            Property::Method {
                private,
                params: params.clone(),
                children: children.clone(),
                returns,
            },
        )];
    }
    unreachable!()
}

pub fn run_access_mut(executor: &mut Executor, node: &Box<Node>) -> NodeRef {
    match node.deref() {
        Node::Var { name, .. } => run_var_access(executor, name),
        Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
        Node::Dereference(value) => run_pointer_access(executor, value),
        Node::Composite { children } => {
            let mut base = match children[0].deref() {
                Node::Var { name, .. } => run_var_access(executor, name),
                Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
                _ => runtime_err("Invalid value access".to_string()),
            };
            for child in children.iter().skip(1) {
                base = match child.deref() {
                    Node::Var { name, .. } => run_prop_access(base, name),
                    Node::ArrayVar { name, indices, .. } => {
                        run_array_prop_access(executor, base, name, indices)
                    }
                    _ => runtime_err("Invalid value access".to_string()),
                };
            }
            base
        }
        _ => runtime_err("Invalid value access".to_string()),
    }
}

pub fn run_access(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    match node.deref() {
        Node::Var { name, .. } => run_var_access(executor, name).clone_node(),
        Node::ArrayVar { name, indices, .. } => {
            run_array_access(executor, name, indices).clone_node()
        }
        Node::Composite { children } => run_composite_access(executor, children),
        _ => runtime_err("Invalid value access".to_string()),
    }
}

pub fn run_composite_access(executor: &mut Executor, children: &Vec<Box<Node>>) -> Box<Node> {
    let mut base = match children[0].deref() {
        Node::Var { name, .. } => run_var_access(executor, name),
        Node::ArrayVar { name, indices, .. } => run_array_access(executor, name, indices),
        Node::FunctionCall { name, params } => {
            NodeRef::new_ref(run_fn_call(executor, name, params))
        }
        _ => runtime_err("Invalid value access".to_string()),
    };
    for child in children.iter().skip(1) {
        base = match child.deref() {
            Node::Var { name, .. } => run_prop_access(base, name),
            Node::ArrayVar { name, indices, .. } => {
                run_array_prop_access(executor, base, name, indices)
            }
            Node::FunctionCall { name, params, .. } => {
                return run_method_call(executor, base, name, params);
            }
            _ => runtime_err("Invalid value access".to_string()),
        };
    }
    base.clone_node()
}

pub fn run_pointer_access(executor: &mut Executor, node: &Box<Node>) -> NodeRef {
    match node.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Dereference(_) => {}
        _ => runtime_err("Cannot dereference value".to_string()),
    };
    let pointer = run_access_mut(executor, node);
    if let Node::Pointer(value) = pointer.borrow().deref().deref() {
        return value.clone();
    }
    runtime_err("Cannot dereference value".to_string());
}

fn run_var_access(executor: &mut Executor, name: &String) -> NodeRef {
    let var = &executor.get_var_mut(name);
    if !var.mutable {
        runtime_err(format!(
            "{} is a constant, it's value cannot be modified",
            name
        ))
    }
    if let Node::RefVar(reference) = var.value.borrow().deref().deref() {
        return reference.clone();
    }
    var.value.clone()
}

fn run_array_access(executor: &mut Executor, name: &String, indices: &Vec<Box<Node>>) -> NodeRef {
    let indices = indices
        .iter()
        .map(|index| match run_expr(executor, index).deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>();
    let var = &executor.get_var_mut(name);
    if !var.mutable {
        runtime_err(format!(
            "{} is a constant, it's value cannot be modified",
            name
        ))
    }
    let value = match var.value.borrow().deref().deref() {
        Node::RefVar(reference) => reference.clone(),
        _ => var.value.clone()
    };
    if let Node::Array { values, shape, .. } = value.borrow().deref().deref() {
        return values[get_array_index(indices, shape)].clone();
    };
    runtime_err("Invalid array access".to_string());
}

fn run_base_prop_access(
    name: &String,
    base: &Box<Node>,
    props: &HashMap<String, Property>,
) -> Option<Property> {
    if let Some(value) = props.get(name) {
        return Some(value.clone());
    } else if let Node::Object { base, props, .. } = base.deref() {
        return run_base_prop_access(name, base, props);
    } else {
        return None;
    }
}

fn run_array_prop_access(
    executor: &mut Executor,
    base: NodeRef,
    name: &String,
    indices: &Vec<Box<Node>>,
) -> NodeRef {
    if let Node::Object { props, base, .. } = base.borrow().deref().deref() {
        let prop = run_base_prop_access(name, base, props);
        if let Some(Property::Var { value, private, .. }) = prop {
            if private {
                runtime_err("Cannot access private property".to_string())
            }
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
    if let Node::Object { props, base, .. } = base.borrow().deref().deref() {
        let prop = run_base_prop_access(name, base, props);
        if let Some(Property::Var { value, private, .. }) = prop {
            if private {
                runtime_err("Cannot access private property".to_string())
            }
            return value;
        }
    }
    runtime_err("Invalid property access".to_string());
}

fn run_method_call(
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
        executor.declare_var(
            &"super".to_string(),
            Box::new(Node::RefVar(base.clone())),
            &Box::new(var_type_of(base.borrow().deref())),
            true,
        );
        if let Some(Property::Method {
            params: fn_params,
            children,
            private,
            returns,
        }) = props.get(name)
        {
            if *private {
                runtime_err("Cannot call private method".to_string())
            }
            let result = run_fn_call_inner(executor, call_params, fn_params, children, *returns);
            executor.exit_scope();
            return result;
        }
    }
    runtime_err("Invalid method access".to_string())
}

pub fn run_create_obj(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::FunctionCall { params, name } = node.deref() {
        if let Definition::Class { props, base, .. } = get_def(&mut executor.defs, name) {
            if let Some(Property::Method {
                private,
                params: fn_params,
                children,
                ..
            }) = props.clone().get("new")
            {
                if *private {
                    runtime_err("Constructor cannot be private".to_string())
                }
                executor.enter_scope();
                let node = def_base_class(props, base, name.clone());
                if let Node::Object { name, base, props } = &node {
                    for (name, prop) in props.iter() {
                        if let Property::Var { value, t, .. } = prop {
                            let value = Box::new(Node::RefVar(value.clone()));
                            executor.declare_var(name, value, t, true);
                        }
                    }
                    executor.declare_var(
                        &"super".to_string(),
                        Box::new(Node::RefVar(NodeRef::new_ref(base.clone()))),
                        &Box::new(var_type_of(&base)),
                        true,
                    );
                    run_fn_call_inner(executor, params, fn_params, children, false);
                    executor.exit_scope();
                    return Box::new(node);
                }
            }
            runtime_err("Constructor is not defined".to_string())
        }
        runtime_err(format!("{} is not a class", name))
    }
    unreachable!()
}

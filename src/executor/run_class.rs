use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, NodeRef, Position, VariableType};
use crate::executor::run_expr::{get_array_index, run_expr, run_fn_call, run_fn_call_inner};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::variable::{Definition, Executor, NodeDeref, Property};
use crate::executor::{def_base_class, default_var};
use crate::utils::err;

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
                for (name, prop) in run_prop_decl(executor, &node).into_iter() {
                    if class_props.contains_key(&name) {
                        if let Property::Var { .. } = prop {
                            err(
                                format!("Property {} already exists", name).as_str(),
                                &node.pos(),
                            );
                        } else {
                            err(
                                format!("Method {} already exists", name).as_str(),
                                &node.pos(),
                            );
                        }
                    };
                    class_props.insert(name, prop);
                }
            }
        }
    }
    if let Node::String { val, pos } = name.deref() {
        let base = match base.deref() {
            Node::String { val, pos } => executor.get_def(val, pos),
            _ => Definition::Null,
        };
        if let Some(Property::Method { .. }) = class_props.get("new") {
            return executor.declare_def(
                val,
                Definition::Class {
                    name: val.clone(),
                    base: Box::new(base),
                    props: class_props,
                },
                pos,
            );
        }
        err("Class must have a constructor", pos);
    }
    unreachable!()
}

pub fn run_record(executor: &mut Executor, name: &Box<Node>, children: &Vec<Box<Node>>) {
    let mut props = HashMap::new();
    for node in children.clone() {
        match node.deref() {
            Node::Null => (),
            _ => {
                for (name, prop) in run_prop_decl(executor, &node).into_iter() {
                    props.insert(name, prop);
                }
            }
        }
    }
    if let Node::String { val, pos } = name.deref() {
        return executor.declare_def(
            val,
            Definition::Record {
                name: val.clone(),
                props,
            },
            pos,
        )
    }
    unreachable!()
}

fn run_prop_decl(executor: &mut Executor, prop: &Box<Node>) -> Vec<(String, Property)> {
    match prop.deref() {
        Node::Procedure {
            name,
            params,
            children,
            private,
            ..
        } => run_method_decl(name, params, children, *private, false),
        Node::Function {
            name,
            params,
            children,
            private,
            ..
        } => run_method_decl(name, params, children, *private, true),
        Node::Declare {
            children,
            t,
            private,
            pos,
            ..
        } => children
            .iter()
            .map(|var_name| {
                (
                    var_name.clone(),
                    Property::Var {
                        private: *private,
                        value: NodeRef::new_ref(default_var(executor, t, pos)),
                        t: t.clone(),
                    },
                )
            })
            .collect(),
        _ => err("Statement not allowed within class", &prop.pos()),
    }
}

fn run_method_decl(
    name: &Box<Node>,
    params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    private: bool,
    returns: bool,
) -> Vec<(String, Property)> {
    let mut names = Vec::new();
    for param in params {
        if let Node::Var { name, .. } = param.deref() {
            if names.contains(name) {
                err(
                    format!("Duplicate parameter {}", name).as_str(),
                    &param.pos(),
                );
            }
            names.push(name.clone());
        }
    }
    if let Node::String { val, .. } = name.deref() {
        if val == "new" && private {
            err("Constructor cannot be private", &name.pos())
        }
        if val == "new" && returns {
            err("Constructor must be a procedure", &name.pos())
        }
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
        Node::Var { name, pos } => run_var_access(executor, name, pos),
        Node::ArrayVar { name, indices, pos } => run_array_access(executor, name, indices, pos),
        Node::Dereference { expr, .. } => run_pointer_access(executor, expr),
        Node::Composite { children, .. } => {
            let mut base = match children[0].deref() {
                Node::Var { name, pos } => run_var_access(executor, name, pos),
                Node::ArrayVar { name, indices, pos } => {
                    run_array_access(executor, name, indices, pos)
                }
                _ => unreachable!(),
            };
            for child in children.iter().skip(1) {
                base = match child.deref() {
                    Node::Var { name, pos } => run_prop_access(base, name, pos),
                    Node::ArrayVar { name, indices, pos } => {
                        run_array_prop_access(executor, base, name, indices, pos)
                    }
                    _ => unreachable!(),
                };
            }
            base
        }
        _ => unreachable!(),
    }
}

pub fn run_access(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    match node.deref() {
        Node::Var { name, pos } => run_var_access(executor, name, pos).clone_node(),
        Node::ArrayVar { name, indices, pos } => {
            run_array_access(executor, name, indices, pos).clone_node()
        }
        Node::Composite { children, .. } => run_composite_access(executor, children),
        _ => unreachable!(),
    }
}

pub fn run_composite_access(executor: &mut Executor, children: &Vec<Box<Node>>) -> Box<Node> {
    let mut base = match children[0].deref() {
        Node::Var { name, pos } => run_var_access(executor, name, pos),
        Node::ArrayVar { name, indices, pos } => run_array_access(executor, name, indices, pos),
        Node::FunctionCall { name, params, pos } => {
            NodeRef::new_ref(run_fn_call(executor, name, params, pos))
        }
        _ => unreachable!(),
    };
    for child in children.iter().skip(1) {
        base = match child.deref() {
            Node::Var { name, pos } => run_prop_access(base, name, pos),
            Node::ArrayVar { name, indices, pos } => {
                run_array_prop_access(executor, base, name, indices, pos)
            }
            Node::FunctionCall { name, params, pos } => {
                return run_method_call(executor, base, name, params, pos);
            }
            _ => unreachable!(),
        };
    }
    base.clone_node()
}

pub fn run_pointer_access(executor: &mut Executor, node: &Box<Node>) -> NodeRef {
    match node.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Dereference { .. } => {}
        _ => err(
            "Value is not a pointer, it cannot be dereferenced",
            &node.pos(),
        ),
    };
    let pointer = run_access_mut(executor, node);
    if let Node::Pointer(value) = pointer.borrow().deref().deref() {
        return value.clone();
    }
    err(
        "Value is not a pointer, it cannot be dereferenced",
        &node.pos(),
    );
}

fn run_var_access(executor: &mut Executor, name: &String, pos: &Position) -> NodeRef {
    let var = &executor.get_var_mut(name, pos);
    if !var.mutable {
        err(
            format!("{} is a constant, it's value cannot be modified", name).as_str(),
            pos,
        )
    }
    if let Node::RefVar(reference) = var.value.borrow().deref().deref() {
        return reference.clone();
    }
    var.value.clone()
}

fn run_array_access(
    executor: &mut Executor,
    name: &String,
    indices: &Vec<Box<Node>>,
    pos: &Position,
) -> NodeRef {
    let nodes = indices.clone();
    let indices = indices
        .iter()
        .map(|index| match run_expr(executor, index).deref() {
            Node::Int { val, .. } => val.clone(),
            _ => unreachable!(),
        })
        .collect::<Vec<i64>>();
    let var = &executor.get_var_mut(name, pos);
    if !var.mutable {
        err(
            format!("{} is a constant, it's value cannot be modified", name).as_str(),
            pos,
        )
    }
    let value = match var.value.borrow().deref().deref() {
        Node::RefVar(reference) => reference.clone(),
        _ => var.value.clone(),
    };
    if let Node::Array { values, shape, .. } = value.borrow().deref().deref() {
        return values[get_array_index(indices, shape, &nodes)].clone();
    };
    err(format!("{} is not an array", name).as_str(), pos)
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
        None
    }
}

fn run_array_prop_access(
    executor: &mut Executor,
    base: NodeRef,
    name: &String,
    indices: &Vec<Box<Node>>,
    pos: &Position,
) -> NodeRef {
    if let Node::Object { props, base, .. } = base.borrow().deref().deref() {
        let prop = run_base_prop_access(name, base, props);
        if let Some(Property::Var { value, private, .. }) = prop {
            if private {
                err("Cannot access private property", pos)
            }
            if let Node::Array { values, shape, .. } = value.borrow().deref().deref() {
                let nodes = indices.clone();
                let indices = indices
                    .iter()
                    .map(|index| match run_expr(executor, index).deref() {
                        Node::Int { val, .. } => val.clone(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<i64>>();
                return values[get_array_index(indices, shape, &nodes)].clone();
            };
            err(format!("{} is not an array", name).as_str(), pos)
        }
        err(format!("Property '{}' not found", name).as_str(), pos)
    }
    err("Value is not an object", pos)
}

fn run_prop_access(base: NodeRef, name: &String, pos: &Position) -> NodeRef {
    if let Node::Object { props, base, .. } = base.borrow().deref().deref() {
        let prop = run_base_prop_access(name, base, props);
        if let Some(Property::Var { value, private, .. }) = prop {
            if private {
                err("Cannot access private property", pos)
            }
            return value;
        }
        err(format!("Property '{}' not found", name).as_str(), pos)
    }
    err("Value is not an object", pos)
}

fn run_method_call(
    executor: &mut Executor,
    base: NodeRef,
    name: &String,
    call_params: &Vec<Box<Node>>,
    pos: &Position,
) -> Box<Node> {
    if let Node::Object { props, base, .. } = base.borrow().deref().deref() {
        let prop = run_base_prop_access(name, base, props);
        if let Some(Property::Method {
            params: fn_params,
            children,
            private,
            returns,
        }) = prop
        {
            executor.enter_scope();
            for (name, prop) in props.iter() {
                if let Property::Var { value, t, .. } = prop {
                    let value = Box::new(Node::RefVar(value.clone()));
                    executor.declare_var(name, value, &t, true, pos);
                } else if let Property::Method {
                    params,
                    children,
                    returns,
                    ..
                } = prop.clone()
                {
                    executor.declare_def(
                        name,
                        Definition::Function {
                            params,
                            children,
                            returns,
                        },
                        pos,
                    );
                }
            }
            if Node::Null != *base.deref() {
                executor.declare_var(
                    &"super".to_string(),
                    Box::new(Node::RefVar(NodeRef::new_ref(base.clone()))),
                    &Box::new(var_type_of(base)),
                    true,
                    pos,
                );
            }
            if private {
                err("Cannot call private method", pos)
            }
            let result =
                run_fn_call_inner(executor, call_params, &fn_params, &children, returns, pos);
            executor.exit_scope();
            return result;
        }
        err(format!("Method '{}' not found", name).as_str(), pos)
    }
    unreachable!()
}

pub fn run_create_obj(executor: &mut Executor, node: &Box<Node>, pos: &Position) -> Box<Node> {
    if let Node::FunctionCall { params, name, .. } = node.deref() {
        if let Definition::Class { props, base, name } = executor.get_def(name, pos) {
            let base = def_base_class(props, base, name.clone());
            let base_ref = NodeRef::new_ref(Box::new(base));
            run_method_call(executor, base_ref.clone(), &"new".to_string(), params, pos);
            return base_ref.clone_node();
        }
        err(format!("{} is not a class", name).as_str(), pos)
    }
    unreachable!()
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::format;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::enums::Node::EnumVal;
use crate::enums::{Node, NodeRef, Position, VariableType};
use crate::executor::run_class::{run_class, run_record};
use crate::executor::run_expr::get_array_index;
use crate::executor::run_expr::{assert_number, run_expr};
use crate::executor::run_file::{run_close_file, run_open_file, run_read_file, run_write_file};
use crate::executor::run_io::{run_input, run_output};
use crate::executor::variable::{declare_def, Definition, Executor, NodeDeref, Property};
use crate::executor::{default_var, runtime_err, var_type_of};

use super::run_class::run_access_mut;

pub fn run_stmts(executor: &mut Executor, nodes: &Vec<Box<Node>>) {
    for node in nodes {
        run_stmt(executor, node)
    }
}

pub fn run_stmt(executor: &mut Executor, node: &Box<Node>) {
    match node.deref() {
        Node::Declare { t, children } => run_declare(executor, children, t),
        Node::Const { name, val, .. } => run_const(executor, name, val),
        Node::RefType { name, ref_to } => run_pointer(executor, name, ref_to),
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
        Node::Switch {
            cmp,
            cases,
            otherwise,
        } => run_switch(executor, cmp, cases, otherwise),
        Node::Output { children } => run_output(executor, children),
        Node::Input { child } => run_input(executor, child),
        Node::Function {
            name,
            params,
            children,
            ..
        } => run_function(executor, name, params, children, true),
        Node::Procedure {
            name,
            params,
            children,
            ..
        } => run_function(executor, name, params, children, false),
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
        Node::OpenFile { filename, mode } => run_open_file(executor, filename, mode),
        Node::ReadFile { filename, var } => run_read_file(executor, filename, var),
        Node::WriteFile { filename, expr } => run_write_file(executor, filename, expr),
        // Node::SeekFile { filename, expr } => run_seek_file(executor, filename, expr),
        Node::CloseFile(filename) => run_close_file(executor, filename),
        _ => unimplemented!(),
    }
}

fn run_function(
    executor: &mut Executor,
    identifier: &Box<Node>,
    params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    returns: bool,
) {
    if let Node::String { val, .. } = identifier.deref() {
        return declare_def(
            &mut executor.defs,
            val,
            Definition::Function {
                params: params.clone(),
                children: children.clone(),
                returns,
            },
        );
    }
    runtime_err("Invalid function declaration".to_string())
}

fn run_pointer(executor: &mut Executor, name: &String, pointer: &Box<VariableType>) {
    declare_def(
        &mut executor.defs,
        name,
        Definition::Pointer {
            name: name.clone(),
            ref_to: pointer.clone(),
        },
    );
}

fn run_const(executor: &mut Executor, identifier: &String, val: &Box<Node>) {
    executor.declare_var(identifier, val.clone(), &Box::from(var_type_of(val)), false);
}

fn run_enum(executor: &mut Executor, name: &String, variants: &[Box<Node>]) {
    declare_def(
        &mut executor.defs,
        name,
        Definition::Enum { name: name.clone() },
    );

    for variant in variants {
        if let Node::String { val, .. } = variant.deref() {
            executor.declare_var(
                val,
                Box::from(EnumVal {
                    family: name.clone(),
                    val: val.clone(),
                }),
                &Box::from(VariableType::Custom(name.clone())),
                false,
            );
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

fn run_switch(
    executor: &mut Executor,
    cmp: &Box<Node>,
    cases: &Vec<Box<Node>>,
    otherwise: &Vec<Box<Node>>,
) {
    let cmp = run_expr(executor, cmp);
    for case in cases {
        if let Node::Case { expr, children } = case.deref() {
            match expr.deref() {
                Node::Range { start, end } => {
                    let (cmp_val, _) = assert_number(&cmp);
                    let (start_val, _) = assert_number(start);
                    let (end_val, _) = assert_number(end);
                    let range = start_val..end_val;
                    if range.contains(&cmp_val) {
                        run_stmts(executor, children);
                        return;
                    }
                }
                _ => {
                    if cmp.deref().val_as_str() == expr.val_as_str() {
                        run_stmts(executor, children);
                        return;
                    }
                }
            }
        } else {
            unreachable!();
        }
    }
    run_stmts(executor, otherwise);
}

pub(crate) fn run_assign(executor: &mut Executor, lhs: &Box<Node>, rhs: &Box<Node>) {
    match lhs.deref() {
        Node::Var { .. } | Node::ArrayVar { .. } | Node::Composite { .. } | Node::Dereference(_) => {}
        _ => runtime_err("Cannot assign to value".to_string()),
    };
    let lhs = run_access_mut(executor, lhs);
    let rhs = run_expr(executor, rhs);
    let lhs_type = var_type_of(lhs.borrow().deref());
    let rhs_type = var_type_of(&rhs);
    if lhs_type != rhs_type {
        runtime_err(format!(
            "Cannot assign type {:?} to type {:?}",
            rhs_type, lhs_type
        ))
    }
    lhs.replace(rhs);
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
                true,
            );
            while start <= end {
                let var = &executor.get_var_mut(name).value;
                var.replace(Box::new(Node::Int {
                    val: start,
                    pos: Position::invalid(),
                }));
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

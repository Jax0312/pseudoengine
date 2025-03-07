use std::ops::Deref;

use crate::enums::Node::EnumVal;
use crate::enums::{Node, Position, VariableType};
use crate::executor::run_class::{run_access_mut, run_class, run_record};
use crate::executor::run_expr::{assert_number, run_expr};
use crate::executor::run_file::{
    run_close_file, run_get_record, run_open_file, run_put_record, run_read_file, run_seek,
    run_write_file,
};
use crate::executor::run_io::{run_input, run_output};
use crate::executor::variable::{Definition, Executor};
use crate::executor::{default_var, var_type_of};
use crate::utils::err;

pub fn run_stmts(executor: &mut Executor, nodes: &Vec<Box<Node>>) -> Option<Box<Node>> {
    for node in nodes {
        if let Some(expr) = run_stmt(executor, node) {
            return Some(expr);
        }
    }
    None
}

pub fn run_stmt(executor: &mut Executor, node: &Box<Node>) -> Option<Box<Node>> {
    match node.deref() {
        Node::Declare {
            t, children, pos, ..
        } => run_declare(executor, children, t, pos),
        Node::Const { name, val, pos } => run_const(executor, name, val, pos),
        Node::PointerDef { name, ref_to, pos } => run_pointer(executor, name, ref_to, pos),
        Node::Enum {
            name,
            variants,
            pos,
        } => run_enum(executor, name, variants, pos),
        Node::Output { children, .. } => run_output(executor, children),
        Node::Input { child, pos } => run_input(executor, child, pos),
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
            ..
        } => run_class(executor, name, base, children),
        Node::Record { name, children, .. } => run_record(executor, name, children),
        Node::Assignment { lhs, rhs, pos } => run_assign(executor, lhs, rhs, pos),
        Node::Null => (),
        Node::OpenFile {
            filename,
            mode,
            pos,
        } => run_open_file(executor, filename, mode, pos),
        Node::ReadFile { filename, var, pos } => run_read_file(executor, filename, var, pos),
        Node::WriteFile {
            filename,
            expr,
            pos,
        } => run_write_file(executor, filename, expr, pos),
        Node::PutRecord { filename, var, pos } => run_put_record(executor, filename, var, pos),
        Node::GetRecord { filename, var, pos } => run_get_record(executor, filename, var, pos),
        Node::SeekFile {
            filename,
            expr,
            pos,
        } => run_seek(executor, filename, expr, pos),
        Node::CloseFile { filename, pos } => run_close_file(executor, filename, pos),
        Node::If { .. }
        | Node::While { .. }
        | Node::For { .. }
        | Node::Switch { .. }
        | Node::Repeat { .. }
        | Node::Return { .. } => {
            if let Some(expr) = run_control_flow(executor, node) {
                return Some(expr);
            }
        }
        _ => {
            run_expr(executor, node);
        }
    }
    None
}

pub fn run_control_flow(executor: &mut Executor, node: &Box<Node>) -> Option<Box<Node>> {
    match node.deref() {
        Node::If {
            cond,
            true_body,
            false_body,
            ..
        } => run_if(executor, cond, true_body, false_body),
        Node::While { cond, body, .. } => run_while(executor, cond, body),
        Node::Repeat { cond, body, .. } => run_repeat(executor, cond, body),
        Node::For {
            iter,
            range,
            step,
            body,
            ..
        } => run_for(executor, iter, range, step, body),
        Node::Switch {
            cmp,
            cases,
            otherwise,
            ..
        } => run_switch(executor, cmp, cases, otherwise),
        Node::Return { expr, .. } => Some(run_expr(executor, expr)),
        _ => unreachable!(),
    }
}

fn run_function(
    executor: &mut Executor,
    identifier: &Box<Node>,
    params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    returns: bool,
) {
    let mut names = Vec::new();
    for param in params {
        if let Node::Var { name, .. } = param.deref() {
            if names.contains(name) {
                err(format!("Duplicate parameter {}", name).as_str(), &param.pos());
            }
            names.push(name.clone());
        }
    }
    if let Node::String { val, .. } = identifier.deref() {
        return executor.declare_def(
            val,
            Definition::Function {
                params: params.clone(),
                children: children.clone(),
                returns,
            },
            &identifier.pos(),
        );
    }
    unreachable!()
}

fn run_pointer(
    executor: &mut Executor,
    name: &String,
    pointer: &Box<VariableType>,
    pos: &Position,
) {
    executor.declare_def(
        name,
        Definition::Pointer {
            name: name.clone(),
            ref_to: pointer.clone(),
        },
        pos,
    );
}

fn run_const(executor: &mut Executor, identifier: &String, val: &Box<Node>, pos: &Position) {
    executor.declare_var(
        identifier,
        val.clone(),
        &Box::from(var_type_of(val)),
        false,
        pos,
    );
}

fn run_enum(executor: &mut Executor, name: &String, variants: &[Box<Node>], pos: &Position) {
    executor.declare_def(name, Definition::Enum { name: name.clone() }, pos);

    for variant in variants {
        if let Node::String { val, pos } = variant.deref() {
            executor.declare_var(
                val,
                Box::from(EnumVal {
                    family: name.clone(),
                    val: val.clone(),
                }),
                &Box::from(VariableType::Custom(name.clone())),
                false,
                pos,
            );
        }
    }
}

fn run_declare(
    executor: &mut Executor,
    identifiers: &[String],
    t: &Box<VariableType>,
    pos: &Position,
) {
    for identifier in identifiers {
        let value = default_var(executor, t, pos);
        executor.declare_var(identifier, value, t, true, pos);
    }
}

fn run_if(
    executor: &mut Executor,
    cond: &Box<Node>,
    true_body: &Vec<Box<Node>>,
    false_body: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    if as_boolean_expr(executor, cond) {
        run_stmts(executor, true_body)
    } else {
        run_stmts(executor, false_body)
    }
}

fn run_while(
    executor: &mut Executor,
    cond: &Box<Node>,
    body: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    loop {
        if as_boolean_expr(executor, cond) {
            if let Some(expr) = run_stmts(executor, body) {
                return Some(expr);
            }
        } else {
            break;
        }
    }
    None
}

fn run_repeat(
    executor: &mut Executor,
    cond: &Box<Node>,
    body: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    loop {
        if let Some(expr) = run_stmts(executor, body) {
            return Some(expr);
        }
        if as_boolean_expr(executor, cond) {
            break;
        }
    }
    None
}

fn run_switch(
    executor: &mut Executor,
    cmp: &Box<Node>,
    cases: &Vec<Box<Node>>,
    otherwise: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    let cmp = run_expr(executor, cmp);
    for case in cases {
        if let Node::Case { expr, children, .. } = case.deref() {
            match expr.deref() {
                Node::Range { start, end, .. } => {
                    let (cmp_val, _) = assert_number(&cmp);
                    let (start_val, _) = assert_number(start);
                    let (end_val, _) = assert_number(end);
                    let range = start_val..end_val;
                    if range.contains(&cmp_val) {
                        return run_stmts(executor, children);
                    }
                }
                _ => {
                    if cmp.deref().val_as_str() == expr.val_as_str() {
                        return run_stmts(executor, children);
                    }
                }
            }
        } else {
            unreachable!();
        }
    }
    run_stmts(executor, otherwise)
}

pub(crate) fn run_assign(
    executor: &mut Executor,
    lhs: &Box<Node>,
    rhs: &Box<Node>,
    pos: &Position,
) {
    match lhs.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Dereference { .. } => {}
        _ => err("Cannot assign to this value", &lhs.pos()),
    };
    let lhs = run_access_mut(executor, lhs);
    let rhs = run_expr(executor, rhs);
    let lhs_type = var_type_of(lhs.borrow().deref());
    let rhs_type = var_type_of(&rhs);
    if lhs_type != rhs_type {
        err(
            format!(
                "Cannot assign type {:?} to type {:?}",
                rhs_type.str(),
                lhs_type.str()
            )
            .as_str(),
            pos,
        )
    }
    lhs.replace(rhs);
}

fn run_for(
    executor: &mut Executor,
    iter: &Box<Node>,
    range: &Box<Node>,
    step: &Box<Node>,
    body: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    if let Node::Var { name, pos } = iter.deref() {
        if let Node::Range { start, end, .. } = range.deref() {
            let mut start = as_number_expr(executor, start);
            let end = as_number_expr(executor, end);
            let step = if let Node::Null = *step.deref() {
                1
            } else {
                as_number_expr(executor, step)
            };
            if !executor.var_exist(name) {
                executor.declare_var(
                    name,
                    Box::new(Node::Int {
                        val: start,
                        pos: Position::invalid(),
                    }),
                    &Box::new(VariableType::Integer),
                    true,
                    pos
                );
            }
            while start <= end {
                let var = &executor.get_var_mut(name, pos).value;
                var.replace(Box::new(Node::Int {
                    val: start,
                    pos: Position::invalid(),
                }));
                start += step;
                if let Some(expr) = run_stmts(executor, body) {
                    return Some(expr);
                }
            }
            return None;
        };
    }
    unreachable!()
}

pub fn as_number_expr(executor: &mut Executor, node: &Box<Node>) -> i64 {
    let expr = run_expr(executor, node);
    return match *expr.deref() {
        Node::Int { val, .. } => val,
        _ => err("Expected INTEGER", &expr.pos()),
    };
}

pub fn as_boolean_expr(executor: &mut Executor, node: &Box<Node>) -> bool {
    let expr = run_expr(executor, node);
    if let Node::Boolean { val, .. } = expr.deref() {
        return val.clone();
    };
    err("Expected BOOLEAN", &expr.pos())
}

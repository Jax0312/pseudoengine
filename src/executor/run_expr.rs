use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::enums::{Index, Node, Position, VariableType};
use crate::executor::run_builtins::match_builtin;
use crate::executor::run_class::{run_access_mut, run_composite_access, run_create_obj};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::var_type_of;
use crate::executor::variable::{Definition, Executor, NodeDeref, Property};
use crate::utils::err;

use super::run_class::run_access;
use super::run_stmt::run_stmts;

pub fn run_expr(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    match node.deref() {
        Node::Binary { op, rhs, lhs, pos } => run_binary(executor, op, rhs, lhs, pos),
        Node::Unary { op, expr, pos } => run_unary(executor, op, expr, pos),
        Node::Var { name, pos } => run_var(executor, name, pos),
        Node::FunctionCall { name, params, pos } => run_fn_call(executor, name, params, pos),
        Node::ArrayVar { name, indices, pos } => run_array_var(executor, name, indices, pos),
        Node::CreateObject { call, pos } => run_create_obj(executor, call, pos),
        Node::Composite { children, .. } => run_composite_access(executor, children),
        Node::Reference { expr, .. } => run_reference(executor, expr),
        Node::Dereference { expr, .. } => run_dereference(executor, expr),
        _ => node.clone(),
    }
}

fn run_array_var(
    executor: &mut Executor,
    name: &String,
    indices: &Vec<Box<Node>>,
    pos: &Position,
) -> Box<Node> {
    let nodes = indices.clone();
    let indices = indices
        .iter()
        .map(|index| as_number_expr(executor, index))
        .collect::<Vec<i64>>();
    let node = executor.get_var(name, pos).value.borrow();
    if let Node::Array { values, shape, .. } = node.deref().deref() {
        return values[get_array_index(indices, shape, &nodes)]
            .borrow()
            .clone();
    };
    err(format!("{} is not an array", name).as_str(), pos)
}

pub fn get_array_index(indices: Vec<i64>, shape: &Vec<Index>, nodes: &Vec<Box<Node>>) -> usize {
    let mut size = 1;
    let mut total_index = 0;
    let pos = Position::range(nodes[0].pos(), nodes[indices.len() - 1].pos());
    if indices.len() != shape.len() {
        err("Number of indices doesnt match array shape", &pos)
    }
    for (i, (shape, index)) in shape.iter().zip(indices).rev().enumerate() {
        // bound check
        if index < shape.lower || index > shape.upper {
            err(
                format!(
                    "Index out of bounds: {} is not in range of {}..{}",
                    index, shape.lower, shape.upper
                )
                .as_str(),
                &nodes[i].pos(),
            );
        }
        // 1D index calculation, bounds are inclusive hence +1
        total_index += (index - shape.lower) * size;
        size = size * (shape.upper - shape.lower + 1);
    }
    total_index as usize
}

fn run_var(executor: &mut Executor, name: &String, pos: &Position) -> Box<Node> {
    let value = &executor.get_var(name, pos).value;
    if let Node::RefVar(reference) = value.borrow().deref().deref() {
        return (*reference).clone_node();
    }
    value.clone_node()
}

fn run_binary(
    executor: &mut Executor,
    op: &String,
    rhs: &Box<Node>,
    lhs: &Box<Node>,
    pos: &Position,
) -> Box<Node> {
    let op = op.as_str();
    match op {
        "+" | "-" | "*" | "/" | "%" | "//" => run_arithmetic_op(executor, op, rhs, lhs),
        "<" | ">" | "<=" | ">=" => run_comparison_op(executor, op, rhs, lhs),
        "=" | "!=" => run_eq_op(executor, op, rhs, lhs, pos),
        "&&" | "||" => run_logical_op(executor, op, rhs, lhs),
        "&" => run_concat_op(executor, rhs, lhs),
        _ => unimplemented!(),
    }
}

fn run_concat_op(executor: &mut Executor, rhs: &Box<Node>, lhs: &Box<Node>) -> Box<Node> {
    let rhs_pos = rhs.pos();
    let lhs_pos = lhs.pos();
    let rhs = run_expr(executor, rhs);
    let lhs = run_expr(executor, lhs);
    if var_type_of(&rhs) != VariableType::String {
        err("'&' can only be performed on STRING", &rhs_pos)
    }
    if var_type_of(&lhs) != VariableType::String {
        err("'&' can only be performed on STRING", &lhs_pos)
    }
    Box::from(Node::String {
        val: format!("{}{}", lhs.val_as_str(), rhs.val_as_str()),
        pos: Position::invalid(),
    })
}

// function for equality op
fn run_eq_op(
    executor: &mut Executor,
    op: &str,
    rhs: &Box<Node>,
    lhs: &Box<Node>,
    pos: &Position,
) -> Box<Node> {
    let rhs = run_expr(executor, rhs);
    let lhs = run_expr(executor, lhs);

    if var_type_of(&rhs) != var_type_of(&lhs) {
        err(
            format!(
                "Cannot compare types {} AND {}",
                var_type_of(&rhs).str(),
                var_type_of(&rhs).str()
            )
            .as_str(),
            pos,
        )
    }

    match lhs.deref() {
        Node::Int { .. }
        | Node::Real { .. }
        | Node::Boolean { .. }
        | Node::String { .. }
        | Node::Date { .. }
        | Node::EnumVal { .. } => {}
        Node::Array { .. } | Node::Object { .. } | Node::Pointer { .. } => err(
            format!(
                "Cannot compare type {}. {}",
                var_type_of(&lhs).str(),
                crate::utils::SUPPORT_MESSAGE
            )
            .as_str(),
            pos,
        ),
        _ => unimplemented!("{:?}", lhs),
    }

    Box::from(Node::Boolean {
        val: match op {
            "=" => rhs.val_as_str() == lhs.val_as_str(),
            "!=" => rhs.val_as_str() != lhs.val_as_str(),
            _ => unreachable!(),
        },
        pos: Position::invalid(),
    })
}

fn run_logical_op(
    executor: &mut Executor,
    op: &str,
    rhs: &Box<Node>,
    lhs: &Box<Node>,
) -> Box<Node> {
    let rhs_pos = rhs.pos();
    let lhs_pos = lhs.pos();
    let rhs = run_expr(executor, rhs);
    let lhs = run_expr(executor, lhs);
    let (rhs_val, is_bool) = assert_boolean(&rhs);
    if !is_bool {
        err(
            format!("Logical operation {} can only be performed on BOOLEAN", op).as_str(),
            &rhs_pos,
        )
    }

    let (lhs_val, is_bool) = assert_boolean(&lhs);
    if !is_bool {
        err(
            format!("Logical operation {} can only be performed on BOOLEAN", op).as_str(),
            &lhs_pos,
        )
    }
    Box::from(match op {
        "&&" => Node::Boolean {
            val: lhs_val && rhs_val,
            pos: Position::invalid(),
        },
        "||" => Node::Boolean {
            val: lhs_val || rhs_val,
            pos: Position::invalid(),
        },
        _ => unreachable!(),
    })
}

fn run_unary(executor: &mut Executor, op: &str, expr: &Box<Node>, pos: &Position) -> Box<Node> {
    let expr_pos = expr.pos();
    let expr = run_expr(executor, expr);
    if op == "!" {
        let (rhs_val, is_bool) = assert_boolean(&expr);
        if !is_bool {
            err(
                format!("Logical operation {} can only be performed on BOOLEAN", op).as_str(),
                &expr_pos,
            )
        }

        Box::from(Node::Boolean {
            val: !rhs_val,
            pos: Position::invalid(),
        })
    } else {
        let (val, is_real) = assert_number(&expr);

        let res = match op {
            "_-" => -val,
            "_+" => val,
            _ => unreachable!(),
        };

        if is_real {
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
}

fn run_arithmetic_op(
    executor: &mut Executor,
    op: &str,
    rhs: &Box<Node>,
    lhs: &Box<Node>,
) -> Box<Node> {
    let rhs = run_expr(executor, rhs);
    let lhs = run_expr(executor, lhs);
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

fn run_comparison_op(
    executor: &mut Executor,
    op: &str,
    rhs: &Box<Node>,
    lhs: &Box<Node>,
) -> Box<Node> {
    let rhs = run_expr(executor, rhs);
    let lhs = run_expr(executor, lhs);
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

pub fn assert_number(node: &Box<Node>) -> (f64, bool) {
    match *node.deref() {
        Node::Int { val, .. } => (val as f64, false),
        Node::Real { val, .. } => (val, true),
        _ => err(
            "Arithmetic operation can only be performed on INTEGER or REAL",
            &node.pos(),
        ),
    }
}

fn assert_boolean(node: &Box<Node>) -> (bool, bool) {
    match *node.deref() {
        Node::Boolean { val, .. } => (val, true),
        _ => (false, false),
    }
}

pub fn run_fn_call(
    executor: &mut Executor,
    name: &String,
    call_params: &Vec<Box<Node>>,
    pos: &Position,
) -> Box<Node> {
    match match_builtin(executor, name, call_params, pos) {
        Some(result) => return result,
        None => {}
    };

    if let Definition::Function {
        params,
        mut children,
        returns,
    } = executor.get_def(name, pos)
    {
        return run_fn_call_inner(executor, call_params, &params, &mut children, returns, pos);
    }
    err("Value is not a function, it cannot be called", pos)
}

pub fn run_fn_call_inner(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    returns: bool,
    pos: &Position,
) -> Box<Node> {
    executor.enter_scope();
    if fn_params.len() != call_params.len() {
        err(
            "Number of parameters doesnt match function definition",
            &pos,
        )
    }
    for (call_param, fn_param) in call_params.iter().zip(fn_params) {
        if let Node::Declare {
            t,
            children,
            byref,
            pos,
            ..
        } = fn_param.deref()
        {
            let param_name = &children[0];
            let value = if *byref {
                match call_param.deref() {
                    Node::Var { .. }
                    | Node::ArrayVar { .. }
                    | Node::Composite { .. }
                    | Node::Dereference { .. } => {}
                    _ => err("Cannot pass this value byref", &call_param.pos()),
                };
                let node = run_access_mut(executor, &call_param);
                Box::new(Node::RefVar(node.clone()))
            } else {
                run_expr(executor, call_param)
            };
            if var_type_of(&value) == *t.deref() {
                executor.declare_var(param_name, value, t, true, pos);
            } else {
                let msg = format!(
                    "Cannot assign type {:?} to parameter of type {:?}",
                    var_type_of(&value).str(),
                    t.str()
                );
                err(msg.as_str(), &call_param.pos())
            }
        }
    }
    if let Some(expr) = run_stmts(executor, children) {
        if !returns {
            err("Cannot return within procedure", pos)
        }
        executor.exit_scope();
        return expr;
    }
    if returns {
        err("Missing return statement", pos)
    } else {
        executor.exit_scope();
        Box::new(Node::Null)
    }
}

pub fn run_reference(executor: &mut Executor, value: &Box<Node>) -> Box<Node> {
    match value.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Dereference { .. } => {}
        _ => err(
            "Value is not a pointer, it cannot be referenced",
            &value.pos(),
        ),
    };
    let pointer = run_access_mut(executor, value);
    return Box::new(Node::Pointer(pointer));
}

pub fn run_dereference(executor: &mut Executor, value: &Box<Node>) -> Box<Node> {
    match value.deref() {
        Node::Var { .. }
        | Node::ArrayVar { .. }
        | Node::Composite { .. }
        | Node::Dereference { .. } => {}
        _ => err(
            "Value is not a pointer, it cannot be dereferenced",
            &value.pos(),
        ),
    };
    let pointer = run_access(executor, value);
    if let Node::Pointer(pointer) = pointer.deref() {
        return pointer.clone_node();
    }
    err(
        "Value is not a pointer, it cannot be dereferenced",
        &value.pos(),
    );
}

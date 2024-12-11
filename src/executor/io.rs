use std::ops::Deref;
use crate::enums::{Node, Position, VariableType};
use crate::executor::{get_var, runtime_err, Scope};

pub fn run_output(scopes: &mut Vec<Scope>, expressions: &Vec<Box<Node>>) {
    evaluate_expr(scopes, expressions.first().unwrap());
}

fn evaluate_expr(scopes: &mut Vec<Scope>, expr: &Box<Node>) {
    
    let mut stack = Vec::<Box<Node>>::new();
    
    match expr.deref() { 
        Node::Expression(children) => {
            for child in children {
                match child.deref() {
                    Node::Op {op, pos} => {
                        let op = op.as_str();
                        if op == "+" || op == "-" || op == "*" || op == "/" || op == "%" || op == "//" || op == "_+" || op == "_-" {
                            arithmetic_op(scopes, &mut stack, op);
                        }
                    },
                    _ => stack.push(child.clone()),
                }
            }
        }
        _ => unreachable!(),
    }

    println!("{:?}", stack);
}

fn arithmetic_op(scopes: &mut Vec<Scope>, stack: &mut Vec<Box<Node>>, op: &str) {
    let rhs = if let Some(value) = stack.pop() { value } else { runtime_err("Invalid operation".to_string())};
    let lhs = if let Some(value) = stack.pop() { value } else { runtime_err("Invalid operation".to_string())};

    let mut cast_to_real = false;
    
    let lhs_val = match *lhs {
        Node::Int {val, pos} => val as f64,
        Node::Real {val, pos} => { cast_to_real = true; val },
        Node::Var {name, pos} => {
            let var = get_var(scopes, &name);
            match *var.value {
                Node::Int {val, pos} => val as f64,
                Node::Real {val, pos} => { cast_to_real = true; val },
                _ => runtime_err("Invalid operation".to_string()),
            }
        },
        Node::ArrayVar {name, indices, pos} => {unimplemented!()},
        _ => runtime_err("Invalid operation".to_string()),
    };

    let rhs_val = match *rhs {
        Node::Int {val, pos} => val as f64,
        Node::Real {val, pos} => { cast_to_real = true; val },
        Node::Var {name, pos} => {
            let var = get_var(scopes, &name);
            match *var.value {
                Node::Int {val, pos} => val as f64,
                Node::Real {val, pos} => { cast_to_real = true; val },
                _ => runtime_err("Invalid operation".to_string()),
            }
        },
        Node::ArrayVar {name, indices, pos} => {unimplemented!()},
        _ => runtime_err("Invalid operation".to_string()),
    };

    let res = match op {
       "+" => lhs_val + rhs_val,
        "-" => lhs_val - rhs_val,
        "*" => lhs_val * rhs_val,
        "/" => lhs_val / rhs_val,
        "%" => lhs_val % rhs_val,
        "//" => (lhs_val / rhs_val).floor(),
        _ => unreachable!()
    };

    stack.push(if cast_to_real {
        Box::from(Node::Real{val: res, pos: Position::invalid()})
    } else {
        Box::from(Node::Int{val: res.trunc() as i64, pos: Position::invalid()})
    })

}
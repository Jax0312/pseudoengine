use std::any::Any;
use crate::enums::{Node, VariableType};
use crate::executor::{get_var, runtime_err, Scope};

pub fn run_output(scopes: &mut Vec<Scope>, expressions: &Vec<Box<Node>>) {
        
}

fn evaluate_expr(scopes: &mut Vec<Scope>, expr: Box<Node>) {
    
    let mut stack = Vec::<Box<Node>>::new();
    
    match *expr {
        Node::Expression(children) => {
            for child in children {
                match *child {
                    Node::Op {op, pos} => {
                        let op = op.as_str();
                        if op == "+" || op == "-" || op == "*" || op == "/" || op == "%" || op == "//" || op == "_+" || op == "_-" {
                            arithmetic_op(scopes, &mut stack, op);                            
                        }
                    },
                    _ => stack.push(child),
                }
            }
        }
        _ => unreachable!(),
    }
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
            if var.t != VariableType::Integer && var.t != VariableType::Real {
                runtime_err("Invalid operation".to_string())
            }
            *var.value.clone().downcast::<f64>().unwrap().clone()
        },
        Node::ArrayVar {name, indices, pos} => {unimplemented!()},
        _ => runtime_err("Invalid operation".to_string()),
    };

    let rhs_val = match *rhs {
        Node::Int {val, pos} => val as f64,
        Node::Real {val, pos} => { cast_to_real = true; val },
        Node::Var {name, pos} => {
            let var = get_var(scopes, &name);
            if var.t != VariableType::Integer && var.t != VariableType::Real {
                runtime_err("Invalid operation".to_string())
            }
            *var.value.downcast::<f64>().unwrap().clone()
        },
        Node::ArrayVar {name, indices, pos} => {unimplemented!()},
        _ => runtime_err("Invalid operation".to_string()),
    };
    
    
}

// "+" => Operator {
// precedence: 3,
// associativity: Associativity::Left,
// },
// "-" => Operator {
// precedence: 3,
// associativity: Associativity::Left,
// },
// "_+" => Operator {
// precedence: 3,
// associativity: Associativity::Right,
// },
// "_-" => Operator {
// precedence: 3,
// associativity: Associativity::Right,
// },
// "*" => Operator {
// precedence: 4,
// associativity: Associativity::Left,
// },
// "/" => Operator {
// precedence: 4,
// associativity: Associativity::Left,
// },
// "//" => Operator {
// precedence: 4,
// associativity: Associativity::Left,
// },
// "%" => Operator {
// precedence: 4,
// associativity: Associativity::Left,
// },
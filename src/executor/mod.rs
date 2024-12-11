// use crate::parser::{Node, VariableType, Operator};
// use std::any::Any;

mod variable;
mod io;

use std::any::Any;
// use std::io;
use std::collections::HashMap;
use crate::enums::{Node, VariableType};
use crate::executor::io::run_output;
use crate::executor::variable::run_declare;

pub fn start(mut nodes: Vec<Box<Node>>) {
    let mut scopes = vec![Scope::Global(State::new())];
    
    for node in nodes {
        match *node {
            Node::Main {children} => {
                for child in children {
                    run(&mut scopes, &child)       
                }
            },
            _ => unimplemented!(),
        }
    }
}

fn run(scopes: &mut Vec<Scope>, node: &Node) {
    match node {
        Node::Declare { t, children } => run_declare(scopes, children, t),
        Node::Output {children} => run_output(scopes, children),
        Node::Null => (),
        _ => unimplemented!(),
    }
}

// pub fn get_var<>(states: &mut Vec<Scope>, identifier: &String) -> &mut Variable {
// 
//     for scope in states.iter_mut().rev() {
//         match scope {
//             &mut Scope::Global(mut state) => {
//                 if let Some(var) = state.variables.get_mut(identifier) {
//                     return var;
//                 } else {
//                     break;
//                 }
//             }
//             &mut Scope::Local(mut state) => {
//                 if let Some(var) = state.variables.get_mut(identifier) {
//                     return var;
//                 }
//             },
//         }
//     }
//     
//     // Search for value in top global
//     let Scope::Global(mut state) = states.first().unwrap() else { unreachable!() };
//     if let Some(var) = state.variables.get_mut(identifier) {
//         return var;
//     }
//     
//     runtime_err(format!("{} is not declared", identifier))
//     
// }

pub fn set_var(states: &mut [Scope], identifier: &String, value: Box<Node>) {
    for scope in states.iter_mut().rev() {
        match scope {
            Scope::Global(ref mut state) => {
                if let Some(var) = state.variables.get_mut(identifier) {
                    return var.value = value;
                } else {
                    break;
                }
            }
            Scope::Local(ref mut state) => {
                if let Some(var) = state.variables.get_mut(identifier) {
                    return var.value = value;
                }
            }
        }
    }

    if let Some(Scope::Global(state)) = states.first_mut() {
        if let Some(var) = state.variables.get_mut(identifier) {
            return var.value = value;
        }
    }

    runtime_err(format!("{} is not declared", identifier))    
}

pub fn get_var<'a>(states: &'a[Scope], identifier: &String) -> &'a Variable {
    for scope in states.iter().rev() {
        match scope {
            Scope::Global(state) => {
                if let Some(var) = state.variables.get(identifier) {
                    return var;
                } else {
                    break;
                }
            }
            Scope::Local(state) => {
                if let Some(var) = state.variables.get(identifier) {
                    return var;
                }
            }
        }
    }

    if let Some(Scope::Global(state)) = states.first() {
        if let Some(var) = state.variables.get(identifier) {
            return var;
        }
    }

    runtime_err(format!("{} is not declared", identifier))
}

fn runtime_err(message: String) -> ! {
    println!("Runtime error: {}", message);
    panic!()
}

pub struct State {
    pub variables: HashMap<String, Variable>,
}

impl State {
    fn new() -> Self {
        Self { variables: HashMap::new() }
    }
}

pub enum Scope {
    Global(State),
    Local(State)
}

#[derive(Debug)]
pub struct Variable {
    pub value: Box<Node>,
    pub t: VariableType,
}


// pub fn run(mut nodes: Vec<Box<Node>>) {
//     let mut g_state = GlobalState {
//         variables: HashMap::new(),
//     };
//     nodes.reverse();
//
//     loop {
//         let node = nodes.pop();
//
//         if node.is_none() {
//             break;
//         }
//
//         let node = node.unwrap();
//
//         match *node {
//             Node::Declare { t, children } => declare(&mut g_state, t, &children),
//             Node::Output { children } => output(&g_state, children),
//             Node::Input { child } => input(&mut g_state, child),
//             Node::Assignment { lhs, rhs } => assign(&mut g_state, *lhs, *rhs),
//             _ => (),
//         }
//     }
// }
//
// fn declare(g_state: &mut GlobalState, t: VariableType, identifiers: &Vec<String>) {
//     for name in identifiers.iter() {
//         if g_state.variables.get(name).is_some() {
//             eprintln!("Variable {} has already been declared before", name);
//             panic!();
//         }
//         g_state.variables.insert(
//             name.clone(),
//             Variable {
//                 value: Box::new(()),
//                 t: t.clone(),
//             },
//         );
//     }
// }
//
// fn assign(g_state: &mut GlobalState, lhs: Node, rhs: Node) {
//     match lhs {
//         Node::Var(identifier) => {
//             match g_state.variables.get(&identifier) {
//                 None => {
//                     eprintln!("Variable with name {} has not been declared", identifier);
//                     panic!();
//                 }
//                 Some(v) => match (&v.t, expr_val(g_state, rhs)) {
//                     (VariableType::Integer, Node::Int(expr)) => {
//                         g_state.variables.insert(
//                             identifier,
//                             Variable {
//                                 t: v.t.clone(),
//                                 value: Box::from(expr),
//                             },
//                         );
//                     }
//                     (VariableType::String, Node::String(expr)) => {
//                         g_state.variables.insert(
//                             identifier,
//                             Variable {
//                                 t: v.t.clone(),
//                                 value: Box::new(expr),
//                             },
//                         );
//                     },
//                     _ => {},
//                 },
//             }
//         }
//         _ => {
//             eprintln!("Expected an identifier");
//             panic!();
//         }
//     }
// }
//
// fn input(g_state: &mut GlobalState, child: Box<Node>) {
//     match *child {
//         Node::Var(identifier) => {
//             let mut temp = String::new();
//             io::stdin()
//                 .read_line(&mut temp)
//                 .expect("Failed to read input");
//             temp = temp.strip_suffix("\r\n").or(temp.strip_suffix("\n")).unwrap().parse().unwrap();
//             match g_state.variables.get(&identifier) {
//                 None => {
//                     eprintln!("Variable with name {} has not been declared", identifier);
//                     panic!();
//                 }
//                 Some(v) => match v.t {
//                     VariableType::Integer => {
//                         g_state.variables.insert(
//                             identifier,
//                             Variable {
//                                 t: v.t.clone(),
//                                 value: Box::from(temp.parse::<i64>().unwrap_or(0)),
//                             },
//                         );
//                     }
//                     VariableType::String => {
//                         g_state.variables.insert(
//                             identifier,
//                             Variable {
//                                 t: v.t.clone(),
//                                 value: Box::new(temp),
//                             },
//                         );
//                     },
//                     _ => {},
//                 },
//             }
//         }
//         _ => {
//             eprintln!("Expected an identifier");
//             panic!();
//         }
//     }
// }
//
// fn output(g_state: &GlobalState, children: Vec<Box<Node>>) {
//     for e in children.into_iter() {
//         output_expr(g_state, *e);
//     }
//     println!();
// }
//
// fn output_expr(g_state: &GlobalState, expr: Node) {
//     match expr_val(g_state, expr) {
//         Node::Int(value) => print!("{}", value.to_string()),
//         Node::String(value) => print!("{}", value),
//         _ => (),
//     }
// }
//
// fn var(g_state: &GlobalState, identifier: String) -> Node {
//     match g_state.variables.get(&identifier) {
//         Some(v) => match v.t {
//             VariableType::Integer => {
//                 Node::Int(v.value.downcast_ref::<i64>().unwrap().clone())
//             }
//             VariableType::String => {
//                 Node::String(v.value.downcast_ref::<String>().unwrap().clone())
//             }
//             _ => unimplemented!()
//         },
//         None => {
//             eprintln!("Variable {} has not been declared", identifier);
//             panic!();
//         }
//     }
// }
//
// fn expr_val(g_state: &GlobalState, expr: Node) -> Node {
//     match expr {
//         Node::Var(identifier) => var(g_state, identifier),
//         Node::BinaryExpr{ op, rhs, lhs } => bin_expr(g_state, op, *rhs, *lhs),
//         _ => expr,
//     }
// }
//
// fn bin_expr(g_state: &GlobalState, op: Operator, rhs: Node, lhs: Node) -> Node {
//     match (op, expr_val(g_state, rhs), expr_val(g_state, lhs)) {
//         (Operator::Plus, Node::Int(rhs), Node::Int(lhs)) => Node::Int(rhs + lhs),
//         (Operator::Minus, Node::Int(rhs), Node::Int(lhs)) => Node::Int(rhs - lhs),
//         (Operator::Star, Node::Int(rhs), Node::Int(lhs)) => Node::Int(rhs * lhs),
//         (Operator::Slash, Node::Int(rhs), Node::Int(lhs)) => Node::Int(rhs / lhs),
//         _ => unimplemented!(),
//     }
// }
//

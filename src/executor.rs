// use crate::parser::{Node, VariableType, Operator};
use std::any::Any;
use std::collections::HashMap;
use std::io;

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
// struct GlobalState {
//     variables: HashMap<String, Variable>,
// }
// #[derive(Debug)]
// struct Variable {
//     value: Box<dyn Any>,
//     t: VariableType,
// }

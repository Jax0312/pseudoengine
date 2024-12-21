mod run_expr;
mod run_io;
mod run_stmt;
mod variable;

use crate::executor::run_stmt::run_stmts;
use crate::executor::variable::Executor;
use crate::enums::Node;

pub fn run(nodes: Vec<Box<Node>>) {
    let mut executor = Executor::new();

    for node in nodes {
        match *node {
            Node::Main { children } => run_stmts(&mut executor, &children),
            _ => unimplemented!(),
        }
    }
}

pub fn runtime_err(message: String) -> ! {
    println!("Runtime error: {}", message);
    panic!()
}

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

use crate::parser::{Node, VariableType};
use std::any::Any;
use std::collections::HashMap;
use std::io;

pub fn run(mut nodes: Vec<Box<Node>>) {
    let mut g_state = GlobalState {
        variables: HashMap::new(),
    };
    nodes.reverse();

    loop {
        let node = nodes.pop();

        if node.is_none() {
            break;
        }

        let node = node.unwrap();

        match *node {
            Node::Declare { t, children } => declare(&mut g_state, t, &children),
            Node::Output { children } => output(&g_state, children),
            Node::Input { child } => input(&mut g_state, child),
            _ => (),
        }
    }
}

fn declare(g_state: &mut GlobalState, t: VariableType, identifiers: &Vec<String>) {
    for name in identifiers.iter() {
        if g_state.variables.get(name).is_some() {
            eprintln!("Variable {} has already been declared before", name);
            panic!();
        }
        g_state.variables.insert(
            name.clone(),
            Variable {
                value: Box::new(()),
                t: t.clone(),
            },
        );
    }
}

fn input(g_state: &mut GlobalState, child: Box<Node>) {
    match *child {
        Node::Var(identifier) => {
            let mut temp = String::new();
            io::stdin()
                .read_line(&mut temp)
                .expect("Failed to read input");
            temp = temp.strip_suffix("\r\n").or(temp.strip_suffix("\n")).unwrap().parse().unwrap();
            match g_state.variables.get(&identifier) {
                None => {
                    eprintln!("Variable with name {} has not been declared", identifier);
                    panic!();
                }
                Some(v) => match v.t {
                    VariableType::Integer => {
                        g_state.variables.insert(
                            identifier,
                            Variable {
                                t: v.t.clone(),
                                value: Box::from(temp.parse::<i64>().unwrap_or(0)),
                            },
                        );
                    }
                    VariableType::String => {
                        g_state.variables.insert(
                            identifier,
                            Variable {
                                t: v.t.clone(),
                                value: Box::new(temp),
                            },
                        );
                    },
                    _ => {},
                },
            }
        }
        _ => {
            eprintln!("Expected an identifier");
            panic!();
        }
    }
}

fn output(g_state: &GlobalState, children: Vec<Box<Node>>) {
    for e in children.into_iter() {
        match *e {
            Node::Int(value) => print!("{}", value.to_string()),
            Node::String(value) => print!("{}", value),
            Node::Var(identifier) => match g_state.variables.get(&identifier) {
                Some(v) => match v.t {
                    VariableType::Integer => {
                        print!("{}", v.value.downcast_ref::<i64>().unwrap());
                    }
                    VariableType::String => {
                        print!("{}", v.value.downcast_ref::<String>().unwrap());
                    }
                    _ => {}
                },
                None => {
                    eprintln!("Variable {} has not been declared", identifier);
                    panic!();
                }
            },
            _ => (),
        }
    }
    println!();
}

struct GlobalState {
    variables: HashMap<String, Variable>,
}
#[derive(Debug)]
struct Variable {
    value: Box<dyn Any>,
    t: VariableType,
}

use std::ops::Deref;

use crate::enums::{Node, Position, VariableType};

use crate::executor::run_expr::run_expr;
use crate::executor::runtime_err;
use crate::executor::variable::Executor;

pub fn run_output(executor: &mut Executor, exprs: &Vec<Box<Node>>) {
    for expr in exprs {
        let res = run_expr(executor, expr);
        match *res {
            Node::Int { val, .. } => print!("{}", val.to_string()),
            Node::Real { val, .. } => print!("{}", val.to_string()),
            Node::String { val, .. } => print!("{}", val),
            Node::Boolean { val, .. } => print!("{}", val.to_string().to_uppercase()),
            Node::Date {val, ..} => print!("{}", val.format("%d-%m-%Y").to_string()),
            Node::Null => print!("null"),
            _ => unimplemented!(),
        }
    }
    print!("\n")
}

pub fn run_input(executor: &mut Executor, child: &Box<Node>) {
    if let Node::Var { name, .. } = child.deref() {
        let mut temp = String::new();
        std::io::stdin()
            .read_line(&mut temp)
            .expect("Failed to read input");
        temp = temp
            .strip_suffix("\r\n")
            .or(temp.strip_suffix("\n"))
            .unwrap()
            .parse()
            .unwrap();
        match executor.get_var(name).t {
            VariableType::Integer => executor.set_var(
                name,
                Box::new(Node::Int {
                    val: temp.parse::<i64>().unwrap_or(0),
                    pos: Position::invalid(),
                }),
            ),
            VariableType::String => executor.set_var(
                name,
                Box::new(Node::String {
                    val: temp,
                    pos: Position::invalid(),
                }),
            ),
            _ => runtime_err("Invalid type".to_string()),
        };
    }
    runtime_err("Invalid input statement".to_string());
}

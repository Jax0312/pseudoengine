use crate::enums::Node;

use crate::executor::run_expr::run_expr;

use super::variable::Executor;

pub fn run_output(executor: &mut Executor, exprs: &Vec<Box<Node>>) {
    for expr in exprs {
        let res = run_expr(executor, expr);
        match *res {
            Node::Int { val, .. } => print!("{}", val.to_string()),
            Node::Real { val, .. } => print!("{}", val.to_string()),
            Node::String { val, .. } => print!("{}", val),
            _ => (),
        }
        print!("\n")
    }
}

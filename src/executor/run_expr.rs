use std::ops::{Deref, DerefMut};

use crate::enums::{Index, Node, Position, VariableType};
use crate::executor::run_stmt::{as_number_expr, run_stmt};
use crate::executor::variable::{get_def, Definition, Executor, Property};
use crate::executor::{runtime_err, var_type_of};
use crate::executor::builtin_func_def::*;

pub fn run_expr(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::Expression(exprs) = node.deref() {
        let mut stack = Vec::<Box<Node>>::new();
        for expr in exprs {
            let expr = match expr.deref() {
                Node::Op { op, .. } => run_op(&mut stack, op),
                Node::Var { name, .. } => run_var(executor, name),
                Node::FunctionCall { name, params } => run_fn_call(executor, name, params),
                Node::ArrayVar { name, indices, .. } => run_array_var(executor, name, indices),
                Node::CreateObject(object) => run_create_obj(executor, expr),
                Node::RefVar(node) => run_ref_var(executor, node),
                Node::Composite { children } => run_composite(executor, children),
                _ => expr.clone(),
            };
            stack.push(expr);
        }
        return stack.first().unwrap().clone();
    } else if let Node::CreateObject(object) = node.deref() {
        return run_create_obj(executor, object);
    }
    unimplemented!()
}

fn run_array_var(executor: &mut Executor, name: &String, indices: &Vec<Box<Node>>) -> Box<Node> {
    let node = executor.get_var(name).value.clone();
    run_array_var_inner(executor, &node, indices)
}

fn run_array_var_inner(
    executor: &mut Executor,
    node: &Box<Node>,
    indices: &Vec<Box<Node>>,
) -> Box<Node> {
    let indices = indices
        .iter()
        .map(|index| as_number_expr(executor, index))
        .collect::<Vec<i64>>();
    if let Node::Array { values, shape, .. } = node.deref() {
        return values[get_array_index(indices, shape)].clone();
    };
    runtime_err("Invalid array access".to_string())
}

pub fn get_array_index(indices: Vec<i64>, shape: &Vec<Index>) -> usize {
    let mut size = 1;
    let mut total_index = 0;
    if indices.len() != shape.len() {
        runtime_err("Missing indices".to_string())
    }
    for (shape, index) in shape.iter().zip(indices).rev() {
        // bound check
        if index < shape.lower || index > shape.upper {
            runtime_err(format!("Index out of bounds: {} is not in range of {}..{}", index, shape.lower, shape.upper));
        }
        // 1D index calculation, bounds are inclusive hence +1
        total_index += (index - shape.lower) * size;
        size = size * (shape.upper - shape.lower + 1);
    }
    total_index as usize
}

fn run_var(executor: &mut Executor, name: &String) -> Box<Node> {
    executor.get_var(name).value.clone()
}

fn run_ref_var(executor: &mut Executor, node: &*mut Box<Node>) -> Box<Node> {
    unsafe { (**node).clone() }
}

fn run_op(stack: &mut Vec<Box<Node>>, op: &String) -> Box<Node> {
    let op = op.as_str();
    match op {
        "+" | "-" | "*" | "/" | "%" | "//" => run_arithmetic_op(stack, op),
        "_+" | "_-" => run_unary_op(stack, op),
        "<" | ">" | "<=" | ">=" => run_comparison_op(stack, op),
        "=" | "!=" => run_eq_op(stack, op),
        "&&" | "||" | "!" => run_logical_op(stack, op),
        "&" => run_concat_op(stack),
        _ => unimplemented!(),
    }
}

fn run_concat_op(stack: &mut Vec<Box<Node>>) -> Box<Node> {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
    
    if var_type_of(&rhs) == VariableType::String && var_type_of(&rhs) == var_type_of(&lhs) {
        Box::from(Node::String { val: format!("{}{}", lhs.val_as_str(), rhs.val_as_str()), pos: Position::invalid() })
    } else {
        runtime_err("'&' can only be performed on STRING".to_string())
    }
    
}

// function for equality op
fn run_eq_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
    
    match lhs.deref() {
        Node::Int {..} | Node::Real{..} | Node::Boolean {..} | Node::String {..} | Node::Date {..} | Node::EnumVal {..} => {},
        Node::Array {..} => runtime_err(format!("Equality check between ARRAY is not a valid operation {}", crate::utils::SUPPORT_MESSAGE)),
        _ => unimplemented!(),
    }
    
    if var_type_of(&rhs) == var_type_of(&lhs) {
        
        // for non-nested structures
        Box::from(Node::Boolean {
            val: match op {
                "=" => rhs.val_as_str() == lhs.val_as_str(),
                "!=" => rhs.val_as_str() != lhs.val_as_str(),
                _=> unreachable!(),
            }, 
            pos: Position::invalid(),
        })
    } else {
        runtime_err("Type mismatch during equality check".to_string())
    }
    
}

fn run_logical_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
    let rhs = stack.pop().expect("Invalid operation");
    let (rhs_val, is_bool) = assert_boolean(&rhs);
    if !is_bool {
        runtime_err(format!("Logical operation {} can only be performed on BOOLEAN", op))
    }

    Box::from(match op {
        "!" => Node::Boolean {val: !rhs_val, pos: Position::invalid()},
        _ => {
            let lhs = stack.pop().expect("Invalid operation");
            let (lhs_val, is_bool) = assert_boolean(&lhs);
            if !is_bool {
                runtime_err(format!("Logical operation {} can only be performed on BOOLEAN", op))
            }
            match op {
                "&&" => Node::Boolean {val: lhs_val && rhs_val, pos: Position::invalid()},
                "||" => Node::Boolean {val: lhs_val || rhs_val, pos: Position::invalid()},
                _ => unreachable!(),
            }
        },
    })
}

fn run_unary_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
    let node = stack.pop().expect("Invalid operation");
    let (val, is_real) = assert_number(&node);

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

fn run_arithmetic_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
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

fn run_comparison_op(stack: &mut Vec<Box<Node>>, op: &str) -> Box<Node> {
    let rhs = stack.pop().expect("Invalid operation");
    let lhs = stack.pop().expect("Invalid operation");
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
        _ => runtime_err("operation can only be performed on numbers".to_string()),
    }
}

fn assert_boolean(node: &Box<Node>) -> (bool, bool) {
    match *node.deref() {
        Node::Boolean { val, .. } => (val, true),
        _ => (false, false),
    }
}

fn node_is_int(executor: &mut Executor, node: Box<Node>) -> bool {
    let expr = run_expr(executor, &node);
    var_type_of(&expr) == VariableType::Integer
}

fn run_fn_call(executor: &mut Executor, name: &String, call_params: &Vec<Box<Node>>) -> Box<Node> {
    match name.to_uppercase().as_str() {
        "LEFT" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String, VariableType::Integer], &builtin_func_left),
        "RIGHT" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String, VariableType::Integer], &builtin_func_right),
        "MID" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String, VariableType::Integer, VariableType::Integer], &builtin_func_mid),
        "LENGTH" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_length),
        "TO_UPPER" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_to_upper),
        "TO_LOWER" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_to_lower),
        "NUM_TO_STR" => {
            if node_is_int(executor, call_params[0].clone()) {
                return run_fn_call_builtin(executor, call_params, &vec![VariableType::Integer], &builtin_func_num_to_str)
            } return run_fn_call_builtin(executor, call_params, &vec![VariableType::Real], &builtin_func_num_to_str)
        },
        "STR_TO_NUM" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_str_to_num),
        "IS_NUM" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_is_num),
        "ASC" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::String], &builtin_func_asc),
        "CHR" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Integer], &builtin_func_chr),
        "INT" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Real], &builtin_func_int),
        "RAND" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Integer], &builtin_func_rand),
        "DAY" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Date], &builtin_func_day),
        "MONTH" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Date], &builtin_func_month),
        "YEAR" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Date], &builtin_func_year),
        "DAYINDEX" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Date], &builtin_func_day_index),
        "SETDATE" => return run_fn_call_builtin(executor, call_params, &vec![VariableType::Integer, VariableType::Integer, VariableType::Integer], &builtin_func_set_date),
        "TODAY" => return run_fn_call_builtin(executor, call_params, &vec![], &builtin_func_today),
        "EOF" => todo!(),
        _ => {}
    }

    if let Definition::Function { params, children } = get_def(&mut executor.defs, name) {
        return run_fn_call_inner(executor, call_params, &params, &children, true);
    }
    runtime_err("Invalid function call".to_string())
}

fn run_fn_call_builtin(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<VariableType>,
    func: &dyn Fn(&Vec<String>) -> Box<Node>,
) -> Box<Node> {
    let mut values = Vec::<String>::new();
    if call_params.len() != fn_params.len() {
        runtime_err("Invalid number of arguments".to_string())
    }
    for (call_param, fn_param) in call_params.iter().zip(fn_params.iter()) {
        let expr = run_expr(executor, call_param);
        if var_type_of(&expr) == *fn_param {
            values.push(expr.val_as_str());
        } else {
            runtime_err("Parameter type mismatch".to_string())
        }
    }

    func(&values)
}

fn run_fn_call_inner(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<Box<Node>>,
    children: &Vec<Box<Node>>,
    returns: bool,
) -> Box<Node> {
    executor.enter_scope();
    if fn_params.len() != call_params.len() {
        runtime_err("Missing parameters".to_string())
    }
    for (call_param, fn_param) in call_params.iter().zip(fn_params) {
        if let Node::Declare { t, children } = fn_param.deref() {
            let expr = run_expr(executor, call_param);
            if var_type_of(&expr) == *t.deref() {
                executor.declare_var(&children[0], expr, t, true);
            } else {
                runtime_err("Parameter type mismatch".to_string())
            }
        } else if let Node::Reference(reference) = fn_param.deref() {
            if let Node::Declare { t, children } = reference.deref() {
                if let Node::Expression(call_param) = call_param.deref() {
                    if let Node::Var { name, .. } = call_param[0].deref() {
                        let variable = executor.get_var_mut(name);
                        if variable.t == *t.deref() {
                            let value = Box::new(Node::RefVar(&mut variable.value as *mut Box<Node>));
                            executor.declare_var(&children[0], value, t, true);
                        } else {
                            runtime_err("Parameter type mismatch".to_string())
                        }
                    } else {
                        runtime_err("Reference parameter must be a variable".to_string())
                    }
                }
            }
        }
    }
    for child in children {
        match child.deref() {
            Node::Return(expr) => {
                let expr = run_expr(executor, expr);
                executor.exit_scope();
                return expr;
            }
            _ => run_stmt(executor, &child),
        }
    }
    if returns {
        runtime_err("Missing return statement".to_string())
    } else {
        executor.exit_scope();
        Box::new(Node::Null)
    }
}

fn run_create_obj(executor: &mut Executor, node: &Box<Node>) -> Box<Node> {
    if let Node::FunctionCall { params, name } = node.deref() {
        if let Definition::Class { props, .. } = get_def(&mut executor.defs, name) {
            if let Some(Property::Procedure {
                private,
                params: fn_params,
                children,
            }) = props.get("new")
            {
                if !private {
                    executor.enter_scope();
                    for (name, prop) in props.iter() {
                        if let Property::Var { value, t, .. } = prop {
                            executor.declare_var(name, value.clone(), t, true);
                        }
                    }
                    run_fn_call_inner(executor, params, fn_params, children, false);
                    let mut props = props.clone();
                    for (name, prop) in props.iter_mut() {
                        if let Property::Var { value, .. } = prop {
                            *value = executor.get_var(name).value.clone()
                        }
                    }
                    executor.exit_scope();
                    return Box::new(Node::Object{ name: name.clone(), props });
                }
                runtime_err("Constructor cannot be private".to_string())
            }
            runtime_err("Constructor is not defined".to_string())
        }
        runtime_err(format!("{} is not a class", name))
    }
    unreachable!()
}

fn run_composite(executor: &mut Executor, children: &Vec<Box<Node>>) -> Box<Node> {
    let mut base = match children[0].deref() {
        Node::Var { name, .. } => run_var(executor, name),
        Node::FunctionCall { name, params } => run_fn_call(executor, name, params),
        Node::RefVar(node) => run_ref_var(executor, node),
        Node::ArrayVar { name, indices, .. } => run_array_var(executor, name, indices),
        _ => runtime_err("Invalid base property access".to_string()),
    };
    for child in children.iter().skip(1) {
        base = match child.deref() {
            Node::Var { name, .. } => run_prop_access(executor, &base, name),
            Node::ArrayVar { name, indices, .. } => {
                run_prop_arr_access(executor, &base, name, indices)
            }
            Node::FunctionCall { name, params } => {
                run_method_call(executor, &mut base, name, params)
            }
            _ => runtime_err("Invalid property access".to_string()),
        };
    }
    base
}

fn run_prop_access(
    executor: &mut Executor,
    base: &Box<Node>,
    name: &String
) -> Box<Node> {
    if let Node::Object{ props, .. } = base.deref() {
        if let Some(Property::Var { value, .. }) = props.get(name) {
            return value.clone();
        }
    }
    runtime_err("Invalid property access".to_string())
}

fn run_prop_arr_access(
    executor: &mut Executor,
    base: &Box<Node>,
    name: &String,
    indices: &Vec<Box<Node>>,
) -> Box<Node> {
    if let Node::Object{ props, .. } = base.deref() {
        if let Some(Property::Var { value, .. }) = props.get(name) {
            return run_array_var_inner(executor, value, indices);
        }
    }
    runtime_err("Invalid property access".to_string())
}

fn run_method_call(
    executor: &mut Executor,
    base: &mut Box<Node>,
    name: &String,
    call_params: &Vec<Box<Node>>,
) -> Box<Node> {
    if let Node::Object{ props, .. } = base.deref_mut() {
            executor.enter_scope();
        for (name, prop) in &mut props.iter_mut() {
            if let Property::Var { value, t, .. } = prop {
                let value = Box::new(Node::RefVar(value as *mut Box<Node>));
                executor.declare_var(name, value, &t, true);
            }
        }
        if let Some(Property::Procedure {
            params: fn_params,
            children,
            ..
        }) = props.get(name)
        {
            run_fn_call_inner(executor, call_params, fn_params, children, false);
            executor.exit_scope();
            return Box::new(Node::Null);
        }
    }
    runtime_err("Invalid property access".to_string())
}

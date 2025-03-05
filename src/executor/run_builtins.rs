use crate::enums::{Node, Position, VariableType};
use crate::executor::run_expr::run_expr;
use crate::executor::variable::Executor;
use crate::executor::{runtime_err, var_type_of};
use chrono::{Datelike, NaiveDate};

pub fn match_builtin(
    executor: &mut Executor,
    name: &String,
    call_params: &Vec<Box<Node>>,
) -> Option<Box<Node>> {
    Some(match name.to_uppercase().as_str() {
        "LEFT" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String, VariableType::Integer],
            &builtin_func_left,
        ),
        "RIGHT" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String, VariableType::Integer],
            &builtin_func_right,
        ),
        "MID" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![
                VariableType::String,
                VariableType::Integer,
                VariableType::Integer,
            ],
            &builtin_func_mid,
        ),
        "LENGTH" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_length,
        ),
        "TO_UPPER" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_to_upper,
        ),
        "TO_LOWER" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_to_lower,
        ),
        "NUM_TO_STR" => {
            let t = var_type_of(&run_expr(executor, &call_params[0].clone()));
            if t == VariableType::Integer {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Integer],
                    &builtin_func_num_to_str,
                )
            } else {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Real],
                    &builtin_func_num_to_str,
                )
            }
        }
        "STR_TO_NUM" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_str_to_num,
        ),
        "IS_NUM" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_is_num,
        ),
        "ASC" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_asc,
        ),
        "CHR" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Integer],
            &builtin_func_chr,
        ),
        "INT" => {
            let t = var_type_of(&run_expr(executor, &call_params[0].clone()));
            if t == VariableType::Integer {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Integer],
                    &builtin_func_int,
                )
            } else {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Real],
                    &builtin_func_int,
                )
            }
        }
        "RAND" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Integer],
            &builtin_func_rand,
        ),
        "DAY" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_day,
        ),
        "MONTH" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_month,
        ),
        "YEAR" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_year,
        ),
        "DAYINDEX" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_day_index,
        ),
        "SETDATE" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![
                VariableType::Integer,
                VariableType::Integer,
                VariableType::Integer,
            ],
            &builtin_func_set_date,
        ),
        "TODAY" => run_fn_call_builtin(executor, call_params, &vec![], &builtin_func_today),
        "EOF" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_eof,
        ),
        _ => return None,
    })
}

fn run_fn_call_builtin(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<VariableType>,
    func: &dyn Fn(&mut Executor, &Vec<String>) -> Box<Node>,
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

    func(executor, &values)
}

pub fn builtin_func_left(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'LEFT' function cannot be less than 0".to_string()),
    };

    if length > operand.len() {
        runtime_err("Length for 'LEFT' function cannot exceed string length".to_string())
    }

    Box::new(Node::String {
        val: operand[..length].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_right(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'RIGHT' function cannot be less than 0".to_string()),
    };

    if length > operand.len() {
        runtime_err("Length for 'RIGHT' function cannot exceed string length".to_string())
    }

    Box::new(Node::String {
        val: operand[operand.len() - length..].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_mid(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    let start = match params[1].clone().parse::<usize>() {
        Ok(length) => {
            if length > 0 {
                length
            } else {
                runtime_err(
                    "Starting position for 'MID' function cannot be less than 1".to_string(),
                )
            }
        }
        Err(_) => {
            runtime_err("Starting position for 'MID' function cannot be less than 1".to_string())
        }
    };
    let length = match params[2].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'MID' function cannot be less than 0".to_string()),
    };

    if start + length > operand.len() + 1 {
        runtime_err("Substring length for 'MID' function cannot exceed string length".to_string())
    }

    Box::new(Node::String {
        val: operand[start - 1..start + length - 1].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_length(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::Int {
        val: operand.len() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_to_upper(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String {
        val: operand.to_uppercase(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_to_lower(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String {
        val: operand.to_lowercase(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_num_to_str(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    Box::new(Node::String {
        val: params[0].clone(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_str_to_num(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    match params[0].clone().parse::<i64>() {
        Ok(number) => Box::new(Node::Int {
            val: number,
            pos: Position::invalid(),
        }),
        Err(_) => match params[0].clone().parse::<f64>() {
            Ok(number) => Box::new(Node::Real {
                val: number,
                pos: Position::invalid(),
            }),
            Err(_) => runtime_err("String is not a valid number".to_string()),
        },
    }
}

pub fn builtin_func_is_num(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    Box::new(match params[0].clone().parse::<f64>() {
        Ok(_) => Node::Boolean {
            val: true,
            pos: Position::invalid(),
        },
        Err(_) => Node::Boolean {
            val: false,
            pos: Position::invalid(),
        },
    })
}

pub fn builtin_func_asc(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    if operand.len() != 1 {
        runtime_err("String length for 'ASC' function must be 1".to_string())
    }
    Box::new(Node::Int {
        val: operand.chars().last().unwrap() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_chr(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let ascii = match params[0].parse::<u8>() {
        Ok(ascii) => ascii,
        Err(_) => runtime_err(
            "Ascii value for 'CHR' function must be between 0-255 inclusive".to_string(),
        ),
    };
    Box::new(Node::String {
        val: (ascii as char).to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_int(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let operand = params[0].parse::<f64>().unwrap();
    Box::new(Node::Int {
        val: operand.trunc() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_rand(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let upper = params[0].parse::<i64>().unwrap();

    if upper < 1 {
        runtime_err("Number for 'RAND' function cannot be less than 1".to_string());
    }

    Box::new(Node::Real {
        val: rand::random_range(0.0..(upper as f64)),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_day(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.day() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_month(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.month() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_year(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.year() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_day_index(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    // Sunday is 1 for CIE
    Box::new(Node::Int {
        val: (date.weekday().num_days_from_sunday() + 1) as i64,
        pos: Position::invalid(),
    })
}
pub fn builtin_func_set_date(_: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let day = params[0].clone();
    let month = params[1].clone();
    let year = params[2].clone();

    match NaiveDate::parse_from_str(&format!("{}/{}/{}", year, month, day), "%Y/%m/%d") {
        Ok(date) => Box::new(Node::Date {
            val: date,
            pos: Position::invalid(),
        }),
        Err(_) => runtime_err("Date given is not valid".to_string()),
    }
}

pub fn builtin_func_today(_: &mut Executor, _: &Vec<String>) -> Box<Node> {
    Box::new(Node::Date {
        val: chrono::offset::Local::now().date_naive(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_eof(executor: &mut Executor, params: &Vec<String>) -> Box<Node> {
    let filename = params[0].clone();
    match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "READ" {
                runtime_err("Function EOF only works on file opened for READ".to_string());
            }
            Box::from(Node::Boolean {
                val: file.cursor > file.content.len(),
                pos: Position::invalid(),
            })
        }
    }
}

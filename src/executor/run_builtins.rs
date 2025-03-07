use crate::enums::{Node, Position, VariableType};
use crate::executor::run_expr::run_expr;
use crate::executor::variable::Executor;
use crate::executor::{err, var_type_of};
use chrono::{Datelike, NaiveDate};

pub fn match_builtin(
    executor: &mut Executor,
    name: &String,
    call_params: &Vec<Box<Node>>,
    pos: &Position,
) -> Option<Box<Node>> {
    Some(match name.to_uppercase().as_str() {
        "LEFT" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String, VariableType::Integer],
            &builtin_func_left,
            pos,
        ),
        "RIGHT" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String, VariableType::Integer],
            &builtin_func_right,
            pos,
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
            pos,
        ),
        "LENGTH" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_length,
            pos,
        ),
        "TO_UPPER" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_to_upper,
            pos,
        ),
        "TO_LOWER" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_to_lower,
            pos,
        ),
        "NUM_TO_STR" => {
            let t = var_type_of(&run_expr(executor, &call_params[0].clone()));
            if t == VariableType::Integer {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Integer],
                    &builtin_func_num_to_str,
                    pos,
                )
            } else {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Real],
                    &builtin_func_num_to_str,
                    pos,
                )
            }
        }
        "STR_TO_NUM" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_str_to_num,
            pos,
        ),
        "IS_NUM" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_is_num,
            pos,
        ),
        "ASC" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_asc,
            pos,
        ),
        "CHR" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Integer],
            &builtin_func_chr,
            pos,
        ),
        "INT" => {
            let t = var_type_of(&run_expr(executor, &call_params[0].clone()));
            if t == VariableType::Integer {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Integer],
                    &builtin_func_int,
                    pos,
                )
            } else {
                run_fn_call_builtin(
                    executor,
                    call_params,
                    &vec![VariableType::Real],
                    &builtin_func_int,
                    pos,
                )
            }
        }
        "RAND" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Integer],
            &builtin_func_rand,
            pos,
        ),
        "DAY" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_day,
            pos,
        ),
        "MONTH" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_month,
            pos,
        ),
        "YEAR" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_year,
            pos,
        ),
        "DAYINDEX" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::Date],
            &builtin_func_day_index,
            pos,
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
            pos,
        ),
        "TODAY" => run_fn_call_builtin(executor, call_params, &vec![], &builtin_func_today, pos),
        "EOF" => run_fn_call_builtin(
            executor,
            call_params,
            &vec![VariableType::String],
            &builtin_func_eof,
            pos,
        ),
        _ => return None,
    })
}

fn run_fn_call_builtin(
    executor: &mut Executor,
    call_params: &Vec<Box<Node>>,
    fn_params: &Vec<VariableType>,
    func: &dyn Fn(&mut Executor, &Vec<String>, &Vec<Box<Node>>) -> Box<Node>,
    pos: &Position,
) -> Box<Node> {
    let mut values = Vec::<String>::new();
    if call_params.len() != fn_params.len() {
        err("Invalid number of arguments", pos)
    }
    for (call_param, fn_param) in call_params.iter().zip(fn_params.iter()) {
        let expr = run_expr(executor, call_param);
        if var_type_of(&expr) == *fn_param {
            values.push(expr.val_as_str());
        } else {
            let msg = format!(
                "Cannot assign type {:?} to parameter of type {:?}",
                var_type_of(&expr).str(),
                fn_param.str()
            );
            err(msg.as_str(), &call_param.pos())
        }
    }

    func(executor, &values, call_params)
}

pub fn builtin_func_left(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => err(
            "Length for 'LEFT' function cannot be less than 0",
            &nodes[1].pos(),
        ),
    };

    if length > operand.len() {
        err(
            "Length for 'LEFT' function cannot exceed string length",
            &nodes[1].pos(),
        )
    }

    Box::new(Node::String {
        val: operand[..length].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_right(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => err(
            "Length for 'RIGHT' function cannot be less than 0",
            &nodes[1].pos(),
        ),
    };

    if length > operand.len() {
        err(
            "Length for 'RIGHT' function cannot exceed string length",
            &nodes[1].pos(),
        )
    }

    Box::new(Node::String {
        val: operand[operand.len() - length..].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_mid(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    let start = match params[1].clone().parse::<usize>() {
        Ok(length) => {
            if length > 0 {
                length
            } else {
                err(
                    "Starting position for 'MID' function cannot be less than 1",
                    &nodes[1].pos(),
                )
            }
        }
        Err(_) => err(
            "Starting position for 'MID' function cannot be less than 1",
            &nodes[1].pos(),
        ),
    };
    let length = match params[2].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => err(
            "Length for 'MID' function cannot be less than 0",
            &nodes[2].pos(),
        ),
    };

    if start + length > operand.len() + 1 {
        err(
            "Substring length for 'MID' function cannot exceed string length",
            &nodes[2].pos(),
        )
    }

    Box::new(Node::String {
        val: operand[start - 1..start + length - 1].to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_length(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::Int {
        val: operand.len() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_to_upper(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String {
        val: operand.to_uppercase(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_to_lower(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String {
        val: operand.to_lowercase(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_num_to_str(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
    Box::new(Node::String {
        val: params[0].clone(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_str_to_num(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
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
            Err(_) => err("String is not a valid number", &nodes[0].pos()),
        },
    }
}

pub fn builtin_func_is_num(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
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

pub fn builtin_func_asc(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let operand = params[0].clone();
    if operand.len() != 1 {
        err(
            "String length for 'ASC' function must be 1",
            &nodes[0].pos(),
        )
    }
    Box::new(Node::Int {
        val: operand.chars().last().unwrap() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_chr(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let ascii = match params[0].parse::<u8>() {
        Ok(ascii) => ascii,
        Err(_) => err(
            "Ascii value for 'CHR' function must be between 0-255 inclusive",
            &nodes[0].pos(),
        ),
    };
    Box::new(Node::String {
        val: (ascii as char).to_string(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_int(_: &mut Executor, params: &Vec<String>, _: &Vec<Box<Node>>) -> Box<Node> {
    let operand = params[0].parse::<f64>().unwrap();
    Box::new(Node::Int {
        val: operand.trunc() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_rand(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let upper = params[0].parse::<i64>().unwrap();

    if upper < 1 {
        err(
            "Number for 'RAND' function cannot be less than 1",
            &nodes[0].pos(),
        );
    }

    Box::new(Node::Real {
        val: rand::random_range(0.0..(upper as f64)),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_day(_: &mut Executor, params: &Vec<String>, _: &Vec<Box<Node>>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.day() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_month(_: &mut Executor, params: &Vec<String>, _: &Vec<Box<Node>>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.month() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_year(_: &mut Executor, params: &Vec<String>, _: &Vec<Box<Node>>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {
        val: date.year() as i64,
        pos: Position::invalid(),
    })
}

pub fn builtin_func_day_index(
    _: &mut Executor,
    params: &Vec<String>,
    _: &Vec<Box<Node>>,
) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    // Sunday is 1 for CIE
    Box::new(Node::Int {
        val: (date.weekday().num_days_from_sunday() + 1) as i64,
        pos: Position::invalid(),
    })
}
pub fn builtin_func_set_date(
    _: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let day = params[0].clone();
    let month = params[1].clone();
    let year = params[2].clone();

    match NaiveDate::parse_from_str(&format!("{}/{}/{}", year, month, day), "%Y/%m/%d") {
        Ok(date) => Box::new(Node::Date {
            val: date,
            pos: Position::invalid(),
        }),
        Err(_) => err(
            "Date given is not valid",
            &Position::range(nodes[0].pos(), nodes[2].pos()),
        ),
    }
}

pub fn builtin_func_today(_: &mut Executor, _: &Vec<String>, _: &Vec<Box<Node>>) -> Box<Node> {
    Box::new(Node::Date {
        val: chrono::offset::Local::now().date_naive(),
        pos: Position::invalid(),
    })
}

pub fn builtin_func_eof(
    executor: &mut Executor,
    params: &Vec<String>,
    nodes: &Vec<Box<Node>>,
) -> Box<Node> {
    let filename = params[0].clone();
    match executor.file_handles.get_mut(&filename) {
        None => err(
            format!("File {} is not opened", filename).as_str(),
            &nodes[0].pos(),
        ),
        Some(file) => {
            if file.mode != "READ" {
                err(
                    "Function EOF only works on file opened for READ",
                    &nodes[0].pos(),
                );
            }
            Box::from(Node::Boolean {
                val: file.cursor > file.content.len(),
                pos: Position::invalid(),
            })
        }
    }
}

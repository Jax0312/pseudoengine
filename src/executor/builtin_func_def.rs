use chrono::{Datelike, NaiveDate};
use crate::enums::{Node, Position};
use crate::executor::runtime_err;

pub fn builtin_func_left(params: &Vec<String>) -> Box<Node> {

    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'LEFT' function cannot be less than 0".to_string()),
    };

    if length > operand.len() {
        runtime_err("Length for 'LEFT' function cannot exceed string length".to_string())
    }

    Box::new(Node::String { val: operand[..length].to_string(), pos: Position::invalid()})

}

pub fn builtin_func_right(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    let length = match params[1].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'RIGHT' function cannot be less than 0".to_string()),
    };

    if length > operand.len() {
        runtime_err("Length for 'RIGHT' function cannot exceed string length".to_string())
    }

    Box::new(Node::String { val: operand[operand.len()-length..].to_string(), pos: Position::invalid()})

}

pub fn builtin_func_mid(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    let start = match params[1].clone().parse::<usize>() {
        Ok(length) => if length > 0 {
            length
        } else {
            runtime_err("Starting position for 'MID' function cannot be less than 1".to_string())
        },
        Err(_) => runtime_err("Starting position for 'MID' function cannot be less than 1".to_string()),
    };
    let length = match params[2].clone().parse::<usize>() {
        Ok(length) => length,
        Err(_) => runtime_err("Length for 'MID' function cannot be less than 0".to_string()),
    };

    if start + length > operand.len() {
        runtime_err("Substring length for 'MID' function cannot exceed string length".to_string())
    }

    Box::new(Node::String { val: operand[start-1..start+length-1].to_string(), pos: Position::invalid()})

}

pub fn builtin_func_length(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::Int {val: operand.len() as i64, pos: Position::invalid()})
}

pub fn builtin_func_to_upper(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String { val: operand.to_uppercase(), pos: Position::invalid()})
}

pub fn builtin_func_to_lower(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    Box::new(Node::String { val: operand.to_lowercase(), pos: Position::invalid()})
}

pub fn builtin_func_num_to_str(params: &Vec<String>) -> Box<Node> {
    Box::new(Node::String {val:params[0].clone(), pos: Position::invalid()})
}

pub fn builtin_func_str_to_num(params: &Vec<String>) -> Box<Node> {
    match params[0].clone().parse::<i64>() {
        Ok(number) => Box::new(Node::Int {val: number, pos: Position::invalid()}),
        Err(_) => match params[0].clone().parse::<f64>() { 
            Ok(number) => Box::new(Node::Real {val: number, pos: Position::invalid()}),
            Err(_) => runtime_err("String is not a valid number".to_string()),
        }
    }
}

pub fn builtin_func_is_num(params: &Vec<String>) -> Box<Node> {
    Box::new(match params[0].clone().parse::<f64>() {
        Ok(_) => Node::Boolean {val: true, pos: Position::invalid()},
        Err(_) => Node::Boolean {val: false, pos: Position::invalid()}
    })
}

pub fn builtin_func_asc(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].clone();
    if operand.len() != 1 {
        runtime_err("String length for 'ASC' function must be 1".to_string())
    }
    Box::new(Node::Int {val: operand.chars().last().unwrap() as i64, pos: Position::invalid()})
}

pub fn builtin_func_chr(params: &Vec<String>) -> Box<Node> {
    let ascii = match params[0].parse::<u8>() {
        Ok(ascii) => ascii,
        Err(_) => runtime_err("Ascii value for 'CHR' function must be between 0-255 inclusive".to_string()),
    };
    Box::new(Node::String {val: (ascii as char).to_string(), pos: Position::invalid()})
}

pub fn builtin_func_int(params: &Vec<String>) -> Box<Node> {
    let operand = params[0].parse::<f64>().unwrap();
    Box::new(Node::Int {val: operand.trunc() as i64, pos: Position::invalid()})
}

pub fn builtin_func_rand(params: &Vec<String>) -> Box<Node> {
    let upper = params[0].parse::<i64>().unwrap();

    if upper < 1 {
        runtime_err("Number for 'RAND' function cannot be less than 1".to_string());
    }

    Box::new(Node::Real {val: rand::random_range(0.0..(upper as f64)), pos: Position::invalid()})
}

pub fn builtin_func_day(params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {val: date.day() as i64, pos:Position::invalid()})
}

pub fn builtin_func_month(params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {val: date.month() as i64, pos:Position::invalid()})
}

pub fn builtin_func_year(params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    Box::new(Node::Int {val: date.year() as i64, pos:Position::invalid()})
}

pub fn builtin_func_day_index(params: &Vec<String>) -> Box<Node> {
    let date = NaiveDate::parse_from_str(params[0].as_str(), "%Y-%m-%d").unwrap();
    // Sunday is 1 for CIE
    Box::new(Node::Int {val: (date.weekday().num_days_from_sunday() + 1) as i64, pos:Position::invalid()})
}
pub fn builtin_func_set_date(params: &Vec<String>) -> Box<Node> {

    let day = params[0].clone();
    let month = params[1].clone();
    let year = params[2].clone();
    
    match NaiveDate::parse_from_str(&format!("{}/{}/{}", year, month, day), "%Y/%m/%d") {
        Ok(date) => Box::new(Node::Date {val: date, pos:Position::invalid()}),
        Err(_) => runtime_err("Date given is not valid".to_string()),
    }
}

pub fn builtin_func_today(params: &Vec<String>) -> Box<Node> {
    Box::new(Node::Date {val: chrono::offset::Local::now().date_naive(), pos:Position::invalid()})
}



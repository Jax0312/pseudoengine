use crate::enums::{Array, Node, NodeRef, Position, VariableType};
use crate::executor::run_expr::{assert_number, run_expr};
use crate::executor::run_stmt::run_assign;
use crate::executor::variable::{Executor, XFile};
use crate::executor::{runtime_err, Property};
use crate::tokens::TToken;
use chrono::NaiveDate;
use serde_json::{Map, Number, Value};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem::discriminant;
use std::ops::{Deref, DerefMut};

use super::variable::NodeDeref;

pub fn run_open_file(executor: &mut Executor, filename: &Box<Node>, mode: &TToken) {
    let filename = run_expr(executor, filename).val_as_str();
    if let TToken::FileMode(mode) = mode {
        match executor.file_handles.get(&filename) {
            Some(_) => runtime_err(format!("File {} is already open", filename)),
            None => {
                let mut content = vec![];
                executor.file_handles.insert(
                    filename.clone(),
                    XFile {
                        handle: match mode.as_str() {
                            "APPEND" => OpenOptions::new().append(true).open(filename).expect(""),
                            "WRITE" => OpenOptions::new()
                                .write(true)
                                .create(true)
                                .truncate(true)
                                .open(filename)
                                .expect(""),
                            "READ" | "RANDOM" => {
                                let mut file = OpenOptions::new()
                                    .write(true)
                                    .read(true)
                                    .create(true)
                                    .append(false)
                                    .open(filename.clone())
                                    .expect("");
                                let mut buf = String::new();
                                file.read_to_string(&mut buf)
                                    .expect(format!("{} contains invalid data", filename).as_str());
                                content = buf.lines().map(|s| s.to_string()).collect();
                                file
                            }
                            _ => unreachable!(),
                        },
                        mode: mode.to_string(),
                        content,
                        cursor: 1,
                    },
                );
            }
        }
    } else {
        unreachable!()
    }
}

pub fn run_close_file(executor: &mut Executor, filename: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let mut file = match executor.file_handles.remove(&filename) {
        None => runtime_err(format!("File {} is already closed", filename)),
        Some(file) => file,
    };
    // commit changes as PUTRECORD does not write to file immediately
    if file.mode == "RANDOM" {
        file.handle.seek(SeekFrom::Start(0)).expect("");
        file.handle
            .write_all(file.content.join("\n").as_bytes())
            .expect("");
    }
}

pub fn run_write_file(executor: &mut Executor, filename: &Box<Node>, data: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let data = run_expr(executor, data).val_as_str() + "\n";
    match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "APPEND" && file.mode != "WRITE" {
                runtime_err(format!(
                    "File {} is not opened for APPEND or WRITE",
                    filename
                ))
            }
            file.handle.write_all(data.as_bytes()).expect("");
        }
    }
}

pub fn run_read_file(executor: &mut Executor, filename: &Box<Node>, destination: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();

    match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "READ" {
                runtime_err(format!("File {} is not opened for READ", filename))
            }

            let content = Box::from(Node::Expression(vec![Box::from(Node::String {
                val: if file.cursor <= file.content.len() {
                    file.content[file.cursor - 1].clone()
                } else {
                    String::new()
                },
                pos: Position::invalid(),
            })]));
            file.cursor += 1;
            run_assign(executor, destination, &content);
        }
    }
}

pub fn run_get_record(executor: &mut Executor, filename: &Box<Node>, destination: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let dest_content = run_expr(
        executor,
        &Box::from(Node::Expression(vec![destination.clone()])),
    );
    let file = match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "RANDOM" {
                runtime_err("GETRECORD only works for file opened for RANDOM".to_string())
            }
            file
        }
    };

    let mut data: Map<String, Value> = match file.content.get(file.cursor - 1) {
        Some(str) => serde_json::from_str(str).unwrap(),
        None => runtime_err(format!(
            "Line {} of file {} is empty",
            file.cursor, filename
        )),
    };

    if let Node::Object {
        name: _name,
        props: _props,
    } = dest_content.deref()
    {
        let name = _name.clone();
        let mut props = _props.clone();
        deserialise_record(&mut props, &mut data);
        run_assign(
            executor,
            &destination,
            &Box::from(Node::Expression(vec![Box::from(Node::Object {
                name,
                props,
            })])),
        );
    }
}

pub fn run_seek(executor: &mut Executor, filename: &Box<Node>, position: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let (position, _) = assert_number(&run_expr(executor, position));

    let file = match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "RANDOM" {
                runtime_err("SEEK only works for file opened for RANDOM".to_string())
            }
            file
        }
    };

    if position <= 0.0 {
        runtime_err("Invalid position. Line number must be greater than 0".to_string())
    }

    file.cursor = position as usize;
}

pub fn run_put_record(executor: &mut Executor, filename: &Box<Node>, data: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let data = run_expr(executor, data);

    let file = match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {
            if file.mode != "RANDOM" {
                runtime_err("PUTRECORD only works for file opened for RANDOM".to_string())
            }
            file
        }
    };
    let json_string;
    if let Node::Object { props, .. } = data.deref() {
        json_string = serde_json::to_string(&serialise_record(props)).unwrap();
    } else {
        runtime_err("Invalid data type. RECORD data type expected".to_string())
    };

    // fill skipped lines
    while file.cursor > file.content.len() {
        file.content.push("\n".to_string());
    }
    file.content[file.cursor - 1] = json_string;
}

fn deserialise_record(props: &HashMap<String, Property>, data: &mut Map<String, Value>) {
    props.iter().for_each(|(k, property)| {
        match data.get_mut(k) {
            Some(dv) => {
                if let Property::Var { value, ref t, .. } = property {
                    value.replace(Box::from(
                        if let Node::Object { name, props } = &value.clone_node().deref() {
                            // handle nested record type
                            if let Value::Object(map) = data.get_mut(name).unwrap() {
                                deserialise_record(props, map);
                                Node::Object {
                                    name: name.clone(),
                                    props: props.clone(),
                                }
                            } else {
                                unreachable!()
                            }
                        } else {
                            // convert Value to VariableType equivalent
                            let dv_type = match dv {
                                Value::Number(val) => {
                                    if val.is_i64() {
                                        VariableType::Integer
                                    } else {
                                        VariableType::Real
                                    }
                                }
                                Value::String(val) => {
                                    match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
                                        Ok(..) => VariableType::Date,
                                        Err(_) => VariableType::String,
                                    }
                                }
                                Value::Bool(..) => VariableType::Boolean,
                                // Content of Array does not matter
                                Value::Array(..) => VariableType::Array(Box::new(Array {
                                    t: Box::new(VariableType::Boolean),
                                    lower: 0,
                                    upper: 0,
                                })),
                                _ => unreachable!(),
                            };
                            if discriminant(&dv_type) != discriminant(&t) {
                                runtime_err("Property type mismatch".to_string())
                            }
                            value_to_node(value.borrow().deref(), dv, t.clone())
                        },
                    ));
                }
            }
            None => runtime_err(format!("Property {} is missing", k)),
        }
    });
}
fn serialise_record(props: &HashMap<String, Property>) -> Map<String, Value> {
    props
        .iter()
        .map(|field| {
            if let Property::Var { value, t, private } = field.1 {
                match t.deref() {
                    VariableType::Custom(..) => {
                        if let Node::Object { props, .. } = value.borrow().deref().deref() {
                            (field.0.clone(), Value::from(serialise_record(props)))
                        } else {
                            runtime_err("Invalid data type. RECORD data type expected".to_string())
                        }
                    }
                    _ => (
                        field.0.clone(),
                        Value::from(serialise_data(value.borrow().deref())),
                    ),
                }
            } else {
                runtime_err("Object is not a record".to_string())
            }
        })
        .collect::<Map<String, Value>>()
}

fn serialise_data(node: &Box<Node>) -> Value {
    match node.deref() {
        Node::String { val, .. } => Value::String(val.clone()),
        Node::Date { val, .. } => Value::String(val.to_string()),
        Node::Real { val, .. } => Value::Number(Number::from_f64(*val).unwrap()),
        Node::Int { val, .. } => Value::Number(Number::from(*val)),
        Node::Boolean { val, .. } => Value::Bool(*val),
        Node::Array { values, .. } => values
            .iter()
            .map(|v| serialise_data(v.borrow().deref()))
            .collect(),
        _ => unreachable!(),
    }
}

fn value_to_node(old: &Box<Node>, new: &Value, new_type: Box<VariableType>) -> Node {
    let new_type = new_type.deref().clone();
    match new {
        Value::String(val) => {
            if new_type == VariableType::String {
                Node::String {
                    val: val.clone(),
                    pos: Position::invalid(),
                }
            } else {
                Node::Date {
                    val: NaiveDate::parse_from_str(val.as_str(), "%Y-%m-%d").unwrap(),
                    pos: Position::invalid(),
                }
            }
        }
        Value::Number(val) => {
            if new_type == VariableType::Integer {
                Node::Int {
                    val: val.as_i64().unwrap(),
                    pos: Position::invalid(),
                }
            } else {
                Node::Real {
                    val: val.as_f64().unwrap(),
                    pos: Position::invalid(),
                }
            }
        }
        Value::Bool(val) => Node::Boolean {
            val: val.clone(),
            pos: Position::invalid(),
        },
        Value::Array(values) => {
            if let Node::Array { shape, .. } = old.clone().deref().deref() {
                if let VariableType::Array(arr_type) = new_type {
                    let mut len_of_narray = 0;
                    shape.iter().for_each(|index| {
                        len_of_narray += (index.upper - index.lower + 1) as usize
                    });
                    if values.len() != len_of_narray {
                        runtime_err("Array shape mismatch while getting record".to_string())
                    }
                    Node::Array {
                        values: values
                            .iter()
                            .map(|v| {
                                NodeRef::new_ref(Box::from(value_to_node(
                                    old,
                                    v,
                                    Box::from(arr_type.t.clone()),
                                )))
                            })
                            .collect(),
                        shape: shape.clone(),
                        pos: Position::invalid(),
                    }
                } else {
                    unreachable!()
                }
            } else {
                unreachable!()
            }
        }
        _ => unreachable!(),
    }
}

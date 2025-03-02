use crate::enums::{Node, Position};
use crate::executor::run_expr::run_expr;
use crate::executor::run_stmt::run_assign;
use crate::executor::runtime_err;
use crate::executor::variable::{Executor, XFile};
use crate::tokens::TToken;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

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
                                let mut file = File::open(filename.clone()).expect("");
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
                        cursor: 0,
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
    match executor.file_handles.remove(&filename) {
        None => runtime_err(format!("File {} is already closed", filename)),
        _ => {},
    }
}

pub fn run_write_file(executor: &mut Executor, filename: &Box<Node>, data: &Box<Node>) {
    let filename = run_expr(executor, filename).val_as_str();
    let data = run_expr(executor, data).val_as_str() + "\n";
    match executor.file_handles.get_mut(&filename) {
        None => runtime_err(format!("File {} is not opened", filename)),
        Some(file) => {

            if file.mode != "APPEND" && file.mode != "WRITE" {
                runtime_err(format!("File {} is not opened for APPEND or WRITE", filename))
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
            
            file.cursor += 1;
            let content = Box::from(Node::Expression(vec![
                Box::from(Node::String {
                    val: if file.cursor <= file.content.len() {
                        file.content[file.cursor - 1].clone()
                    } else {
                        String::new()
                    },
                    pos: Position::invalid(),
                })
            ]));
            run_assign(executor, destination, &content);
        }
    }
}

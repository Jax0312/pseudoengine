use std::collections::HashMap;

use crate::enums::{Node, VariableType};
use crate::executor::runtime_err;

pub struct Executor {
    pub scopes: Vec<Scope>,
    pub functions: HashMap<String, Function>,
}

pub struct State {
    pub variables: HashMap<String, Variable>,
}

impl State {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

pub enum Scope {
    Global(State),
    Local(State),
}

#[derive(Debug)]
pub struct Variable {
    pub value: Box<Node>,
    pub t: VariableType,
}

#[derive(Debug)]
pub struct Function {
    pub value: Box<Node>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            scopes: vec![Scope::Global(State::new())],
            functions: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::Local(State::new()))
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare_fn(&mut self, identifier: &String, node: &Box<Node>) {
        if !self.functions.contains_key(identifier) {
            self.functions.insert(
                identifier.clone(),
                Function {
                    value: node.clone(),
                },
            );
            return;
        }
        runtime_err(format!("{} is already declared", identifier))
    }

    pub fn get_fn(&mut self, identifier: &String) -> Box<Node> {
        if self.functions.contains_key(identifier) {
            return self.functions.get(identifier).unwrap().value.clone();
        }
        runtime_err(format!("{} is not declared", identifier))
    }

    pub fn declare_var(&mut self, identifier: &String, value: Box<Node>, t: &Box<VariableType>) {
        let scope = self.scopes.last_mut().unwrap();
        match scope {
            Scope::Global(ref mut state) | Scope::Local(ref mut state) => {
                if !state.variables.contains_key(identifier) {
                    state.variables.insert(
                        identifier.clone(),
                        Variable {
                            value,
                            t: *t.clone(),
                        },
                    );
                } else {
                    runtime_err(format!("{} is already initialized", identifier))
                }
            }
        }
    }

    pub fn set_var(&mut self, identifier: &String, value: Box<Node>) {
        for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Global(ref mut state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        return var.value = value;
                    } else {
                        break;
                    }
                }
                Scope::Local(ref mut state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        return var.value = value;
                    }
                }
            }
        }

        runtime_err(format!("{} is not declared", identifier))
    }

    pub fn get_var<'a>(&'a mut self, identifier: &String) -> &'a Variable {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Global(state) => {
                    if let Some(var) = state.variables.get(identifier) {
                        return var;
                    } else {
                        break;
                    }
                }
                Scope::Local(state) => {
                    if let Some(var) = state.variables.get(identifier) {
                        return var;
                    }
                }
            }
        }

        runtime_err(format!("{} is not declared", identifier))
    }

    pub fn get_var_mut<'a>(&'a mut self, identifier: &String) -> &'a mut Variable {
        for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Global(state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        return var;
                    } else {
                        break;
                    }
                }
                Scope::Local(state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        return var;
                    }
                }
            }
        }

        runtime_err(format!("{} is not declared", identifier))
    }
}

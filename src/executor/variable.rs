use std::collections::HashMap;
use std::ops::Deref;

use crate::enums::{Node, VariableType};
use crate::executor::runtime_err;

pub struct Executor {
    pub scopes: Vec<Scope>,
    pub defs: HashMap<String, Definition>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Scope {
    Global(State),
    Local(State),
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub value: Box<Node>,
    pub t: VariableType,
}

#[derive(Debug, Clone)]
pub enum Property {
    Var {
        value: Box<Node>,
        t: Box<VariableType>,
        private: bool,
    },
    Procedure {
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
        private: bool,
    },
}

#[derive(Debug, Clone)]
pub enum Definition {
    Function {
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
    },
    Class {
        name: String,
        props: HashMap<String, Property>,
    },
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            scopes: vec![Scope::Global(State::new())],
            defs: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::Local(State::new()))
    }

    pub fn exit_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }

    pub fn declare_def(&mut self, identifier: &String, def: Definition) {
        if !self.defs.contains_key(identifier) {
            self.defs.insert(identifier.clone(), def);
            return;
        }
        runtime_err(format!("{} is already declared", identifier))
    }

    pub fn get_def(&mut self, identifier: &String) -> Definition {
        if self.defs.contains_key(identifier) {
            return self.defs.get(identifier).unwrap().clone();
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

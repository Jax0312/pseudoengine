use std::collections::HashMap;
use std::ops::Deref;

use crate::enums::{Node, VariableType};
use crate::executor::{runtime_err, var_type_of};

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
    pub mutable: bool,
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
    Record {
        name: String,
        props: HashMap<String, Property>,
    },
    Enum {
        name: String,
    },
    Ref {
        name: String,
        ref_to: Box<VariableType>,
    }
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

    pub fn declare_var(&mut self, identifier: &String, value: Box<Node>, t: &Box<VariableType>, mutable: bool) {
        let scope = self.scopes.last_mut().unwrap();
        match scope {
            Scope::Global(ref mut state) | Scope::Local(ref mut state) => {
                if !state.variables.contains_key(identifier) {
                    state.variables.insert(
                        identifier.clone(),
                        Variable {
                            value,
                            t: *t.clone(),
                            mutable,
                        },
                    );
                } else {
                    runtime_err(format!("{} is already initialized", identifier))
                }
            }
        }
    }


    // Assign value to variable with type checking
    pub fn set_var(&mut self, identifier: &String, value: Box<Node>) {

        let rhs_type = match value.deref() {
            Node::Reference(val) => {
                if let Node::Var {name, pos} = val.deref() {
                    VariableType::Pointer(Box::from(self.get_var(&name).t.clone()))
                } else {
                    unreachable!()
                }

            },
            _ => var_type_of(&value),
        };


        let mut var = None;
        for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Global(ref mut state) => {
                    if let Some(_var) = state.variables.get_mut(identifier) {
                        var = Some(_var);
                    } else {
                        break;
                    }
                },
                Scope::Local(ref mut state) => {
                    if let Some(_var) = state.variables.get_mut(identifier) {
                        var = Some(_var);
                    }
                }
            }
        }

        let var = match var {
            Some(var) => var,
            None => runtime_err(format!("{} is not declared", identifier))
        };

        // assigning logic
        if var.mutable {
            println!("{:?}", value);
            let lhs_type = match &var.t {
                VariableType::Custom(udt) => match get_def(&mut self.defs, udt) {
                    Definition::Ref {name, ref_to} => {Some(VariableType::Pointer(ref_to))}
                    _ => None,
                }
                _ => None
            };
            if lhs_type.unwrap_or(var.t.clone()) == rhs_type {
                var.value = value;
            } else {
                runtime_err(format!("Cannot assign {:?} to {:?}", rhs_type, var.t))
            }

        } else {
            runtime_err(format!("{} is a constant, it's value cannot be modified", identifier))
        }

    }

    pub fn get_var(&self, identifier: &String) -> &Variable {
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
                        if var.mutable {
                            return var;
                        } else {
                            runtime_err(format!("{} is a constant, it's value cannot be modified", identifier))
                        }
                    } else {
                        break;
                    }
                }
                Scope::Local(state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        if var.mutable {
                            return var;
                        } else {
                            runtime_err(format!("{} is a constant, it's value cannot be modified", identifier))
                        }
                    }
                }
            }
        }

        runtime_err(format!("{} is not declared", identifier))
    }
}

pub fn declare_def(defs: &mut HashMap<String, Definition>, identifier: &String, def: Definition) {
    if !defs.contains_key(identifier) {
        defs.insert(identifier.clone(), def);
        return;
    }
    runtime_err(format!("{} is already declared", identifier))
}

pub fn get_def(defs: &mut HashMap<String, Definition>, identifier: &String) -> Definition {
    if defs.contains_key(identifier) {
        return defs.get(identifier).unwrap().clone();
    }
    runtime_err(format!("{} is not declared", identifier))
}


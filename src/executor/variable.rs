use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

use crate::{
    enums::{Node, NodeRef, Position, VariableType},
    utils::err,
};

pub struct Executor {
    pub scopes: Vec<Scope>,
    pub file_handles: HashMap<String, XFile>,
}

pub struct XFile {
    pub handle: File,
    pub mode: String,
    pub content: Vec<String>,
    pub cursor: usize,
}

#[derive(Debug)]
pub struct State {
    pub variables: HashMap<String, Variable>,
    pub defs: HashMap<String, Definition>,
}

impl State {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            defs: HashMap::new(),
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
    pub value: NodeRef,
    pub t: VariableType,
    pub mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Property {
    Var {
        value: NodeRef,
        t: Box<VariableType>,
        private: bool,
    },
    Method {
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
        private: bool,
        returns: bool,
    },
}

#[derive(Debug, Clone)]
pub enum Definition {
    Function {
        params: Vec<Box<Node>>,
        children: Vec<Box<Node>>,
        returns: bool,
    },
    Class {
        name: String,
        base: Box<Definition>,
        props: HashMap<String, Property>,
    },
    Record {
        name: String,
        props: HashMap<String, Property>,
    },
    Enum {
        name: String,
    },
    Pointer {
        name: String,
        ref_to: Box<VariableType>,
    },
    Null,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            scopes: vec![Scope::Global(State::new())],
            file_handles: HashMap::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::Local(State::new()))
    }

    pub fn exit_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }

    pub fn declare_var(
        &mut self,
        identifier: &String,
        value: Box<Node>,
        t: &Box<VariableType>,
        mutable: bool,
        pos: &Position,
    ) {
        let scope = self.scopes.last_mut().unwrap();
        match scope {
            Scope::Global(ref mut state) | Scope::Local(ref mut state) => {
                if !state.variables.contains_key(identifier) {
                    state.variables.insert(
                        identifier.clone(),
                        Variable {
                            value: Rc::new(RefCell::new(value)),
                            t: *t.clone(),
                            mutable,
                        },
                    );
                } else {
                    err(
                        format!("'{}' is already initialized", identifier).as_str(),
                        pos,
                    )
                }
            }
        }
    }

    pub fn var_exist(&self, identifier: &String) -> bool {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Global(state) | Scope::Local(state) => {
                    if let Some(_) = state.variables.get(identifier) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn get_var(&self, identifier: &String, pos: &Position) -> &Variable {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Global(state) | Scope::Local(state) => {
                    if let Some(var) = state.variables.get(identifier) {
                        return var;
                    }
                }
            }
        }

        err(format!("'{}' is not declared", identifier).as_str(), pos)
    }

    pub fn get_var_mut(&mut self, identifier: &String, pos: &Position) -> &mut Variable {
        for scope in self.scopes.iter_mut().rev() {
            match scope {
                Scope::Global(state) | Scope::Local(state) => {
                    if let Some(var) = state.variables.get_mut(identifier) {
                        if var.mutable {
                            return var;
                        } else {
                            err(
                                format!(
                                    "'{}' is a constant, it's value cannot be modified",
                                    identifier
                                )
                                .as_str(),
                                pos,
                            )
                        }
                    }
                }
            }
        }

        err(format!("'{}' is not declared", identifier).as_str(), pos)
    }

    pub fn declare_def(&mut self, identifier: &String, def: Definition, pos: &Position) {
        let scope = self.scopes.last_mut().unwrap();
        match scope {
            Scope::Global(ref mut state) | Scope::Local(ref mut state) => {
                if !state.defs.contains_key(identifier) {
                    state.defs.insert(identifier.clone(), def.clone());
                } else {
                    err(
                        format!("'{}' is already declared", identifier).as_str(),
                        pos,
                    )
                }
            }
        }
    }

    pub fn get_def(&mut self, identifier: &String, pos: &Position) -> Definition {
        for scope in self.scopes.iter().rev() {
            match scope {
                Scope::Global(state) | Scope::Local(state) => {
                    if let Some(def) = state.defs.get(identifier) {
                        return def.clone();
                    }
                }
            }
        }
        err(format!("'{}' is not declared", identifier).as_str(), pos)
    }
}

pub trait NodeDeref {
    fn new_ref(node: Box<Node>) -> NodeRef;
    fn clone_node(&self) -> Box<Node>;
    fn clone_ref(&self) -> NodeRef;
}

impl NodeDeref for NodeRef {
    fn new_ref(node: Box<Node>) -> NodeRef {
        Rc::new(RefCell::new(node))
    }

    fn clone_node(&self) -> Box<Node> {
        self.borrow().clone()
    }

    fn clone_ref(&self) -> NodeRef {
        Rc::new(RefCell::new(self.borrow().clone()))
    }
}

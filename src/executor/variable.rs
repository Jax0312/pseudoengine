use crate::enums::VariableType;
use crate::executor::{runtime_err, Scope, Variable};

pub fn run_declare(scopes: &mut Vec<Scope>, identifiers: &[String], t: &Box<VariableType>) {
    let scope = scopes.last_mut().unwrap();
    for identifier in identifiers {
        match scope {
            Scope::Global(ref mut state) | Scope::Local(ref mut state) => {
                if !state.variables.contains_key(identifier) {
                    state.variables.insert(identifier.clone(), Variable { value: Box::new(()), t: *t.clone() });
                } else {
                    runtime_err(format!("{} is already initialized", identifier))
                }
            }
        }
    }
}

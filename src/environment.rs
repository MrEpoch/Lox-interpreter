use std::{cell::RefCell, collections::HashMap, process::exit, rc::Rc};

use crate::{interpreter::Global, Expr};

#[derive(Clone, Debug, PartialEq)]
pub enum EnvironmentValue {
    Expr(Expr),
    Global(Global),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Environment {
    pub map: RefCell<HashMap<String, EnvironmentValue>>,
    pub enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            map: RefCell::new(HashMap::new()),
        }
    }

    pub fn migrate_environment(
        &mut self,
        map: RefCell<HashMap<String, EnvironmentValue>>,
        enclosing: Option<Rc<Environment>>,
    ) {
        self.map = map;
        if let Some(enclosing) = enclosing {
            self.enclosing = Some(enclosing);
        } else {
            self.enclosing = None;
        }
    }

    pub fn assign(&self, name: &str, value: EnvironmentValue) {
        if self.check_definition(name) {
            self.map.borrow_mut().remove(name);
            self.map.borrow_mut().insert(name.to_string(), value);
            return;
        }

        if let Some(ref enclosing) = self.enclosing {
            enclosing.assign(name, value);
            return;
        }

        self.environment_error(&format!("Undefined variable '{}'", name));
    }

    pub fn define(&mut self, name: &str, value: EnvironmentValue) {
        if self.map.borrow().contains_key(name) {
            self.map.borrow_mut().remove(name);
            self.map.borrow_mut().insert(name.to_string(), value);
        } else {
            self.map.borrow_mut().insert(name.to_string(), value);
        }
    }

    pub fn check_definition(&self, name: &str) -> bool {
        self.map.borrow().contains_key(name)
    }

    pub fn get(&self, name: &str, line: u32) -> Option<EnvironmentValue> {
        if self.check_definition(name) {
            if let Some(val) = self.map.borrow().get(name) {
                return Some(val.clone());
            }
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name, line);
        }
        // println!("Undefined variable '{name}'");
        // println!("[line {line}]");
        self.environment_error(&format!("[line {}] Undefined variable '{}'", line, name))
    }

    fn environment_error(&self, message: &str) -> Option<EnvironmentValue> {
        // println!("{}", message);
        exit(70);
    }
}

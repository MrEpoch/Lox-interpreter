use std::{collections::HashMap, process::exit};

use crate::{
    interpreter::{self, Clock, Global, LoxCallable},
    Expr,
};

#[derive(Clone, Debug)]
pub enum EnvironmentValue {
    Expr(Expr),
    Global(Global),
}

#[derive(Clone, Debug)]
pub struct Environment {
    pub map: HashMap<String, EnvironmentValue>,
    pub enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            map: HashMap::new(),
        }
    }

    pub fn migrate_environment(
        &mut self,
        map: HashMap<String, EnvironmentValue>,
        enclosing: Option<Box<Environment>>,
    ) {
        self.map = map;
        self.enclosing = enclosing;
    }

    pub fn assign(&mut self, name: &str, value: EnvironmentValue) {
        if self.check_definition(name) {
            self.map.remove(name);
            self.map.insert(name.to_string(), value);
            return;
        }

        if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value);
            return;
        }

        exit(70);
    }

    pub fn define(&mut self, name: &str, value: EnvironmentValue) {
        if self.map.contains_key(name) {
            self.map.remove(name);
            self.map.insert(name.to_string(), value);
        } else {
            self.map.insert(name.to_string(), value);
        }
    }

    pub fn check_definition(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    pub fn get(&self, name: &str, line: u32) -> Option<&EnvironmentValue> {
        if self.check_definition(name) {
            return self.map.get(name);
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.get(name, line);
        }
        // println!("Undefined variable '{name}'");
        // println!("[line {line}]");
        exit(70);
    }
}

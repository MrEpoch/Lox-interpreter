use std::collections::HashMap;

use crate::Expr;


pub struct Environment {
    map: HashMap<String, Expr>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Expr) {
        self.map.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str, line: u32) -> Option<&Expr> {
        if self.map.contains_key(name) {
            self.map.get(name)
        } else {
            println!("Undefined variable '{name}'");
            println!("[line {line}]");
            None
        }
    }
}

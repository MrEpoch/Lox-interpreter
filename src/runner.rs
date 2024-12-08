use std::process::exit;

use crate::{Expr, Literal, TokenType};

pub struct Runner {
    pub statements: Vec<Expr>,
}

impl Runner {
    pub fn new(statements: Vec<Expr>) -> Self {
        Self { statements }
    }

    pub fn interpret(&self) {
        for statement in &self.statements {
            match statement {
                Expr::String(s) => {
                    println!("{}", s);
                }
                Expr::Number(n) => {
                    println!("{}", n);
                }
                Expr::Bool(b) => {
                    println!("{}", b);
                }
                Expr::Nil => {
                    println!("nil");
                }
                _ => {
                    print!("Invalid expression");
                }
            }
        }
    }
}

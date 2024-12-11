
use crate::Expr;

pub struct Runner {
    pub statement: Expr,
}

impl Runner {
    pub fn new(statement: Expr) -> Self {
        Self { statement }
    }

    pub fn interpret(&self) {
        match &self.statement {
            Expr::Print(e) => {
                match &**e {
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
            _ => {}
        }
    }
}

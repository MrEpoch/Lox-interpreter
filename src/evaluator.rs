use crate::{Expr, Literal};


pub struct Evaluator {
    pub expression: Expr,
}

impl Evaluator {
    pub fn new(expression: Expr) -> Self {
        Self {
            expression
        }
    }

    pub fn evaluate(&self) -> Expr {
        match &self.expression {
            Expr::Literal(l) => {
                match l {
                    Literal::Bool(b) => Expr::Bool(*b),
                    Literal::String(s) => {
                        Expr::String(s.clone())
                    },
                    Literal::Number(n) => Expr::Number(n.0),
                    _ => Expr::Nil,
                }
            }
            _ => Expr::Nil,
        }
    }
}


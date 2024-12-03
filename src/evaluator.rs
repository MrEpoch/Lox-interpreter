use crate::{Expr, Literal, TokenType};


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
        self.evaluator(&self.expression)
    }

    fn evaluator(&self, expr: &Expr) -> Expr {
        match expr {
            Expr::Literal(l) => {
                match l {
                    Literal::Bool(b) => Expr::Bool(*b),
                    Literal::String(s) => {
                        Expr::String(s.clone())
                    },
                    Literal::Number(n) => Expr::Number(n.0),
                    _ => Expr::Nil,
                }
            },
            Expr::Binary { operator, left, right } => {
                let left = self.evaluator(left);
                let right = self.evaluator(right);

                match operator.token_type {
                    TokenType::MINUS => {
                        match (left, right) {
                            // Here i convert the left and right values to Expr::Number and use
                            // them
                            (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 - n2),
                            _ => Expr::Nil,
                        }
                    },
                    TokenType::SLASH => {
                        match (left, right) {
                            (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 / n2),
                            _ => Expr::Nil,
                        }
                    },
                    TokenType::STAR => {
                        match (left, right) {
                            (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 * n2),
                            _ => Expr::Nil,
                        }
                    },
                    TokenType::PLUS => {
                        match (left, right) {
                            (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 + n2),
                            (Expr::String(s1), Expr::String(s2)) => Expr::String(format!("{}{}", s1, s2)),
                            _ => Expr::Nil,
                        }
                    }
                    _ => Expr::Nil,
                }
            }
            Expr::Unary { operator, right } => {
                let evaluated = self.evaluator(right);
                match operator.token_type {
                    TokenType::BANG => {
                        match evaluated {
                            Expr::Bool(b) => Expr::Bool(!b),
                            Expr::Unary { operator:_, right:_ } => {
                                self.evaluator(&evaluated)
                            },
                            Expr::Nil => Expr::Bool(true),
                            _ => Expr::Nil,
                        }
                    },
                    TokenType::MINUS => {
                        match evaluated {
                            Expr::Number(n) => Expr::Number(-n),
                            _ => Expr::Nil,
                        }
                    },
                    _ => Expr::Nil,
                }
            }
            Expr::Grouping(exprs) => {
                self.evaluator(&exprs[0])
            },
            _ => Expr::Nil,
        }
    }
}


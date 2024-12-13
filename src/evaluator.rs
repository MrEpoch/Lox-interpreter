use std::process::exit;

use crate::{environment, Expr, Literal, Token, TokenType};

pub struct Evaluator {
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
        }
    }

    pub fn evaluate(&self, statement: &Expr, enviroment: &mut environment::Environment) -> Expr {
        self.evaluator(&statement, enviroment)
    }

    fn invalid_error(&self) -> Expr {
        exit(70)
    }

    fn evaluator(&self, expr: &Expr, enviroment: &mut environment::Environment) -> Expr {
        match expr {
            Expr::Literal(l) => match l {
                Literal::Bool(b) => Expr::Bool(*b),
                Literal::String(s) => {
                    Expr::String(s.clone())
                }
                Literal::Number(n) => Expr::Number(n.0),
                _ => Expr::Nil,
            },
            Expr::Print(e) => {
                Expr::Print(Box::new(self.evaluator(e, enviroment)))
            }
            Expr::Logical(left, right, operator) => {
                let left = self.evaluator(left, enviroment);

                match operator {
                    TokenType::OR => {
                        if self.is_truthy(&left) {
                            left
                        } else {
                            self.evaluator(right, enviroment)
                        }
                    }
                    TokenType::AND => {
                        if !self.is_truthy(&left) {
                            left
                        } else {
                            self.evaluator(right, enviroment)
                        }
                    }
                    _ => self.invalid_error(),
                }
            }
            Expr::Var(t) => {
                let val = enviroment.get(&t.lexeme, t.line).unwrap().clone();
                // self.evaluator(&val, enviroment)
                match &val {
                    Expr::Literal(t) => {
                        self.evaluator(&val, enviroment)
                    }
                    _ => {
                        val
                    }
                }
            },
            Expr::Assign { name, value } => {
                let value = self.evaluator(value, enviroment);
                enviroment.assign(name, value.clone());
                value
            }
            Expr::Block(vec) => {
                let mut environment_block = environment::Environment::new();
                environment_block.enclosing = Some(Box::new(enviroment.clone()));
                let mut returning_vec = vec![];

                for expr in vec {
                    returning_vec.push(self.evaluator(expr, &mut environment_block));
                }
                enviroment.map = environment_block.enclosing.unwrap().map;
                Expr::Block(returning_vec)
            }
            Expr::If { condition, then_branch, else_branch } => {
                if self.is_truthy(&self.evaluator(condition, enviroment)) {
                    self.evaluator(then_branch, enviroment)
                } else if let Some(else_branch) = else_branch {
                    self.evaluator(else_branch, enviroment)
                } else {
                    Expr::Nil
                }
            }
            Expr::Variable { name, value } => {
                let value_def = self.evaluator(value, enviroment);
                enviroment.define(name, value_def.clone());
                Expr::Variable { name: name.clone(), value: Box::new(value_def) }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.evaluator(left, enviroment);
                let right = self.evaluator(right, enviroment);

                match operator.token_type {
                    TokenType::MINUS => {
                        match (left, right) {
                            // Here i convert the left and right values to Expr::Number and use
                            // them
                            (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 - n2),
                            _ => self.invalid_error(),
                        }
                    }
                    TokenType::SLASH => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 / n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::STAR => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 * n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::PLUS => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 + n2),
                        (Expr::String(s1), Expr::String(s2)) => {
                            Expr::String(format!("{}{}", s1, s2))
                        }
                        _ => { 
                            self.invalid_error()
                        }
                    },
                    TokenType::GREATER => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 > n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::GREATER_EQUAL => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 >= n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::LESS => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 < n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::LESS_EQUAL => match (left, right) {
                        (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 <= n2),
                        _ => self.invalid_error(),
                    },
                    TokenType::EQUAL_EQUAL => {
                        if self.is_equal(left, right) {
                            Expr::Bool(true)
                        } else {
                            Expr::Bool(false)
                        }
                    }
                    TokenType::BANG_EQUAL => {
                        if self.is_equal(left, right) {
                            Expr::Bool(false)
                        } else {
                            Expr::Bool(true)
                        }
                    }
                    _ => Expr::Nil,
                }
            }
            Expr::Unary { operator, right } => {
                let evaluated = self.evaluator(right, enviroment);
                match operator.token_type {
                    TokenType::BANG => match evaluated {
                        Expr::Bool(b) => Expr::Bool(!b),
                        Expr::Unary {
                            operator: _,
                            right: _,
                        } => self.evaluator(&evaluated, enviroment),
                        Expr::Nil => Expr::Bool(true),
                        _ => Expr::Nil,
                    },
                    TokenType::MINUS => {
                        match evaluated {
                            Expr::Number(n) => Expr::Number(-n),
                            _ => {
                                /*
                                println!("Operand must be a number.");
                                println!("[line {}]", operator.line);
                                */
                                exit(70);
                            }
                        }
                    }
                    _ => Expr::Nil,
                }
            }
            Expr::Grouping(exprs) => self.evaluator(&exprs[0], enviroment),
            _ => Expr::Nil,
        }
    }

    fn is_truthy(&self, expr: &Expr) -> bool {
        match expr {
            &Expr::Nil => false,
            &Expr::Bool(b) => b,
            _ => true,
        }
    }

    fn is_equal(&self, left: Expr, right: Expr) -> bool {
        if left == right {
            return true;
        }
        return false;
    }
}

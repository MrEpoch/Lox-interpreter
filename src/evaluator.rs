use std::process::exit;

use crate::interpreter::LoxCallable;
use crate::{
    environment::{self, EnvironmentValue},
    interpreter::{CallReturn, EvaluatorReturn, Global},
    runner::{self},
    Expr, Literal, TokenType,
};

pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate(
        &self,
        statement: &Expr,
        environment: &mut environment::Environment,
        statements: &Vec<Expr>,
    ) -> EvaluatorReturn {
        self.evaluator(&statement, environment, statements)
    }

    fn invalid_error(&self, message: String) -> Expr {
        // println!("{}", message);
        exit(70)
    }

    fn evaluator(
        &self,
        expr: &Expr,
        environment: &mut environment::Environment,
        statements: &Vec<Expr>,
    ) -> EvaluatorReturn {
        match expr {
            Expr::Var(t) => {
                let val = environment.get(&t.lexeme, t.line).unwrap().clone();
                // self.evaluator(&val, environment)
                match val {
                    EnvironmentValue::Expr(e) => match &e {
                        Expr::Literal(t) => {
                            EvaluatorReturn::Expr(self.expr_match(&e, environment, statements))
                        }
                        Expr::Function {
                            name, params, body, ..
                        } => EvaluatorReturn::Expr(Expr::Function {
                            name: name.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            function_type: String::from("local"),
                        }),
                        _ => EvaluatorReturn::Expr(e),
                    },
                    EnvironmentValue::Global(g) => EvaluatorReturn::Global(g.clone()),
                }
            }
            _ => EvaluatorReturn::Expr(self.expr_match(expr, environment, statements)),
        }
    }

    fn is_truthy(&self, expr: &Expr) -> bool {
        match expr {
            &Expr::Nil => false,
            &Expr::Bool(b) => b,
            _ => true,
        }
    }

    fn expr_match(
        &self,
        expr: &Expr,
        environment: &mut environment::Environment,
        statements: &Vec<Expr>,
    ) -> Expr {
        match expr {
            Expr::Literal(l) => match l {
                Literal::Bool(b) => Expr::Bool(*b),
                Literal::String(s) => Expr::String(s.clone()),
                Literal::Number(n) => Expr::Number(n.0),
                _ => Expr::Nil,
            },
            Expr::Print(e) => {
                if let EvaluatorReturn::Expr(v) = self.evaluate(e, environment, statements) {
                    return Expr::Print(Box::new(v));
                } else {
                    return Expr::Nil;
                }
            }
            Expr::Logical(left, right, operator) => {
                let left = self.expr_match(left, environment, statements);

                match operator {
                    TokenType::OR => {
                        if self.is_truthy(&left) {
                            left
                        } else {
                            self.expr_match(right, environment, statements)
                        }
                    }
                    TokenType::AND => {
                        if !self.is_truthy(&left) {
                            left
                        } else {
                            self.expr_match(right, environment, statements)
                        }
                    }
                    _ => self.invalid_error(String::from("Logical error")),
                }
            }
            Expr::Assign { name, value } => {
                let value_e = self.evaluate(value, environment, statements);
                if let EvaluatorReturn::Expr(e) = value_e {
                    environment.assign(name, EnvironmentValue::Expr(e.clone()));
                    e
                } else {
                    self.invalid_error(String::from("Assign error"))
                }
            }
            Expr::Block(vec) => {
                let mut environment_clone = environment::Environment::new();
                let mut evaluated: Expr;

                environment_clone.enclosing = Some(Box::new(environment.clone()));

                for expr in vec {
                    evaluated = self.expr_match(expr, &mut environment_clone, statements);
                    runner::interpret(evaluated);
                }

                let prev_env = environment_clone.enclosing.unwrap();
                environment.migrate_environment(prev_env.map, prev_env.enclosing);

                Expr::Nil
            }
            Expr::Increment(i) => match self.evaluate(i, environment, statements) {
                EvaluatorReturn::Expr(e) => e,
                _ => self.invalid_error(String::from("Increment error")),
            },
            Expr::While(condition, body) => {
                let mut evaluated: Expr;

                let eval_condition = self.evaluate(condition, environment, statements);
                if let EvaluatorReturn::Expr(mut e) = eval_condition {
                    while self.is_truthy(&e) {
                        evaluated = self.expr_match(body, environment, statements);
                        runner::interpret(evaluated);

                        e = if let EvaluatorReturn::Expr(e) =
                            self.evaluate(condition, environment, statements)
                        {
                            e
                        } else {
                            Expr::Nil
                        }
                    }
                }

                Expr::Nil
            }
            Expr::Function {
                name, params, body, ..
            } => {
                environment.define(
                    &name.lexeme,
                    EnvironmentValue::Expr(Expr::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        function_type: String::from("local"),
                    }),
                );
                Expr::String(format!("<fn {}>", name.lexeme))
            }
            Expr::Call(callee, _, args) => {
                let callee_ev = self.evaluate(callee, environment, statements);

                let mut arguments = vec![];

                for argument in args {
                    arguments.push(self.expr_match(argument, environment, statements));
                }

                match &callee_ev {
                    EvaluatorReturn::Expr(e) => match e {
                        Expr::Function { .. } => {
                            if !e.is_lox_callable(&callee) {
                                self.invalid_error(String::from(
                                    "Can only call functions and classes.",
                                ));
                            }

                            if arguments.len() != e.arity() {
                                self.invalid_error(format!(
                                    "Expected {} arguments but got {}.",
                                    e.arity(),
                                    arguments.len()
                                ));
                            }

                            match e.call(environment, statements, arguments) {
                                CallReturn::Expr(e) => e,
                            }
                        }
                        _ => return Expr::Nil,
                    },
                    EvaluatorReturn::Global(g) => match g {
                        Global::Clock(c) => {
                            if !c.is_lox_callable(&callee) {
                                self.invalid_error(String::from(
                                    "Can only call functions and classes.",
                                ));
                            }

                            if arguments.len() != c.arity() {
                                self.invalid_error(format!(
                                    "Expected {} arguments but got {}.",
                                    c.arity(),
                                    arguments.len()
                                ));
                            }

                            match c.call(environment, statements, arguments) {
                                CallReturn::Expr(e) => e,
                            }
                        }
                    },
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if let EvaluatorReturn::Expr(e) = self.evaluate(condition, environment, statements)
                {
                    if self.is_truthy(&e) {
                        self.expr_match(then_branch, environment, statements)
                    } else if let Some(else_branch) = else_branch {
                        self.expr_match(else_branch, environment, statements)
                    } else {
                        Expr::Nil
                    }
                } else {
                    self.invalid_error(String::from("If condition error"))
                }
            }
            Expr::Variable { name, value } => {
                let value_def = self.evaluate(value, environment, statements);
                if let EvaluatorReturn::Expr(e) = value_def {
                    environment.define(name, EnvironmentValue::Expr(e.clone()));
                    Expr::Variable {
                        name: name.clone(),
                        value: Box::new(e),
                    }
                } else {
                    self.invalid_error(String::from("Variable error"))
                }
            }
            Expr::Binary {
                operator,
                left,
                right,
            } => {
                let left = self.evaluate(left, environment, statements);
                let right = self.evaluate(right, environment, statements);

                match (left, right) {
                    (EvaluatorReturn::Expr(left), EvaluatorReturn::Expr(right)) => {
                        match operator.token_type {
                            TokenType::MINUS => {
                                match (left, right) {
                                    // Here i convert the left and right values to Expr::Number and use
                                    // them
                                    (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 - n2),
                                    _ => self.invalid_error(String::from("Binary minus error")),
                                }
                            }
                            TokenType::SLASH => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 / n2),
                                _ => self.invalid_error(String::from("Binary slash error")),
                            },
                            TokenType::STAR => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 * n2),
                                _ => self.invalid_error(String::from("Binary star error")),
                            },
                            TokenType::PLUS => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Number(n1 + n2),
                                (Expr::String(s1), Expr::String(s2)) => {
                                    Expr::String(format!("{}{}", s1, s2))
                                }
                                _ => self.invalid_error(String::from("Binary plus error")),
                            },
                            TokenType::GREATER => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 > n2),
                                _ => self.invalid_error(String::from("Binary greater error")),
                            },
                            TokenType::GREATER_EQUAL => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 >= n2),
                                _ => self.invalid_error(String::from("Binary greater equal error")),
                            },
                            TokenType::LESS => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 < n2),
                                _ => self.invalid_error(String::from("Binary less error")),
                            },
                            TokenType::LESS_EQUAL => match (left, right) {
                                (Expr::Number(n1), Expr::Number(n2)) => Expr::Bool(n1 <= n2),
                                _ => self.invalid_error(String::from("Binary less equal error")),
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
                    _ => Expr::Nil,
                }
            }
            Expr::Unary { operator, right } => {
                let evaluated = self.evaluate(right, environment, statements);
                if let EvaluatorReturn::Expr(e) = evaluated {
                    match operator.token_type {
                        TokenType::BANG => match e {
                            Expr::Bool(b) => Expr::Bool(!b),
                            Expr::Unary {
                                operator: _,
                                right: _,
                            } => {
                                if let EvaluatorReturn::Expr(e_u) =
                                    self.evaluate(right, environment, statements)
                                {
                                    e_u
                                } else {
                                    Expr::Nil
                                }
                            }
                            Expr::Nil => Expr::Bool(true),
                            _ => Expr::Nil,
                        },
                        TokenType::MINUS => {
                            match e {
                                Expr::Number(n) => Expr::Number(-n),
                                _ => {
                                    /*
                                    println!("Operand must be a number.");
                                    println!("[line {}]", operator.line);
                                    */
                                    self.invalid_error(String::from("Unary minus error"))
                                }
                            }
                        }
                        _ => Expr::Nil,
                    }
                } else {
                    Expr::Nil
                }
            }
            Expr::Grouping(exprs) => self.expr_match(&exprs[0], environment, statements),
            _ => Expr::Nil,
        }
    }

    fn is_equal(&self, left: Expr, right: Expr) -> bool {
        if left == right {
            return true;
        }
        return false;
    }
}

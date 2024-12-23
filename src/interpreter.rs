use core::fmt;
use std::fmt::Debug;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{collections::HashMap, fs, io, process::exit, sync::Mutex};

use once_cell::sync::Lazy;

use crate::environment::EnvironmentValue;
use crate::formatters::{get_from_unary, handle_grouping, handle_match, print_based_on_literal};
use crate::{environment, evaluator, parser, runner, scanner};

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number((f64, usize)),
    Bool(bool),
    Null,
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TokenType {
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,

    IDENTIFIER,
    STRING,
    NUMBER,

    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Bool(bool),
    Logical(Box<Expr>, Box<Expr>, TokenType),
    Literal(Literal),
    Print(Box<Expr>),
    Return(Token, Box<Expr>),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Expr>,
        environment: Option<environment::Environment>,
    },
    Variable {
        name: String,
        value: Box<Expr>,
    },
    Block(Vec<Expr>),
    While(Box<Expr>, Box<Expr>),
    Var(Token),
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Increment(Box<Expr>),
    Number(f64),
    Nil,
    String(String),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        operator: Token,
        right: Box<Expr>,
        left: Box<Expr>,
    },
    Grouping(Vec<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
}

impl<'a> fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Return(keyword, value) => f.write_fmt(format_args!("{keyword} {value}")),
            Expr::Function {
                name,
                params,
                body,
                environment
            } => f.write_fmt(format_args!(
                "{name} {:?} {:?}",
                params, body
            )),
            Expr::Call(a, b, c) => f.write_fmt(format_args!("{a} {b} {:?}", c)),
            Expr::Increment(a) => f.write_fmt(format_args!("{a}")),
            Expr::While(a, b) => f.write_fmt(format_args!("{a} {b}")),
            Expr::Logical(a, b, c) => f.write_fmt(format_args!("{a} {b} {c}")),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => f.write_fmt(format_args!(
                "if {} {} {}",
                *condition,
                *then_branch,
                else_branch.as_ref().unwrap()
            )),
            Expr::Block(vec_expr) => {
                for expr in vec_expr {
                    f.write_fmt(format_args!("{expr}"))?;
                }
                Ok(())
            }
            Expr::Assign { name, value } => f.write_fmt(format_args!("{name} = {value}")),
            Expr::Var(expr) => f.write_fmt(format_args!("{expr}")),
            Expr::Variable { name, value } => f.write_fmt(format_args!("{name} = {value}")),
            Expr::Print(expr) => f.write_fmt(format_args!("{expr}")),
            Expr::Bool(b) => f.write_fmt(format_args!("{}", b)),
            Expr::Nil => f.write_str("nil"),
            Expr::String(s) => f.write_fmt(format_args!("{s:?}")),
            Expr::Number(n) => f.write_fmt(format_args!("{n:?}")),
            Expr::Literal(l) => f.write_fmt(format_args!("{l:?}")),
            Expr::Unary { operator, right } => {
                f.write_fmt(format_args!("{} {right}", operator.lexeme))
            }
            Expr::Binary {
                operator,
                right,
                left,
            } => f.write_fmt(format_args!("({} {left} {right}", operator.lexeme)),
            Expr::Grouping(_) => f.write_str("()"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum EvaluatorReturn {
    Expr(Expr),
    Global(Global),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Global {
    Clock(Clock),
}

pub trait LoxCallable: Debug + Clone {
    fn call(
        &self,
        environment: &mut environment::Environment,
        fn_bind: Option<&Expr>,
        arguments: Vec<Expr>,
    ) -> CallReturn;
    fn arity(&self) -> usize;
}

impl Expr {
    pub fn is_lox_callable(&self, callee: &Expr) -> bool {
        match &callee {
            Expr::Var(_) => true,
            Expr::Call(..) => true,
            _ => false,
        }
    }
}

impl LoxCallable for Expr {
    fn call(
        &self,
        environment: &mut environment::Environment,
        _fn_bind: Option<&Expr>,
        arguments: Vec<Expr>,
    ) -> CallReturn {
        // Don't have declaration

        match self {
            Expr::Function { params, body, environment: env_fn, name } => {
                let mut env_f = env_fn.clone();
                env_f.as_mut().unwrap().define(
                    &name.lexeme,
                    EnvironmentValue::Expr(self.clone())
                );
                for i in 0..params.len() {
                    env_f.as_mut().unwrap().define(
                        &params[i].lexeme,
                        EnvironmentValue::Expr(arguments[i].clone()),
                    );
                }

                let evaluator = evaluator::Evaluator::new();
                let expr_block = Expr::Block(body.clone());
                let evaluated =
                    evaluator.evaluate(&expr_block, &mut env_f.clone().unwrap(), Some(&expr_block));
                if let EvaluatorReturn::Expr(e) = evaluated {
                    match e {
                        Expr::Return(_, v) => return CallReturn::Expr(*v),
                        _ => return CallReturn::Expr(Expr::Nil),
                    }
                } else {
                    return CallReturn::Expr(Expr::Nil);
                }
            }
            _ => {}
        }

        CallReturn::Expr(Expr::String(format!("<fn Nil>")))
    }

    fn arity(&self) -> usize {
        match &self {
            Expr::Function { params, .. } => params.len(),
            _ => 0,
        }
    }
}

pub enum CallReturn {
    Expr(Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Clock {}

impl LoxCallable for Clock {
    fn call(
        &self,
        _environment: &mut environment::Environment,
        _fn_bind: Option<&Expr>,
        _arguments: Vec<Expr>,
    ) -> CallReturn {
        CallReturn::Expr(Expr::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        ))
    }

    fn arity(&self) -> usize {
        0
    }
}

impl Clock {
    pub fn new() -> Self {
        Self {}
    }

    pub fn is_lox_callable(&self, _callee: &Expr) -> bool {
        true
    }

    pub fn to_string(&self) -> String {
        String::from("<native fn>")
    }
}

pub static RESERVED_KEYWORDS: Lazy<Mutex<HashMap<&'static str, TokenType>>> = Lazy::new(|| {
    let mut map = HashMap::new();

    map.insert("and", TokenType::AND);
    map.insert("class", TokenType::CLASS);
    map.insert("else", TokenType::ELSE);
    map.insert("false", TokenType::FALSE);
    map.insert("for", TokenType::FOR);
    map.insert("fun", TokenType::FUN);
    map.insert("if", TokenType::IF);
    map.insert("nil", TokenType::NIL);
    map.insert("or", TokenType::OR);
    map.insert("print", TokenType::PRINT);
    map.insert("return", TokenType::RETURN);
    map.insert("super", TokenType::SUPER);
    map.insert("this", TokenType::THIS);
    map.insert("true", TokenType::TRUE);
    map.insert("var", TokenType::VAR);
    map.insert("while", TokenType::WHILE);

    Mutex::new(map)
});

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {:?}", self.token_type, self.lexeme, self.literal)
    }
}

pub struct Interpreter {
    pub file_contents: String,
    expressions: Option<Vec<Expr>>,
}

impl Interpreter {
    pub fn new(filename: &str) -> Self {
        let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
            writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
            String::new()
        });

        if file_contents.is_empty() {
            println!("EOF  null");
        }

        Self {
            file_contents,
            expressions: None,
        }
    }

    pub fn tokenize(&mut self) {
        if !self.file_contents.is_empty() {
            let mut error_code: u8 = 0;
            let mut scanner = scanner::Scanner::new();
            scanner.scan_tokens(&self.file_contents, &mut error_code);
            for v in scanner.tokens.iter() {
                println!(
                    "{} {} {}",
                    v.token_type,
                    v.lexeme,
                    print_based_on_literal(&v.literal.as_ref().unwrap())
                );
            }

            if error_code == 65 {
                exit(65);
            }
        }
    }

    pub fn parse(&mut self) {
        if !self.file_contents.is_empty() {
            let mut scanned = scanner::Scanner::new();
            scanned.scan_tokens(&self.file_contents, &mut 0);

            let mut parser = parser::Parser::new(scanned.tokens);
            let expressions = parser.expression();

            self.expressions = Some(vec![expressions]);

            match &self.expressions.as_ref().unwrap()[0] {
                Expr::Grouping(exprs) => {
                    println!(
                        "{}",
                        handle_grouping(exprs, &String::from("(group "), &String::from(")"))
                            .join(" ")
                    );
                }
                Expr::Number(n) => {
                    println!("{n}");
                }
                Expr::Binary {
                    operator,
                    left,
                    right,
                } => {
                    println!(
                        "({} {} {})",
                        operator.lexeme,
                        handle_match(left, &String::from(""), &String::from("")),
                        handle_match(right, &String::from(""), &String::from(""))
                    );
                }
                Expr::Literal(l) => {
                    println!("{}", print_based_on_literal(&l));
                }
                Expr::Unary { .. } => {
                    println!("{}", get_from_unary(&self.expressions.as_ref().unwrap()[0]));
                }
                Expr::String(s) => {
                    println!("{}", s);
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

    pub fn evaluate(&mut self) {
        if !self.file_contents.is_empty() {
            let mut scanner = scanner::Scanner::new();
            scanner.scan_tokens(&self.file_contents, &mut 0);
            let mut parser = parser::Parser::new(scanner.tokens);
            let expression = parser.expression();
            let evaluator = evaluator::Evaluator::new();
            match evaluator.evaluate(&expression, &mut environment::Environment::new(), None) {
                EvaluatorReturn::Expr(e) => match e {
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
                },
                _ => {
                    print!("Invalid expression");
                }
            }
        }
    }

    pub fn run(&self) {
        if !self.file_contents.is_empty() {
            let mut scanner = scanner::Scanner::new();
            scanner.scan_tokens(&self.file_contents, &mut 0);
            let mut parser = parser::Parser::new(scanner.tokens);
            parser.parse();
            let evaluator = evaluator::Evaluator::new();
            let mut environment = environment::Environment::new();

            environment.define(
                "clock",
                EnvironmentValue::Global(Global::Clock(Clock::new())),
            );

            let mut index = 0;
            while index < parser.statements.len() {
                let s = &parser.statements[index];
                let evaluated = evaluator.evaluate(s, &mut environment, None);
                match evaluated {
                    EvaluatorReturn::Expr(e) => {
                        runner::interpret(e);
                    }
                    _ => {}
                }
                index += 1;
            }
        } else {
            println!("EOF  null"); // Placeholder, remove this line when implementing the Scanner
        }
    }
}

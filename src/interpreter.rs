use core::fmt;
use std::io::Write;
use std::{collections::HashMap, fs, io, process::exit, sync::Mutex};

use once_cell::sync::Lazy;

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
}

impl<'a> fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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

        Self { file_contents }
    }

    pub fn tokenize(&self) {
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

    pub fn parse(&self) {
        if !self.file_contents.is_empty() {
            let mut scanned = scanner::Scanner::new();
            scanned.scan_tokens(&self.file_contents, &mut 0);
            let mut parser = parser::Parser::new(scanned.tokens);
            let expressions = parser.expression();
            match expressions {
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
                        handle_match(*left, &String::from(""), &String::from("")),
                        handle_match(*right, &String::from(""), &String::from(""))
                    );
                }
                Expr::Literal(l) => {
                    println!("{}", print_based_on_literal(&l));
                }
                Expr::Unary { operator, right } => {
                    println!("{}", get_from_unary(Expr::Unary { operator, right }));
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

    pub fn evaluate(&self) {
        if !self.file_contents.is_empty() {
            let mut scanner = scanner::Scanner::new();
            scanner.scan_tokens(&self.file_contents, &mut 0);
            let mut parser = parser::Parser::new(scanner.tokens);
            let expression = parser.expression();
            let evaluator = evaluator::Evaluator::new();
            match evaluator.evaluate(&expression, &mut environment::Environment::new()) {
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

    pub fn run(&self) {
            if !self.file_contents.is_empty() {
                let mut scanner = scanner::Scanner::new();
                scanner.scan_tokens(&self.file_contents, &mut 0);
                let mut parser = parser::Parser::new(scanner.tokens);
                parser.parse();
                let evaluator = evaluator::Evaluator::new();
                let mut environment = environment::Environment::new();
                for s in parser.statements.iter() {
                    let evaluated = evaluator.evaluate(s, &mut environment);
                    runner::interpret(evaluated)
                }
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the Scanner
            }
    }
}

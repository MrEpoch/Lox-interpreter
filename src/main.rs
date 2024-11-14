use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use std::sync::Mutex;

mod scanner;
mod parser;

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number((f64, usize)),
    Bool(bool),
    Null,
    Nil
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

#[derive(Debug, Copy, Clone, PartialEq)]
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
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Bool(bool),
    Literal(Literal),
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
    Grouping(Vec<Expr>)
}

impl<'a> fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Bool(b) => f.write_fmt(format_args!("{}", b)),
            Expr::Nil => f.write_str("nil"),
            Expr::String(s) => f.write_fmt(format_args!("{s:?}")),
            Expr::Number(n) => f.write_fmt(format_args!("{n:?}")),
            Expr::Literal(l) => f.write_fmt(format_args!("{l:?}")),
            Expr::Unary { operator, right } => {
                f.write_fmt(format_args!("{} {right}", operator.lexeme))
            }
            Expr::Binary { operator, right, left } => {
                f.write_fmt(format_args!("({} {left} {right}", operator.lexeme))
            }
            Expr::Grouping(_) => f.write_str("()"),
        }
    }
}

static RESERVED_KEYWORDS: Lazy<Mutex<HashMap<&'static str, TokenType>>> = Lazy::new(|| {
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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            if !file_contents.is_empty() {
                let mut error_code: u8 = 0;
                let mut scanner = scanner::Scanner::new();
                scanner.scan_tokens(&file_contents, &mut error_code);

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
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the Scanner
            }
        },
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            if !file_contents.is_empty() {
                let mut scanned = scanner::Scanner::new();
                scanned.scan_tokens(&file_contents, &mut 0);
                let mut parser = parser::Parser::new(scanned.tokens);
                let expressions = parser.expression();
                match expressions {
                    Expr::Grouping(exprs) => {
                        println!("{}", handle_grouping(exprs, &String::from("(group "), &String::from(")")).join(" "));
                    }
                    Expr::Number(n) => {
                        println!("{n}");
                    }
                    Expr::Binary { operator, left, right } => {
                        println!("({} {} {})", operator.lexeme, handle_match(*left, &String::from(""), &String::from("")), handle_match(*right, &String::from(""), &String::from("")));
                    }
                    Expr::Literal(l) => {
                        println!("{}", print_based_on_literal(&l));
                    }
                    Expr::Unary { operator, right } => {
                        println!("({})", get_from_unary(Expr::Unary { operator, right }));
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
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the Scanner
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

fn print_based_on_literal(literal: &Literal) -> String {
    match literal {
        Literal::String(s) => format!("{s}"),
        Literal::Number(f) => {
            if (f.0 % 1.0).abs() < f64::EPSILON {
                f.0.to_string() + ".0"
            } else {
                f.0.to_string()
            }
        }
        Literal::Bool(b) => b.to_string(),
        Literal::Null => String::from("null"),
        Literal::Nil => String::from("nil"),
    }
}

fn handle_grouping(exprs: Vec<Expr>, left_side: &String, right_side: &String) -> Vec<String> {
    let mut r: Vec<String> = vec![];
    for e in exprs {
        r.push(handle_match(e, left_side, right_side));
    }
    r
}

fn handle_match(expr: Expr, left_side: &String, right_side: &String) -> String {
    match expr {
            Expr::Grouping(exprs) => {
                format!("{left_side}{}{right_side}", handle_grouping(exprs, &format!("(group "), &format!(")")).join(" "))
            }
            Expr::Number(n) => {
                format!("{left_side}{n}{right_side}")
            }
            Expr::Binary { operator, left, right } => {
                format!("{left_side}({} {} {}){right_side}", operator.lexeme, handle_match(*left, &String::from(""), &String::from("")), handle_match(*right, &String::from(""), &String::from("")))
            }
            Expr::Literal(l) => {
                format!("{left_side}{}{right_side}", print_based_on_literal(&l))
            }
            Expr::Unary { operator, right } => {
                format!("{left_side}({}){right_side}", get_from_unary(Expr::Unary { operator, right }))
            }
            Expr::String(s) => {
                format!("{left_side}{s}{right_side}")
            }
            Expr::Nil => {
                format!("{left_side}nil{right_side}")
            }
            _ => {
                format!("Invalid expression")
            }
        }
}

fn get_from_unary(expr_unary: Expr) -> String {
    if let Expr::Unary { operator, right } = expr_unary {
        format!("{} {}", operator.lexeme, get_from_unary(*right))
    } else if let Expr::Literal(literal) = expr_unary {
        print_based_on_literal(&literal)
    }
    else {
        expr_unary.to_string()
    }
}

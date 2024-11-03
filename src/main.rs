use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

#[derive(Debug)]
enum Literal {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Nil,
}
/*
impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
*/

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum TokenType {
    LEFT_PAREN, RIGHT_PAREN, LEFT_BRACE, RIGHT_BRACE,
    COMMA, DOT, MINUS, PLUS, SEMICOLON, SLASH, STAR,

    BANG, BANG_EQUAL,
    EQUAL, EQUAL_EQUAL,
    GREATER, GREATER_EQUAL,
    LESS, LESS_EQUAL,

    IDENTIFIER, STRING, NUMBER,

    AND, CLASS, ELSE, FALSE, FUN, FOR, IF, NIL, OR,
    PRINT, RETURN, SUPER, THIS, TRUE, VAR, WHILE,

    EOF
}
#[derive(Debug)]
struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: i32,
}

impl Token {
    fn new(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: i32) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line
        }
        }
}

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
                let tokens: Vec<Token> = scan_tokens(&file_contents, &mut error_code);

                for v in tokens.iter() {
                    println!("{} {} null", v.token_type, v.lexeme);
                }

                if error_code == 65 {
                    exit(65);
                }

            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

fn scan_tokens(source: &String, error_code: &mut u8) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut line = 1;

    for c in source.chars() {
        match c {
            '(' => tokens.push(Token::new(TokenType::LEFT_PAREN, String::from("("), Option::from(Literal::Nil), line)),
            ')' => tokens.push(Token::new(TokenType::RIGHT_PAREN, String::from(")"), Option::from(Literal::Nil), line)),
            '{' => tokens.push(Token::new(TokenType::LEFT_BRACE, String::from("{"), Option::from(Literal::Nil), line)),
            '}' => tokens.push(Token::new(TokenType::RIGHT_BRACE, String::from("}"), Option::from(Literal::Nil), line)),
            ',' => tokens.push(Token::new(TokenType::COMMA, String::from(","), Option::from(Literal::Nil), line)),
            '.' => tokens.push(Token::new(TokenType::DOT, String::from("."), Option::from(Literal::Nil), line)),
            '-' => tokens.push(Token::new(TokenType::MINUS, String::from("-"), Option::from(Literal::Nil), line)),
            '+' => tokens.push(Token::new(TokenType::PLUS, String::from("+"), Option::from(Literal::Nil), line)),
            ';' => tokens.push(Token::new(TokenType::SEMICOLON, String::from(";"), Option::from(Literal::Nil), line)),
            '*' => tokens.push(Token::new(TokenType::STAR, String::from("*"), Option::from(Literal::Nil), line)),
            '\n' => line += 1,
            _ => {
                eprintln!("[line {line}] Error: Unexpected character: {c}");
                *error_code = 65;
            }
        }
    }

    tokens.push(Token::new(TokenType::EOF, String::new(), Option::from(Literal::Nil), 0));
    tokens
}

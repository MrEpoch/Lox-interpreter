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

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
enum TokenType {
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
            line,
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
    let mut current: usize = 0;
    let mut char_array = source.chars().collect::<Vec<char>>();
    let char_count = char_array.len();

    while current < char_count {
        let c = char_array.get(current).unwrap();
        current += 1;
        match c {
            '(' => tokens.push(Token::new(
                TokenType::LEFT_PAREN,
                String::from("("),
                Option::from(Literal::Nil),
                line,
            )),
            ')' => tokens.push(Token::new(
                TokenType::RIGHT_PAREN,
                String::from(")"),
                Option::from(Literal::Nil),
                line,
            )),
            '{' => tokens.push(Token::new(
                TokenType::LEFT_BRACE,
                String::from("{"),
                Option::from(Literal::Nil),
                line,
            )),
            '}' => tokens.push(Token::new(
                TokenType::RIGHT_BRACE,
                String::from("}"),
                Option::from(Literal::Nil),
                line,
            )),
            ',' => tokens.push(Token::new(
                TokenType::COMMA,
                String::from(","),
                Option::from(Literal::Nil),
                line,
            )),
            '.' => tokens.push(Token::new(
                TokenType::DOT,
                String::from("."),
                Option::from(Literal::Nil),
                line,
            )),
            '-' => tokens.push(Token::new(
                TokenType::MINUS,
                String::from("-"),
                Option::from(Literal::Nil),
                line,
            )),
            '+' => tokens.push(Token::new(
                TokenType::PLUS,
                String::from("+"),
                Option::from(Literal::Nil),
                line,
            )),
            ';' => tokens.push(Token::new(
                TokenType::SEMICOLON,
                String::from(";"),
                Option::from(Literal::Nil),
                line,
            )),
            '*' => tokens.push(Token::new(
                TokenType::STAR,
                String::from("*"),
                Option::from(Literal::Nil),
                line,
            )),
            '!' => {
                let is_bang = match_operator(&mut char_array, &mut current, '=', char_count);
                tokens.push(Token::new(
                    if is_bang {
                        TokenType::BANG_EQUAL
                    } else {
                        TokenType::BANG
                    },
                    if is_bang {
                        String::from("!=")
                    } else {
                        String::from("!")
                    },
                    Option::from(Literal::Nil),
                    line,
                ))
            }
            '=' => {
                let is_equal = match_operator(&mut char_array, &mut current, '=', char_count);
                tokens.push(Token::new(
                    if is_equal {
                        TokenType::EQUAL_EQUAL
                    } else {
                        TokenType::EQUAL
                    },
                    if is_equal {
                        String::from("==")
                    } else {
                        String::from("=")
                    },
                    Option::from(Literal::Nil),
                    line,
                ))
            }
            '<' => {
                let is_less = match_operator(&mut char_array, &mut current, '=', char_count);
                tokens.push(Token::new(
                    if is_less {
                        TokenType::LESS_EQUAL
                    } else {
                        TokenType::LESS
                    },
                    if is_less {
                        String::from("<=")
                    } else {
                        String::from("<")
                    },
                    Option::from(Literal::Nil),
                    line,
                ))
            }
            '>' => {
                let is_greater = match_operator(&mut char_array, &mut current, '=', char_count);
                tokens.push(Token::new(
                    if is_greater {
                        TokenType::GREATER_EQUAL
                    } else {
                        TokenType::GREATER
                    },
                    if is_greater {
                        String::from(">=")
                    } else {
                        String::from(">")
                    },
                    Option::from(Literal::Nil),
                    line,
                ))
            }
            '/' => {
                let matched = match_operator(&mut char_array, &mut current, '/', char_count);
                if matched {
                    while peek(&mut char_array, &mut current, char_count) != '\n'
                        && !is_end(&mut current, char_count)
                    {
                        current += 1;
                    }
                } else {
                    tokens.push(Token::new(
                        TokenType::SLASH,
                        String::from("/"),
                        Option::from(Literal::Nil),
                        line,
                    ));
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => line += 1,
            _ => {
                eprintln!("[line {line}] Error: Unexpected character: {c}");
                *error_code = 65;
            }
        }
    }

    fn is_end(current: &mut usize, char_count: usize) -> bool {
        if char_count <= *current {
            true
        } else {
            false
        }
    }

    fn match_operator(
        char_array: &mut Vec<char>,
        current: &mut usize,
        operator: char,
        char_count: usize,
    ) -> bool {
        if is_end(current, char_count) || (*char_array.get(*current).unwrap() != operator) {
            return false;
        }
        *current += 1;
        true
    }

    fn peek(char_array: &mut Vec<char>, current: &mut usize, char_count: usize) -> char {
        if is_end(current, char_count) {
            '\0'
        } else {
            *char_array.get(*current).unwrap()
        }
    }

    tokens.push(Token::new(
        TokenType::EOF,
        String::new(),
        Option::from(Literal::Nil),
        0,
    ));
    tokens
}

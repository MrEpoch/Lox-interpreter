use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

#[derive(Debug)]
enum Literal {
    String(String),
    Int(i32),
    Number((f64, usize)),
    Float(f32),
    Bool(bool),
    Null,
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
    line: u32,
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
                let mut scanner = Scanner::new();
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
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
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
        Literal::String(s) => s.clone(),
        Literal::Int(i) => i.to_string(),
        Literal::Number(f) => if (f.0 % 1.0).abs() < f64::EPSILON {
            f.0.to_string() + ".0"
        } else {
            f.0.to_string()
        }
        Literal::Float(f) => f.to_string(),
        Literal::Bool(b) => b.to_string(),
        Literal::Null => String::from("null"),
    }
}

struct Scanner {
    pub tokens: Vec<Token>,
}

impl Scanner {
    fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    fn scan_tokens(&mut self, source: &String, error_code: &mut u8) {
        self.tokens = scan_tokens(source, error_code);
    }
}

fn scan_tokens(source: &String, error_code: &mut u8) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut line: u32 = 1;
    let mut current: usize = 0;
    let mut start: usize = 0;
    let mut char_array = source.chars().collect::<Vec<char>>();
    let char_count = char_array.len();

    while current < char_count {
        let c = char_array.get(current).unwrap();
        start = current;
        current += 1;
        match c {
            '(' => tokens.push(Token::new(
                TokenType::LEFT_PAREN,
                String::from("("),
                Option::from(Literal::Null),
                line,
            )),
            ')' => tokens.push(Token::new(
                TokenType::RIGHT_PAREN,
                String::from(")"),
                Option::from(Literal::Null),
                line,
            )),
            '{' => tokens.push(Token::new(
                TokenType::LEFT_BRACE,
                String::from("{"),
                Option::from(Literal::Null),
                line,
            )),
            '}' => tokens.push(Token::new(
                TokenType::RIGHT_BRACE,
                String::from("}"),
                Option::from(Literal::Null),
                line,
            )),
            ',' => tokens.push(Token::new(
                TokenType::COMMA,
                String::from(","),
                Option::from(Literal::Null),
                line,
            )),
            '.' => tokens.push(Token::new(
                TokenType::DOT,
                String::from("."),
                Option::from(Literal::Null),
                line,
            )),
            '-' => tokens.push(Token::new(
                TokenType::MINUS,
                String::from("-"),
                Option::from(Literal::Null),
                line,
            )),
            '+' => tokens.push(Token::new(
                TokenType::PLUS,
                String::from("+"),
                Option::from(Literal::Null),
                line,
            )),
            ';' => tokens.push(Token::new(
                TokenType::SEMICOLON,
                String::from(";"),
                Option::from(Literal::Null),
                line,
            )),
            '*' => tokens.push(Token::new(
                TokenType::STAR,
                String::from("*"),
                Option::from(Literal::Null),
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
                    Option::from(Literal::Null),
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
                    Option::from(Literal::Null),
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
                    Option::from(Literal::Null),
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
                    Option::from(Literal::Null),
                    line,
                ))
            }
            '/' => {
                let matched = match_operator(&mut char_array, &mut current, '/', char_count);
                if matched {
                    while peek(&mut char_array, current, char_count) != '\n'
                        && !is_end(current, char_count)
                    {
                        current += 1;
                    }
                } else {
                    tokens.push(Token::new(
                        TokenType::SLASH,
                        String::from("/"),
                        Option::from(Literal::Null),
                        line,
                    ));
                }
            }
            '"' => {
                match string_process(&mut char_array, &mut current, char_count, start, &mut line) {
                    Ok(string) => {
                        *error_code = 0;
                        tokens.push(Token::new(
                            TokenType::STRING,
                            string.clone(),
                            Option::from(if string.len() > 3 {
                                // Need to cut \ for string value "
                                Literal::String(string[1..string.len() - 1].to_string())
                            } else {
                                Literal::String(String::new())
                            }),
                            line,
                        ));
                    }
                    Err(_) => {
                        *error_code = 65;
                    }
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => line += 1,
            _ => {
                if is_digit(*c) {
                    let number = number_process(&mut char_array, &mut current, start, char_count);
                    tokens.push(Token::new(
                        TokenType::NUMBER,
                        number.2,
                        Option::from(Literal::Number((number.0, number.1))),
                        line,
                    ));
                } else if is_alpha(*c) {
                    let identifier_value = identifier(&mut char_array, &mut current, char_count, start);
                    tokens.push(Token::new(
                        TokenType::IDENTIFIER,
                        identifier_value,
                        Option::from(Literal::Null),
                        line
                    ))
                } else {
                    eprintln!("[line {line}] Error: Unexpected character: {c}");
                    *error_code = 65;
                }
            }
        }
    }

    fn is_alpha(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        is_alpha(c) || is_digit(c)
    }

    fn identifier(char_array: &mut Vec<char>, current: &mut usize, char_count: usize, start: usize) -> String {
        while is_alpha_numeric(peek(char_array, *current, char_count)) {
            *current += 1;
        }

        char_array[start..*current]
            .iter()
            .collect::<String>()
    }

    fn is_digit(c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn number_process(
        char_array: &mut Vec<char>,
        current: &mut usize,
        start: usize,
        char_count: usize,
    ) -> (f64, usize, String) {
        let mut peeked_value: char = peek(char_array, *current, char_count);

        while is_digit(peeked_value) && !is_end(*current, char_count) {
            *current += 1;
            peeked_value = peek(char_array, *current, char_count);
        }

        let mut formatting_size: usize = 0;
        // This uses regular peek fn, but adds current + 1, it will not throw cause we handle that
        // in peek() or more exactly is_end()
        if peeked_value == '.' && is_digit(peek(char_array, *current + 1, char_count)) {
            *current += 1;
            peeked_value = peek(char_array, *current, char_count);
            while is_digit(peeked_value) && !is_end(*current, char_count) {
                formatting_size += 1;
                *current += 1;
                peeked_value = peek(char_array, *current, char_count);
            }
        } else {
            formatting_size += 1;
        }

        let number = char_array[start..*current]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .unwrap();
        let string = char_array[start..*current].iter().collect::<String>();

        (number, formatting_size, string)
    }

    fn string_process(
        char_array: &mut Vec<char>,
        current: &mut usize,
        char_count: usize,
        start: usize,
        line: &mut u32,
    ) -> Result<String, u8> {
        let mut peeked_value: char = peek(char_array, *current, char_count);
        while peeked_value != '"' && !is_end(*current, char_count) {
            if peeked_value == '\n' {
                *line += 1;
            }
            *current += 1;
            peeked_value = peek(char_array, *current, char_count);
        }

        if is_end(*current, char_count) {
            eprintln!("[line {line}] Error: Unterminated string.");
            return Err(65);
        }

        *current += 1;

        if (*current - 2) == start {
            Ok(['"', '"'].iter().collect::<String>())
        } else {
            Ok(char_array[start..*current].iter().collect::<String>())
        }
    }

    fn is_end(current: usize, char_count: usize) -> bool {
        if char_count <= current {
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
        if is_end(*current, char_count) || (*char_array.get(*current).unwrap() != operator) {
            return false;
        }
        *current += 1;
        true
    }

    fn peek(char_array: &mut Vec<char>, current: usize, char_count: usize) -> char {
        if is_end(current, char_count) {
            '\0'
        } else {
            *char_array.get(current).unwrap()
        }
    }

    tokens.push(Token::new(
        TokenType::EOF,
        String::new(),
        Option::from(Literal::Null),
        line,
    ));
    tokens
}

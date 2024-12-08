use crate::{Literal, Token, TokenType, RESERVED_KEYWORDS};

pub struct Scanner {
    pub tokens: Vec<Token>,
    current: usize,
    char_count: usize,
    start: usize,
    line: u32,
    char_array: Vec<char>,
}

impl Scanner {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
            char_count: 0,
            start: 0,
            line: 1,
            char_array: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self, source: &String, error_code: &mut u8) {
        self.char_array = source.chars().collect::<Vec<char>>();
        self.char_count = self.char_array.len();

        while self.current < self.char_count {
            let c = self.char_array.get(self.current).unwrap();
            self.start = self.current;
            self.current += 1;
            match c {
                '(' => self.tokens.push(Token::new(
                    TokenType::LEFT_PAREN,
                    String::from("("),
                    Option::from(Literal::Null),
                    self.line,
                )),
                ')' => self.tokens.push(Token::new(
                    TokenType::RIGHT_PAREN,
                    String::from(")"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '{' => self.tokens.push(Token::new(
                    TokenType::LEFT_BRACE,
                    String::from("{"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '}' => self.tokens.push(Token::new(
                    TokenType::RIGHT_BRACE,
                    String::from("}"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                ',' => self.tokens.push(Token::new(
                    TokenType::COMMA,
                    String::from(","),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '.' => self.tokens.push(Token::new(
                    TokenType::DOT,
                    String::from("."),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '-' => self.tokens.push(Token::new(
                    TokenType::MINUS,
                    String::from("-"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '+' => self.tokens.push(Token::new(
                    TokenType::PLUS,
                    String::from("+"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                ';' => self.tokens.push(Token::new(
                    TokenType::SEMICOLON,
                    String::from(";"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '*' => self.tokens.push(Token::new(
                    TokenType::STAR,
                    String::from("*"),
                    Option::from(Literal::Null),
                    self.line,
                )),
                '!' => {
                    let is_bang = self.match_operator('=');
                    self.tokens.push(Token::new(
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
                        self.line,
                    ))
                }
                '=' => {
                    let is_equal = self.match_operator('=');
                    self.tokens.push(Token::new(
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
                        self.line,
                    ))
                }
                '<' => {
                    let is_less = self.match_operator('=');
                    self.tokens.push(Token::new(
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
                        self.line,
                    ))
                }
                '>' => {
                    let is_greater = self.match_operator('=');
                    self.tokens.push(Token::new(
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
                        self.line,
                    ))
                }
                '/' => {
                    let matched = self.match_operator('/');
                    if matched {
                        while self.peek() != '\n' && !self.is_end() {
                            self.current += 1;
                        }
                    } else {
                        self.tokens.push(Token::new(
                            TokenType::SLASH,
                            String::from("/"),
                            Option::from(Literal::Null),
                            self.line,
                        ));
                    }
                }
                '"' => {
                    match self.string_process() {
                        Ok(string) => {
                            *error_code = 0;
                            self.tokens.push(Token::new(
                                TokenType::STRING,
                                string.clone(),
                                Option::from(if string.len() > 1 {
                                    // Need to cut \ for string value "
                                    Literal::String(string[1..string.len() - 1].to_string())
                                } else {
                                    Literal::String(String::new())
                                }),
                                self.line,
                            ));
                        }
                        Err(_) => {
                            *error_code = 65;
                        }
                    }
                }
                ' ' | '\r' | '\t' => (),
                '\n' => self.line += 1,
                _ => {
                    if self.is_digit(*c) {
                        let number = self.number_process();
                        self.tokens.push(Token::new(
                            TokenType::NUMBER,
                            format!("{:.*}", number.1, number.0),
                            Option::from(Literal::Number((number.0, number.1))),
                            self.line,
                        ));
                    } else if self.is_alpha(*c) {
                        let identifier_value = self.identifier();
                        self.tokens.push(Token::new(
                            identifier_value.1,
                            identifier_value.0,
                            Option::from(Literal::Null),
                            self.line,
                        ))
                    } else {
                        eprintln!("[line {}] Error: Unexpected character: {}", self.line, c);
                        *error_code = 65;
                    }
                }
            }
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            Option::from(Literal::Null),
            self.line,
        ));
    }

    fn is_end(&mut self) -> bool {
        if self.char_count <= self.current {
            true
        } else {
            false
        }
    }

    fn is_alpha_numeric(&mut self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn identifier(&mut self) -> (String, TokenType) {
        let mut peeked_value = self.peek();
        while self.is_alpha_numeric(peeked_value) {
            self.current += 1;
            peeked_value = self.peek();
        }

        let type_of_token = self.char_array[self.start..self.current]
            .iter()
            .collect::<String>();

        if RESERVED_KEYWORDS
            .lock()
            .unwrap()
            .contains_key(&type_of_token.as_str())
        {
            (
                type_of_token.clone(),
                TokenType::from(
                    RESERVED_KEYWORDS
                        .lock()
                        .unwrap()
                        .get(&type_of_token.as_str())
                        .unwrap()
                        .clone(),
                ),
            )
        } else {
            (type_of_token.clone(), TokenType::IDENTIFIER)
        }
    }

    fn is_digit(&self, c: char) -> bool {
        c >= '0' && c <= '9'
    }

    fn number_process(&mut self) -> (f64, usize, String) {
        let mut peeked_value: char = self.peek();

        while self.is_digit(peeked_value) && !self.is_end() {
            self.current += 1;
            peeked_value = self.peek();
        }

        let mut formatting_size: usize = 0;
        // This uses regular peek fn, but adds current + 1, it will not throw cause we handle that
        // in peek() or more exactly is_end()
        if peeked_value == '.' {
            self.current += 1;
            peeked_value = self.peek();
            while self.is_digit(peeked_value) && !self.is_end() {
                formatting_size += 1;
                self.current += 1;
                peeked_value = self.peek();
            }
        }

        let number = self.char_array[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse::<f64>()
            .unwrap();
        let string = self.char_array[self.start..self.current]
            .iter()
            .collect::<String>();

        (number, formatting_size, string)
    }

    fn string_process(&mut self) -> Result<String, u8> {
        let mut peeked_value: char = self.peek();
        while peeked_value != '"' && !self.is_end() {
            if peeked_value == '\n' {
                self.line += 1;
            }
            self.current += 1;
            peeked_value = self.peek();
        }

        if self.is_end() {
            eprintln!("[line {}] Error: Unterminated string.", self.line);
            return Err(65);
        }

        self.current += 1;

        if (self.current - 2) == self.start {
            Ok(['"', '"'].iter().collect::<String>())
        } else {
            Ok(self.char_array[self.start..self.current]
                .iter()
                .collect::<String>())
        }
    }

    fn match_operator(&mut self, operator: char) -> bool {
        if self.is_end() || (*self.char_array.get(self.current).unwrap() != operator) {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&mut self) -> char {
        if self.is_end() {
            '\0'
        } else {
            *self.char_array.get(self.current).unwrap()
        }
    }
}

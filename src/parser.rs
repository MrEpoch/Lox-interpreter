use std::process::exit;

use crate::{language_error, Expr, Literal, Token, TokenType};


pub struct Parser {
    pub tokens: Vec<Token>,
    pub statements: Vec<Expr>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0, statements: vec![] }
    }

    pub fn expression(&mut self) -> Expr {
        self.equality()
    }

    // !=, ==
    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.comparison();
            expr = Expr::Binary { operator, left: Box::new(expr), right: Box::new(right) };
        }

        expr
    }

    // >, >=, <, <=
    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::GREATER, TokenType::GREATER_EQUAL, TokenType::LESS, TokenType::LESS_EQUAL]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.term();
            expr = Expr::Binary { operator, left: Box::new(expr), right: Box::new(right) };
        }

        expr
    }

    // +, -
    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::MINUS, TokenType::PLUS]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.factor();
            expr = Expr::Binary { operator, left: Box::new(expr), right: Box::new(right) };
        }

        expr
    }

    // /, *
    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        let mut right: Expr;
        let mut operator: Token;

        while self.match_operators(vec![TokenType::SLASH, TokenType::STAR]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.unary();
            expr = Expr::Binary { operator, left: Box::new(expr), right: Box::new(right) };
        }

        expr
    }

    // !, -
    fn unary(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.tokens.get(self.current - 1).unwrap().clone();
            let right = self.unary();
            Expr::Unary { operator: operator.clone(), right: Box::new(right) }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::FALSE]) {
            return Expr::Literal(Literal::Bool(false));
        } else if self.match_operators(vec![TokenType::TRUE]) {
            return Expr::Literal(Literal::Bool(true));
        } else if self.match_operators(vec![TokenType::NIL]) {
            return Expr::Literal(Literal::Nil);
        }

        if self.match_operators(vec![TokenType::NUMBER, TokenType::STRING]) {
            let operator = self.tokens.get(self.current - 1).unwrap().clone();
            return Expr::Literal(operator.clone().literal.unwrap());
        }

        if self.match_operators(vec![TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(vec![expr]);
        }


        // println!("Expect expression.");
        exit(65);
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_end() {
            if self.tokens.get(self.current - 1).unwrap().token_type == TokenType::SEMICOLON {
                return;
            }

            match self.tokens.get(self.current - 1).unwrap().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.check(token_type) {
            self.advance();
            return;
        }

        exit(65);
    }

    fn match_operators(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            } 
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn is_end(&self) -> bool {
        if self.peek().token_type == TokenType::EOF {
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn advance(&mut self) {
        if !self.is_end() {
            self.current += 1;
        } else {
            self.tokens.get(self.current - 1).unwrap();
        }
    }

    pub fn parse(&mut self) {
        let mut statement;
        while !self.is_end() {
            statement = self.statement();
            self.statements.push(statement);
        }
    }

    fn statement(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::PRINT]) {
            Expr::Print(Box::new(self.print_statement()))
        } else {
            self.expression_statement()
        }     
    }

    fn print_statement(&mut self) -> Expr {
        let value = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.");

        value
    }

    fn expression_statement(&mut self) -> Expr {
        let expr = self.expression();
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.");

        expr
    }
}

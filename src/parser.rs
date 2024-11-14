use std::process::exit;

use crate::{language_error, Expr, Literal, Token, TokenType};


pub struct Parser {
    pub tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
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
            right = self.factor();
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

        // self.throw_error(self.peek().clone(), "Expect expression.");
        exit(65);
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.check(token_type) {
            self.advance();
            return;
        }

        // self.throw_error(self.peek().clone(), message);
        exit(65);
    }

    fn throw_error(&self, token: Token, message: &str) {
        language_error(token, message);
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
}

use std::process::exit;

use crate::{environment::Environment, Expr, Literal, Token, TokenType};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub statements: Vec<Expr>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            statements: vec![],
        }
    }

    fn invalid_error(&self, message: String) -> Expr {
        // println!("{}", message);
        exit(65)
    }

    fn and(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.equality();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::AND]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.and();
            expr = Expr::Logical(Box::new(expr), Box::new(right), operator.clone().token_type);
        }
        expr
    }

    fn or(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.and();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::OR]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.and();
            expr = Expr::Logical(Box::new(expr), Box::new(right), operator.clone().token_type);
        }

        expr
    }

    fn assignment(&mut self) -> Expr {
        let expr = self.or();

        if self.match_operators(vec![TokenType::EQUAL]) {
            //  In case of error   let equals = self.tokens.get(self.current - 1).unwrap().clone();
            let value = self.assignment();

            match &expr {
                Expr::Var(t) => {
                    return Expr::Assign {
                        name: String::from(t.lexeme.clone()),
                        value: Box::new(value),
                    };
                }
                _ => {
                    // println!("err");
                    // Error
                    self.invalid_error(String::from("Invalid assignment target"));
                }
            }
        }

        expr
    }

    pub fn expression(&mut self) -> Expr {
        self.assignment()
    }

    // !=, ==
    fn equality(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.comparison();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.comparison();
            expr = Expr::Binary {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    // >, >=, <, <=
    fn comparison(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.term();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
        ]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.term();
            expr = Expr::Binary {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    // +, -
    fn term(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.factor();
        let mut operator: Token;
        let mut right: Expr;

        while self.match_operators(vec![TokenType::MINUS, TokenType::PLUS]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.factor();
            expr = Expr::Binary {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    // /, *
    fn factor(&mut self) -> Expr {
        // Variables will be enums, having them mutable, reduces the number of heap allocations

        let mut expr = self.unary();
        let mut right: Expr;
        let mut operator: Token;

        while self.match_operators(vec![TokenType::SLASH, TokenType::STAR]) {
            operator = self.tokens.get(self.current - 1).unwrap().clone();
            right = self.unary();
            expr = Expr::Binary {
                operator,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        expr
    }

    // !, -
    fn unary(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.tokens.get(self.current - 1).unwrap().clone();
            let right = self.unary();
            Expr::Unary {
                operator: operator.clone(),
                right: Box::new(right),
            }
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Expr {
        // Variable will be enum, having it mutable, reduces the number of heap allocations

        let mut expr = self.primary();

        loop {
            if self.match_operators(vec![TokenType::LEFT_PAREN]) {
                expr = self.finish_call(expr);
            } else {
                break;
            }
        }

        expr
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

        if self.match_operators(vec![TokenType::IDENTIFIER]) {
            return Expr::Var(self.tokens.get(self.current - 1).unwrap().clone());
        }

        if self.match_operators(vec![TokenType::LEFT_PAREN]) {
            let expr = self.expression();
            self.consume(TokenType::RIGHT_PAREN, "Expect ')' after expression.");
            return Expr::Grouping(vec![expr]);
        }

        self.invalid_error(String::from("Expect expression."))
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

    fn consume(&mut self, token_type: TokenType, _message: &str) -> &Token {
        if self.check(token_type) {
            return self.advance();
        }
        self.invalid_error(String::from("Expect '") + &token_type.to_string() + "'.");
        self.tokens.get(self.current - 1).unwrap()
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

    fn advance(&mut self) -> &Token {
        if !self.is_end() {
            self.current += 1;
        }
        self.tokens.get(self.current - 1).unwrap()
    }

    pub fn parse(&mut self) {
        // Variable will be enum, having it mutable, reduces the number of heap allocations

        let mut declaration;
        while !self.is_end() {
            declaration = self.declaration();
            self.statements.push(declaration);
        }
    }

    fn declaration(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::FUN]) {
            return self.function(String::from("function"));
        }
        if self.match_operators(vec![TokenType::VAR]) {
            match self.var_declaration() {
                Some(expr) => expr,
                None => {
                    self.synchronize();
                    Expr::Nil
                }
            }
        } else {
            self.statement()
        }
    }

    fn function(&mut self, kind: String) -> Expr {
        let name = self
            .consume(TokenType::IDENTIFIER, &format!("Expect {} name.", kind))
            .clone();

        self.consume(
            TokenType::LEFT_PAREN,
            &format!("Expect '(' after {} name.", kind),
        );
        let mut parameters: Vec<Token> = vec![];

        if !self.check(TokenType::RIGHT_PAREN) {
            parameters.push(
                self.consume(TokenType::IDENTIFIER, "Expect parameter name.")
                    .clone(),
            );
            while self.match_operators(vec![TokenType::COMMA]) {
                if parameters.len() >= 250 {
                    self.invalid_error(String::from("Cannot have more than 250 parameters."));
                }

                parameters.push(
                    self.consume(TokenType::IDENTIFIER, "Expect parameter name.")
                        .clone(),
                );
            }
        }

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after parameters.");

        self.consume(
            TokenType::LEFT_BRACE,
            &format!("Expect '{{' before {} body.", kind),
        );

        let body = self.block();

        Expr::Function {
            name,
            params: parameters,
            body,
            environment: None,
        }
    }

    fn var_declaration(&mut self) -> Option<Expr> {
        let variable = self.consume(TokenType::IDENTIFIER, "Expect variable name.");
        let variable_name: String;

        let mut initializer = Expr::Nil;
        variable_name = String::from(variable.lexeme.clone());

        if self.match_operators(vec![TokenType::EQUAL]) {
            initializer = self.expression();
        }

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        );

        Some(Expr::Variable {
            name: variable_name,
            value: Box::new(initializer),
        })
    }

    fn statement(&mut self) -> Expr {
        if self.match_operators(vec![TokenType::FOR]) {
            return self.for_statement();
        }

        if self.match_operators(vec![TokenType::IF]) {
            return self.if_statement();
        }

        if self.match_operators(vec![TokenType::PRINT]) {
            return Expr::Print(Box::new(self.print_statement()));
        }

        if self.match_operators(vec![TokenType::RETURN]) {
            return self.return_statement();
        }

        if self.match_operators(vec![TokenType::WHILE]) {
            return self.while_statement();
        }

        if self.match_operators(vec![TokenType::LEFT_BRACE]) {
            return Expr::Block(self.block());
        }

        self.expression_statement()
    }

    fn while_statement(&mut self) -> Expr {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after 'if'.");

        let body = self.statement();

        Expr::While(Box::new(condition), Box::new(body))
    }

    fn for_statement(&mut self) -> Expr {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'for'.");
        let initializer: Option<Expr>;

        if self.match_operators(vec![TokenType::SEMICOLON]) {
            initializer = None;
        } else if self.match_operators(vec![TokenType::VAR]) {
            initializer = self.var_declaration();
        } else {
            initializer = Some(self.expression_statement());
        }

        let mut condition: Option<Expr> = None;

        if !self.check(TokenType::SEMICOLON) {
            condition = Some(self.expression());
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.");

        let mut increment: Option<Expr> = None;

        if !self.check(TokenType::RIGHT_PAREN) {
            increment = Some(self.expression());
        }

        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after the clauses.");

        let mut body = self.statement();

        if increment != None {
            body = Expr::Block(vec![body, Expr::Increment(Box::new(increment.unwrap()))])
        }

        if condition == None {
            condition = Some(Expr::Literal(Literal::Bool(true)));
        }

        body = Expr::While(Box::new(condition.unwrap()), Box::new(body));

        if initializer != None {
            body = Expr::Block(vec![initializer.unwrap(), body]);
        }

        body
    }

    fn if_statement(&mut self) -> Expr {
        self.consume(TokenType::LEFT_PAREN, "Expect '(' after 'if'.");
        let condition = self.expression();
        self.consume(TokenType::RIGHT_PAREN, "Expect ')' after 'if'.");

        let then_branch = self.statement();
        let mut else_branch: Option<Box<Expr>> = None;

        if self.match_operators(vec![TokenType::ELSE]) {
            else_branch = Some(Box::new(self.statement()));
        }

        Expr::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        }
    }

    fn block(&mut self) -> Vec<Expr> {
        let mut statements = vec![];

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_end() {
            statements.push(self.declaration());
        }

        self.consume(TokenType::RIGHT_BRACE, "Expect '}' after block.");

        statements
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

    fn return_statement(&mut self) -> Expr {
        let keyword = self.tokens.get(self.current - 1).unwrap().clone();
        let mut value = Expr::Nil;

        if !self.check(TokenType::SEMICOLON) {
            value = self.expression();
        }

        self.consume(TokenType::SEMICOLON, "Expect ';' after return value.");

        Expr::Return(keyword, Box::new(value))
    }

    fn finish_call(&mut self, expr: Expr) -> Expr {
        // Variable will be enum, having it mutable, reduces the number of heap allocations

        let mut arguments = vec![];

        if !self.check(TokenType::RIGHT_PAREN) {
            arguments.push(self.expression());
            while self.match_operators(vec![TokenType::COMMA]) {
                if arguments.len() >= 255 {
                    self.invalid_error(String::from("Can't have more than 255 arguments."));
                }
                arguments.push(self.expression());
            }
        }

        let paren = self.consume(TokenType::RIGHT_PAREN, "Expect ')' after arguments.");

        Expr::Call(Box::new(expr), paren.clone(), arguments)
    }
}

use std::process::exit;

use crate::interpreter::{Expr, Literal, Token, TokenType};

pub fn print_based_on_literal(literal: &Literal) -> String {
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

pub fn handle_grouping(exprs: Vec<Expr>, left_side: &String, right_side: &String) -> Vec<String> {
    let mut r: Vec<String> = vec![];
    for e in exprs {
        r.push(handle_match(e, left_side, right_side));
    }
    r
}

pub fn handle_match(expr: Expr, left_side: &String, right_side: &String) -> String {
    match expr {
        Expr::Grouping(exprs) => {
            format!(
                "{left_side}{}{right_side}",
                handle_grouping(exprs, &format!("(group "), &format!(")")).join(" ")
            )
        }
        Expr::Number(n) => {
            format!("{left_side}{n}{right_side}")
        }
        Expr::Binary {
            operator,
            left,
            right,
        } => {
            format!(
                "{left_side}({} {} {}){right_side}",
                operator.lexeme,
                handle_match(*left, &String::from(""), &String::from("")),
                handle_match(*right, &String::from(""), &String::from(""))
            )
        }
        Expr::Literal(l) => {
            format!("{left_side}{}{right_side}", print_based_on_literal(&l))
        }
        Expr::Unary { operator, right } => {
            format!(
                "{left_side}({} {}){right_side}",
                operator.lexeme,
                handle_match(*right, &String::from(""), &String::from(""))
            )
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

pub fn get_from_unary(expr_unary: Expr) -> String {
    if let Expr::Unary { operator, right } = expr_unary {
        format!("({} {})", operator.lexeme, get_from_unary(*right))
    } else if let Expr::Literal(literal) = expr_unary {
        print_based_on_literal(&literal)
    } else {
        expr_unary.to_string()
    }
}

/*

pub fn language_error(token: Token, message: &str) {
    if token.token_type == TokenType::EOF {
        println!("[line {}] at end {message}", token.line);
    } else {
        println!("[line {}] at end '{}' {message}", token.line, token.lexeme);
    }
    exit(65);
}

*/

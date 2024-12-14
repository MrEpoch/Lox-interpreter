use std::env;
use std::fs;
use std::io::{self, Write};

use formatters::parse_eval_inter;
use interpreter::{Expr, Literal, Token, TokenType};

mod environment;
mod evaluator;
mod formatters;
mod interpreter;
mod parser;
mod runner;
mod scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];
    let interpreter = interpreter::Interpreter::new(filename);

    match command.as_str() {
        "tokenize" => {
            interpreter.tokenize();
        }
        "parse" => {
            interpreter.parse();
        }
        "evaluate" => {
            interpreter.evaluate();
        }
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            if !file_contents.is_empty() {
                let mut scanner = scanner::Scanner::new();
                scanner.scan_tokens(&file_contents, &mut 0);
                let mut parser = parser::Parser::new(scanner.tokens);
                parser.parse();
                let evaluator = evaluator::Evaluator::new();
                let mut environment = environment::Environment::new();
                for s in parser.statements.iter() {
                    let evaluated = evaluator.evaluate(s, &mut environment);
                    parse_eval_inter(evaluated);
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

use interpreter::{Expr, Literal, Token, TokenType};
use std::env;
use std::io::{self, Write};

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
    let mut interpreter = interpreter::Interpreter::new(filename);

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
            interpreter.run();
        }
        _ => {
            writeln!(io::stderr(), "Unknown command: {}", command).unwrap();
            return;
        }
    }
}

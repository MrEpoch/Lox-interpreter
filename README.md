---
This is a starting point for Rust solutions to the
["Build your own Interpreter" Challenge](https://app.codecrafters.io/courses/interpreter/overview).

This challenge follows the book
[Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom.
---

# Overview

The project is split into multiple parts here are main ones:

1. scanner.rs - Scannes each character in file (including \n new line) and tokenizes it and then puts it into vector of tokens.
2. parser.rs - Parses tokens into AST.
3. evaluator.rs - Evaluates AST and returns result.
4. runner.rs - Executes AST and logs result.
5. interpreter.rs - Contains all logic like Token struct definitions and Environment definitions.
6. environment.rs - Logis for memory management and variables.
7. formatter.rs - Has some helper functions for formatting output.

The individual parts of project followed through [Crafting interpreters book](https://craftinginterpreters.com/), but book was written in Java 
and this project was made in Rust so it has some inconsistencies.

Working of environment relies on recursion and special Hashmap which manages memory, deep down in recursion i couldn't reference higher up memory
so i had to make somewhat rough solution which allowed me to manipulate memory on different levels.

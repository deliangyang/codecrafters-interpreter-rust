use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::exit;

use codecrafters_interpreter::evaluator::Evaluator;
use codecrafters_interpreter::token::Token;
use codecrafters_interpreter::lexer::Lexing;
use codecrafters_interpreter::parser::Parser;



fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let lex = Lexing::new(&file_contents);
                let mut parse = Parser::new(lex);
                let program = parse.parse();
                if parse.has_errors() {
                    exit(65);
                }
                for stmt in &program {
                    println!("{}", stmt);
                }
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
        }
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let lex = Lexing::new(&file_contents);
                let mut parse = Parser::new(lex);
                let program = parse.parse();
                if parse.has_errors() {
                    exit(65);
                }
                let mut evaluator = Evaluator::new(program, true);
                evaluator.evaluate();
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
        }
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let lex = Lexing::new(&file_contents);
                let mut parse = Parser::new(lex);
                let program = parse.parse();
                if parse.has_errors() {
                    exit(65);
                }
                let imports = parse.get_imports(program.clone());

                let mut prog_modules = Vec::new();
                if let Some(modules) = imports {
                    for (_, module) in modules {
                        let lex = Lexing::new(&module);
                        let mut parse = Parser::new(lex);
                        let prog: Vec<codecrafters_interpreter::ast::Stmt> = parse.parse();
                        if parse.has_errors() {
                            exit(65);
                        }
                        prog_modules.extend(prog);
                    }
                }
                prog_modules.extend(program);

                let mut evaluator = Evaluator::new(prog_modules, false);
                evaluator.evaluate();
            } else {
                println!("EOF  null"); // Placeholder, remove this line when implementing the scanner
            }
        }
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            // writeln!(io::stderr(), "Logs from your program will appear here!").unwrap();

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let mut lex = Lexing::new(&file_contents);
                loop {
                    let token = lex.next();
                    match token {
                        Token::Eof => {
                            println!("{}", token);
                            break;
                        }
                        Token::Comment(_) => {}
                        _ => {
                            println!("{}", token);
                        }
                    }
                }
                let mut return_code = 0;
                if lex.errors.len() > 0 {
                    return_code = 65;
                }
                exit(return_code);
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

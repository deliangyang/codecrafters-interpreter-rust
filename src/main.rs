use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

use codecrafters_interpreter::compiler::Compiler;
use codecrafters_interpreter::evaluator::Evaluator;
use codecrafters_interpreter::imports::Imports;
use codecrafters_interpreter::lexer::Lexing;
use codecrafters_interpreter::opcode::Opcode;
use codecrafters_interpreter::parser::Parser;
use codecrafters_interpreter::token::Token;
use codecrafters_interpreter::vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
        writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
        String::new()
    });

    if file_contents.is_empty() {
        writeln!(io::stderr(), "File is empty").unwrap();
        return;
    }

    match command.as_str() {
        "dump" => {
            let lex = Lexing::new(&file_contents);
            let mut parse = Parser::new(lex);
            let program = parse.parse();
            if parse.has_errors() {
                exit(65);
            }
            let mut compile = Compiler::new(program);
            compile.compile();

            println!("symbols:");
            for (i, symbol) in compile.symbols.borrow().store.iter().enumerate() {
                println!("\t{:04} {:?}", i, symbol);
            }

            println!("\nconstants:");
            for (i, constant) in compile.constants.iter().enumerate() {
                println!("\t{:04} {:?}", i, constant);
            }

            println!("\ninstructions:");
            for (i, instruction) in compile.instructions.iter().enumerate() {
                match instruction {
                    Opcode::GetBuiltin(index) => {
                        println!("\t{:04} {:?}\t\t# {:020}", i, instruction, compile.builtins.get_name(*index).unwrap());
                    }
                    Opcode::LoadConstant(index) => {
                        println!("\t{:04} {:?}\t\t# {:?}", i, instruction, compile.constants[*index]);
                    }
                    _ =>  println!("\t{:04} {:?}", i, instruction)
                }
            }

        }
        "compile" => {
            let lex = Lexing::new(&file_contents);
            let mut parse = Parser::new(lex);
            let program = parse.parse();
            if parse.has_errors() {
                exit(65);
            }
            let mut compile = Compiler::new(program);
            compile.compile();
            let mut vm = VM::new();
            vm.define_constants(compile.constants);
            let result = vm.run(compile.instructions);
            println!("result: {:?}", result);
        }
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

            let file_current_dir = Path::new(filename).parent().unwrap();
            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let lex = Lexing::new(&file_contents);
                let mut parse = Parser::new(lex);
                let program = parse.parse();
                if parse.has_errors() {
                    exit(65);
                }
                let mut import = Imports::new(program, file_current_dir.to_path_buf());
                let mut evaluator = Evaluator::new(import.load(), false);
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

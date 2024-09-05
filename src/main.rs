use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::str::Chars;

#[derive(Debug)]
enum Token {
    VAR,
    IDENTIFIER(String),
    EQUAL,
    STRING(String),
    SEMICOLON,  // ;
    EOF,        // null

    LeftParen, // (
    RightParen, // )

    LeftBrace, // {
    RightBrace, // }
}

static KEYWORDS: [&str; 1] = ["var"];

fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

impl Display for  Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::VAR => write!(f, "VAR var null"),
            Token::IDENTIFIER(s) => write!(f, "IDENTIFIER {} null", s),
            Token::EQUAL => write!(f, "EQUAL = null"),
            Token::STRING(s) => write!(f, "STRING \"{}\" {}", s, s),
            Token::SEMICOLON => write!(f, "SEMICOLON ; null"),
            Token::EOF => write!(f, "EOF  null"),
            Token::LeftParen => write!(f, "LEFT_PAREN ( null"),
            Token::RightParen => write!(f, "RIGHT_PAREN ) null"),
            Token::LeftBrace => write!(f, "LEFT_BRACE {{ null"),
            Token::RightBrace => write!(f, "RIGHT_BRACE }} null"),
        }
    }
}

struct Lexing<'a> {
    input: Chars<'a>,
    position: usize,
    l: usize,
}

impl<'a> Lexing<'a> {
    fn new(input: &str) -> Lexing {
        let l = input.len();
        let input = input.chars();
        Lexing {
            input,
            position: 0,
            l,
        }
    }

    fn get_char(&mut self) ->char {
        self.position += 1;
        self.input.nth(0).unwrap()
    }

    fn peek(&mut self) -> char {
        self.input.clone().nth(0).unwrap()
    }

    fn next(&mut self) -> Token {
        while self.l > self.position  {
            let c = self.peek();
            match c {
                ' ' | '\n' | '\t' => {
                    self.get_char();
                    continue;
                }
                '=' => {
                    self.get_char();
                    return Token::EQUAL;
                }
                ';' => {
                    self.get_char();
                    return Token::SEMICOLON;
                }
                '"' => {
                    let mut s = String::new();
                    self.get_char();
                    while self.l > self.position {
                        let c = self.peek();
                        if c == '"' {
                            self.get_char();
                            return Token::STRING(s);
                        }
                        s.push(self.get_char());
                    }
                    panic!("unterminated string");
                }
                '(' => {
                    self.get_char();
                    return Token::LeftParen;
                }
                ')' => {
                    self.get_char();
                    return Token::RightParen;
                }
                '{' => {
                    self.get_char();
                    return Token::LeftBrace;
                }
                '}' => {
                    self.get_char();
                    return Token::RightBrace;
                }
                _ => {
                    let mut s = String::new();
                    while self.l > self.position {
                        let c = self.peek();
                        if c.is_alphabetic() {
                            s.push(self.get_char());
                        } else {
                            break;
                        }
                    }
                    if is_keyword(&s) {
                        return Token::VAR;
                    } else {
                        return Token::IDENTIFIER(s);
                    }
                }
            }
        }
        return Token::EOF;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        writeln!(io::stderr(), "Usage: {} tokenize <filename>", args[0]).unwrap();
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
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
                        Token::EOF => {
                            println!("EOF  null");
                            break
                        },
                        _ => {
                            println!("{}", token);
                        },
                    }
                }
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

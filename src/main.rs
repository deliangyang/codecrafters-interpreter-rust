use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
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
    // ,, ., -, +, ; and *. /
    Star, // *
    Slash, // /
    Minus, // -
    Plus, // +
    Comma, // ,
    Dot, // .

    EqualEqual, // ==
    Bang, // !
    BangEqual, // !=

    Less, // <
    LessEqual, // <=

    Greater, // >
    GreaterEqual, // >=

    Comment(String), // //
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
            Token::Star => write!(f, "STAR * null"),
            Token::Slash => write!(f, "SLASH / null"),
            Token::Minus => write!(f, "MINUS - null"),
            Token::Plus => write!(f, "PLUS + null"),
            Token::Comma => write!(f, "COMMA , null"),
            Token::Dot => write!(f, "DOT . null"),
            Token::EqualEqual => write!(f, "EQUAL_EQUAL == null"),
            Token::Bang => write!(f, "BANG ! null"),
            Token::BangEqual => write!(f, "BANG_EQUAL != null"),
            Token::Less => write!(f, "LESS < null"),
            Token::LessEqual => write!(f, "LESS_EQUAL <= null"),
            Token::Greater => write!(f, "GREATER > null"),
            Token::GreaterEqual => write!(f, "GREATER_EQUAL >= null"),
            Token::Comment(s) => write!(f, "COMMENT //{} null", s),
        }
    }
}

struct Lexing<'a> {
    input: Chars<'a>,
    position: usize,
    l: usize,
    lines: usize,
    errors: Vec<String>,
}

impl<'a> Lexing<'a> {
    fn new(input: &str) -> Lexing {
        let l = input.chars().count();
        let input = input.chars();
        Lexing {
            input,
            position: 0,
            l,
            lines: 0,
            errors: Vec::new(),
        }
    }

    fn get_char(&mut self) ->char {
        self.position += 1;
        let c = self.input.nth(0).unwrap();
        if c == '\n' {
            self.lines += 1;
        }
        c
    }

    fn peek(&mut self) -> char {
        if self.l > self.position {
            return self.input.clone().nth(0).unwrap();
        }
        return '\0';
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
                    if self.peek() == '=' {
                        self.get_char();
                        return Token::EqualEqual;
                    }
                    return Token::EQUAL;
                }
                ';' => {
                    self.get_char();
                    return Token::SEMICOLON;
                }
                '!' => {
                    self.get_char();
                    if self.peek() == '=' {
                        self.get_char();
                        return Token::BangEqual;
                    }
                    return Token::Bang;
                }
                '<' => {
                    self.get_char();
                    if self.peek() == '=' {
                        self.get_char();
                        return Token::LessEqual;
                    }
                    return Token::Less;
                }
                '>' => {
                    self.get_char();
                    if self.peek() == '=' {
                        self.get_char();
                        return Token::GreaterEqual;
                    }
                    return Token::Greater;
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
                    self.errors.push(format!("[line {}] Error: Unterminated string.", self.lines+1));
                    eprintln!("[line {}] Error: Unterminated string.", self.lines+1);
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
                '*' => {
                    self.get_char();
                    return Token::Star;
                }
                '/' => {
                    self.get_char();
                    if self.peek() == '/' {
                        let mut s = String::new();
                        self.get_char();
                        while self.l > self.position {
                            let c = self.peek();
                            if c == '\n' {
                                self.get_char();
                                break;
                            }
                            s.push(self.get_char());
                        }
                        return Token::Comment(s);
                    }
                    return Token::Slash;
                }
                '-' => {
                    self.get_char();
                    return Token::Minus;
                }
                '+' => {
                    self.get_char();
                    return Token::Plus;
                }
                ',' => {
                    self.get_char();
                    return Token::Comma;
                }
                '.' => {
                    self.get_char();
                    return Token::Dot;
                }
                'Z'..'a' => {
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
                _ => {
                    let c = self.get_char();
                    self.errors.push(format!("[line {}] Error: Unexpected character: {}", self.lines+1, c));
                    eprintln!("[line {}] Error: Unexpected character: {}", self.lines+1, c);
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
                            println!("{}", token);
                            break
                        },
                        Token::Comment(_) => {},
                        _ => {
                            println!("{}", token);
                        },
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

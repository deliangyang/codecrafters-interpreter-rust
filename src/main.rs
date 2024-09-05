use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::fs;
use std::io::{self, Write};
use std::process::exit;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    VAR,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FOR,
    FUN,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    WHILE,
    IDENTIFIER(String),
    EQUAL,
    STRING(String),
    Number(String),
    SEMICOLON, // ;
    EOF,       // null

    LeftParen,  // (
    RightParen, // )

    LeftBrace,  // {
    RightBrace, // }
    // ,, ., -, +, ; and *. /
    Star,  // *
    Slash, // /
    Minus, // -
    Plus,  // +
    Comma, // ,
    Dot,   // .

    EqualEqual, // ==
    Bang,       // !
    BangEqual,  // !=

    Less,      // <
    LessEqual, // <=

    Greater,      // >
    GreaterEqual, // >=

    Comment(String), // //
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::VAR => write!(f, "VAR var null"),
            Token::AND => write!(f, "AND and null"),
            Token::CLASS => write!(f, "CLASS class null"),
            Token::ELSE => write!(f, "ELSE else null"),
            Token::FALSE => write!(f, "FALSE false null"),
            Token::FOR => write!(f, "FOR for null"),
            Token::FUN => write!(f, "FUN fun null"),
            Token::IF => write!(f, "IF if null"),
            Token::NIL => write!(f, "NIL nil null"),
            Token::OR => write!(f, "OR or null"),
            Token::PRINT => write!(f, "PRINT print null"),
            Token::RETURN => write!(f, "RETURN return null"),
            Token::SUPER => write!(f, "SUPER super null"),
            Token::THIS => write!(f, "THIS this null"),
            Token::TRUE => write!(f, "TRUE true null"),
            Token::WHILE => write!(f, "WHILE while null"),
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
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                let inum = (num as i64) as f64;
                if num > inum {
                    write!(f, "NUMBER {} {}", n, num)
                } else {
                    write!(f, "NUMBER {} {}.0", n, num)
                }
            }
        }
    }
}

struct Lexing<'a> {
    input: Chars<'a>,
    position: usize,
    l: usize,
    lines: usize,
    errors: Vec<String>,
    keywords: HashMap<&'static str, Token>,
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
            keywords: HashMap::from([
                ("var", Token::VAR),
                ("and", Token::AND),
                ("class", Token::CLASS),
                ("else", Token::ELSE),
                ("false", Token::FALSE),
                ("for", Token::FOR),
                ("fun", Token::FUN),
                ("if", Token::IF),
                ("nil", Token::NIL),
                ("or", Token::OR),
                ("print", Token::PRINT),
                ("return", Token::RETURN),
                ("super", Token::SUPER),
                ("this", Token::THIS),
                ("true", Token::TRUE),
                ("while", Token::WHILE),
            ]),
        }
    }

    fn is_keyword(&self, var: &str) -> bool {
        self.keywords.contains_key(var)
    }

    fn get_char(&mut self) -> char {
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

    fn peek_n(&mut self, n: usize) -> char {
        if self.l > self.position + n - 1 {
            return self.input.clone().nth(n - 1).unwrap();
        }
        return '\0';
    }

    fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    fn next(&mut self) -> Token {
        while self.l > self.position {
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
                    self.errors.push(format!(
                        "[line {}] Error: Unterminated string.",
                        self.lines + 1
                    ));
                    eprintln!("[line {}] Error: Unterminated string.", self.lines + 1);
                }
                '0'..='9' => {
                    let mut s = String::new();
                    while self.l > self.position {
                        let c = self.peek();
                        if c.is_numeric() {
                            s.push(self.get_char());
                        } else if c == '.' && self.peek_n(2).is_numeric() {
                            s.push(self.get_char());
                        } else {
                            break;
                        }
                    }
                    return Token::Number(s);
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
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut s = String::new();
                    while self.l > self.position {
                        let c = self.peek();
                        if c.is_alphabetic() || c == '_' || c.is_numeric() {
                            s.push(self.get_char());
                        } else {
                            break;
                        }
                    }
                    if self.is_keyword(&s) {
                        match self.keywords.get(s.to_lowercase().as_str()) {
                            Some(t) => return t.clone(),
                            None => {
                                self.errors.push(format!(
                                    "[line {}] Error: Unknown keyword: {}",
                                    self.lines + 1,
                                    s
                                ));
                                eprintln!(
                                    "[line {}] Error: Unknown keyword: {}",
                                    self.lines + 1,
                                    s
                                );
                            }
                        }
                    } else {
                        return Token::IDENTIFIER(s);
                    }
                }
                _ => {
                    let c = self.get_char();
                    self.errors.push(format!(
                        "[line {}] Error: Unexpected character: {}",
                        self.lines + 1,
                        c
                    ));
                    eprintln!(
                        "[line {}] Error: Unexpected character: {}",
                        self.lines + 1,
                        c
                    );
                }
            }
        }
        return Token::EOF;
    }
}

trait Expr {
    fn literal(&self) -> String;
}

struct StringLiteral {
    value: String,
}

impl Expr for StringLiteral {
    fn literal(&self) -> String {
        format!("{}", self.value)
    }
}

struct NumberLiteral {
    value: f64,
}

impl Expr for NumberLiteral {
    fn literal(&self) -> String {
        let inum = (self.value as i64) as f64;
        if self.value > inum {
            format!("{}", self.value)
        } else {
            format!("{}.0", self.value)
        }
    }
}

struct NilLiteral {}

impl Expr for NilLiteral {
    fn literal(&self) -> String {
        "nil".to_string()
    }
}

struct BoolExpr {
    value: bool,
}

impl Expr for BoolExpr {
    fn literal(&self) -> String {
        format!("{}", self.value)
    }
}

struct GroupingExpr {
    exprs: Vec<Box<dyn Expr>>,
}

impl Expr for GroupingExpr {
    fn literal(&self) -> String {
        let mut ss = String::new();
        for e in self.exprs.iter() {
            ss.push_str(&e.literal());
        }
        format!("(group {})", ss)
    }
}

struct UnaryExpr {
    operator: Token,
    right: Box<dyn Expr>,
}

impl Expr for UnaryExpr {
    fn literal(&self) -> String {
        match self.operator {
            Token::Bang => {
                format!("(! {})", self.right.literal())
            }
            Token::Minus => {
                format!("(- {})", self.right.literal())
            }
            _ => {
                format!("({} {})", self.operator, self.right.literal())
            }
        }
    }
}

struct Parse<'a> {
    lex: Lexing<'a>,
    current: Token,
    next: Token,
    prev: Token,
}

impl<'a> Parse<'a> {
    fn new(mut lex: Lexing<'a>) -> Parse {
        let prev = Token::EOF;
        let current = lex.next();
        let next = lex.next();
        Parse {
            lex,
            current,
            next,
            prev,
        }
    }

    fn parse(&mut self) {
        let expr = self.next_expr();
        if let Some(e) = expr {
            println!("{}", e.literal());
        }
    }

    fn next_expr(&mut self) -> Option<Box<dyn Expr>> {
        match self.current.clone() {
            Token::EOF => {
                return None;
            }
            Token::TRUE => {
                self.next();
                return Some(Box::new(BoolExpr { value: true }));
            }
            Token::FALSE => {
                self.next();
                return Some(Box::new(BoolExpr { value: false }));
            }
            Token::NIL => {
                self.next();
                return Some(Box::new(NilLiteral {}));
            }
            Token::STRING(s) => {
                self.next();
                return Some(Box::new(StringLiteral { value: s }));
            }
            Token::Bang => {
                self.next();
                let unary = UnaryExpr {
                    operator: self.prev.clone(),
                    right: self.next_expr().unwrap(),
                };
                return Some(Box::new(unary));
            }
            Token::Minus => {
                self.next();
                let unary = UnaryExpr {
                    operator: self.prev.clone(),
                    right: self.next_expr().unwrap(),
                };
                return Some(Box::new(unary));
            }
            Token::LeftParen => {
                let mut group = GroupingExpr { exprs: Vec::new() };
                self.next();

                while self.current != Token::RightParen {
                    group.exprs.push(self.next_expr().unwrap());
                }
                self.next();
                return Some(Box::new(group));
            }
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                self.next();
                return Some(Box::new(NumberLiteral { value: num }));
            }
            _ => {
                self.next();
            }
        }
        None
    }

    fn next(&mut self) -> Token {
        let next = self.lex.next();
        self.prev = self.current.clone();
        self.current = self.next.clone();
        self.next = next;
        self.current.clone()
    }

    fn has_errors(&self) -> bool {
        self.lex.has_errors()
    }

    // fn peek(&self) -> Token {
    //     self.next.clone()
    // }
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
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                writeln!(io::stderr(), "Failed to read file {}", filename).unwrap();
                String::new()
            });

            // Uncomment this block to pass the first stage
            if !file_contents.is_empty() {
                let lex = Lexing::new(&file_contents);
                let mut parse = Parse::new(lex);
                parse.parse();
                if parse.has_errors() {
                    exit(65);
                }
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
                        Token::EOF => {
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

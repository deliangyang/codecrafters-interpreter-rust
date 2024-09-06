use std::collections::HashMap;
use std::str::Chars;
use crate::token::Token;

pub struct Lexing<'a> {
    input: Chars<'a>,
    position: usize,
    l: usize,
    lines: usize,
    pub errors: Vec<String>,
    keywords: HashMap<&'static str, Token>,
}

impl<'a> Lexing<'a> {
    pub fn new(input: &str) -> Lexing {
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

    pub fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    pub fn next(&mut self) -> Token {
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
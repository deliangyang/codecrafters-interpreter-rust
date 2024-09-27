use crate::token::Token;
use std::collections::HashMap;
use std::str::Chars;

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
                ("var", Token::Var),
                ("and", Token::And),
                ("class", Token::Class),
                ("else", Token::Else),
                ("false", Token::False),
                ("for", Token::For),
                ("fun", Token::Fun),
                ("if", Token::If),
                ("nil", Token::Nil),
                ("or", Token::Or),
                ("print", Token::Print),
                ("return", Token::Return),
                ("super", Token::Super),
                ("this", Token::This),
                ("true", Token::True),
                ("while", Token::While),
                ("switch", Token::Switch),
                ("case", Token::Case),
                ("default", Token::Default),
                ("new", Token::New),
                ("in", Token::In),
                ("import", Token::Import),
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
                    return Token::Equal;
                }
                ';' => {
                    self.get_char();
                    return Token::Semicolon;
                }
                '!' => {
                    self.get_char();
                    if self.peek() == '=' {
                        self.get_char();
                        return Token::BangEqual;
                    }
                    return Token::Bang;
                }
                '|' => {
                    self.get_char();
                    if self.peek() == '|' {
                        self.get_char();
                        return Token::Or;
                    }
                    return Token::BitOr;
                }
                '&' => {
                    self.get_char();
                    if self.peek() == '&' {
                        self.get_char();
                        return Token::And;
                    }
                    return Token::BitAnd;
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
                '\'' => {
                    let mut s = String::new();
                    self.get_char();
                    while self.l > self.position {
                        let mut c = self.peek();
                        if c == '\\' {
                            self.get_char();
                            let c = self.get_char();
                            if c == '\'' {
                                s.push('\'');
                            } else if c == 'r' {
                                s.push('\r');
                            } else if c == 'n' {
                                s.push('\n');
                            } else if c == 't' {
                                s.push('\t');
                            } else if c == '0' {
                                s.push('\0');
                            } else if c == '\\' {
                                s.push('\\');
                            } else if c == '\'' {
                                s.push('\'');
                            } else {
                                self.errors.push(format!(
                                    "[line {}] Error: Unknown escape character: {}",
                                    self.lines + 1,
                                    c
                                ));
                                eprintln!(
                                    "[line {}] Error: Unknown escape character: {}",
                                    self.lines + 1,
                                    c
                                );
                            }
                        }
                        c = self.peek();
                        if c == '\'' {
                            self.get_char();
                            return Token::String(s);
                        }
                        s.push(self.get_char());
                    }
                    self.errors.push(format!(
                        "[line {}] Error: Unterminated string.",
                        self.lines + 1
                    ));
                    eprintln!("[line {}] Error: Unterminated string.", self.lines + 1);
                }
                '"' => {
                    let mut s = String::new();
                    self.get_char();
                    while self.l > self.position {
                        let mut c = self.peek();
                        if c == '\\' {
                            self.get_char();
                            let c = self.get_char();
                            if c == '"' {
                                s.push('"');
                            } else if c == 'r' {
                                s.push('\r');
                            } else if c == 'n' {
                                s.push('\n');
                            } else if c == 't' {
                                s.push('\t');
                            } else if c == '0' {
                                s.push('\0');
                            } else if c == '\\' {
                                s.push('\\');
                            } else if c == '\'' {
                                s.push('\'');
                            } else {
                                self.errors.push(format!(
                                    "[line {}] Error: Unknown escape character: {}",
                                    self.lines + 1,
                                    c
                                ));
                                eprintln!(
                                    "[line {}] Error: Unknown escape character: {}",
                                    self.lines + 1,
                                    c
                                );
                            }
                        }
                        c = self.peek();
                        if c == '"' {
                            self.get_char();
                            return Token::String(s);
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
                ':' => {
                    self.get_char();
                    return Token::Colon;
                }
                '[' => {
                    self.get_char();
                    return Token::LeftBracket;
                }
                ']' => {
                    self.get_char();
                    return Token::RightBracket;
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
                        return Token::Identifier(s);
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
        return Token::Eof;
    }

    pub fn log_error(&mut self, token: Token, message: &str) {
        let token = match token {
            Token::Identifier(s) => s,
            Token::String(s) => s,
            Token::Number(s) => s,
            Token::RightParen => ")".to_string(),
            _ => "".to_string(),
        };
        self.errors.push(format!(
            "[line {}] Error at '{}': {}.",
            self.lines + 1,
            token,
            message
        ));
        eprintln!(
            "[line {}] Error at '{}': {}.",
            self.lines + 1,
            token,
            message
        );
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_lexing() {
        use crate::lexer::Lexing;
        use crate::token::Token;
        let input = "var a = 10;";
        let mut lex = Lexing::new(input);
        let mut tokens = Vec::new();
        loop {
            let tok = lex.next();
            tokens.push(tok.clone());
            if tok == Token::Eof {
                break;
            }
        }
        assert_eq!(
            tokens,
            vec![
                Token::Var,
                Token::Identifier("a".to_string()),
                Token::Equal,
                Token::Number("10".to_string()),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }
}

use std::process::exit;

use crate::ast::{ExprType, Ident, Literal, Precedence, Progam, Stmt};
use crate::lexer::Lexing;
use crate::token::{self, Token};

pub struct Parser<'a> {
    lex: Lexing<'a>,
    current: Token,
    next: Token,
    prev: Token,
}

impl<'a> Parser<'a> {
    pub fn new(mut lex: Lexing<'a>) -> Parser {
        let prev = Token::Eof;
        let current = lex.next();
        let next = lex.next();
        Parser {
            lex,
            current,
            next,
            prev,
        }
    }

    pub fn parse(&mut self) -> Progam {
        let mut program: Progam = vec![];
        while self.current != Token::Eof {
            match self.parse_stmt() {
                Some(stmt) => {
                    program.push(stmt);
                }
                None => {
                    break;
                }
            };
        }
        program
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current.clone() {
            Token::Eof => {
                return None;
            }
            Token::Var => self.parse_var_stmt(),
            Token::LeftBrace => {
                self.next();
                let mut stmts: Progam = vec![];
                while self.current != Token::RightBrace {
                    match self.parse_stmt() {
                        Some(stmt) => {
                            stmts.push(stmt);
                        }
                        None => {
                            break;
                        }
                    }
                }
                if self.current != Token::RightBrace {
                    exit(65);
                }
                self.next();
                return Some(Stmt::Block(stmts));
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.current == Token::Semicolon {
                    self.next();
                }
                return Some(Stmt::Expr(expr));
            }
            None => {
                return None;
            }
        }
    }

    fn parse_var_stmt(&mut self) -> Option<Stmt> {
        self.next();
        let ident = match self.current.clone() {
            Token::Identifier(s) => {
                self.next();
                Ident(s)
            }
            _ => {
                return None;
            }
        };
        if self.current == Token::Semicolon {
            self.next();
            return Some(Stmt::Var(ident, ExprType::Literal(Literal::Nil)));
        }
        self.next();
        let expr = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::Semicolon {
            eprintln!(
                "self.current != Token::Semicolon Unexpected token: {:?}",
                self.current
            );
            exit(0);
        }
        self.next();
        Some(Stmt::Var(ident, expr))
    }

    fn current_token_precedence(&self) -> Precedence {
        self.token_precedence(self.current.clone())
    }

    // fn next_token_precedence(&self) -> Precedence {
    //     self.token_precedence(self.next.clone())
    // }

    fn token_precedence(&self, token: Token) -> Precedence {
        match token {
            Token::EqualEqual | Token::BangEqual | Token::Equal => Precedence::Equals,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => {
                Precedence::LessGreater
            }
            Token::Star | Token::Slash => Precedence::Star,
            Token::Plus | Token::Minus => Precedence::Plus,
            _ => Precedence::Lowest,
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<ExprType> {
        // println!("parse_expr: {:?} {:?}", self.current, precedence);
        // prefix
        let mut left = match self.current.clone() {
            Token::Bang | Token::Plus | Token::Minus => self.parse_prefix_expr(),
            Token::LeftParen => self.parse_grouped_expr(),
            Token::Number(_) => self.parse_number_literal(),
            Token::Identifier(ident) => {
                self.next();
                Some(ExprType::Ident(Ident(ident)))
            }
            Token::True => {
                self.next();
                Some(ExprType::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.next();
                Some(ExprType::Literal(Literal::Bool(false)))
            }
            Token::String(s) => {
                self.next();
                Some(ExprType::Literal(Literal::String(s)))
            }
            Token::Nil => {
                self.next();
                Some(ExprType::Literal(Literal::Nil))
            }
            Token::Print => {
                self.next();
                if self.current == Token::Semicolon {
                    self.lex.log_error(self.current.clone(), "Expect semicolon");
                    return None;
                }
                let expr = self.parse_expr(Precedence::Lowest);
                if self.current != Token::Semicolon {
                    self.lex.log_error(self.current.clone(), "Expect semicolon");
                    return None;
                }
                return Some(ExprType::PrintExpr(Box::new(expr.unwrap())));
            }
            _ => {
                self.lex
                    .log_error(self.current.clone(), "Expect expression");
                return None;
            }
        };
        // println!(
        //     "left: {:?} precedence:{:?} current_precedence:{:?}, token: {:?}, < {:?}",
        //     left,
        //     precedence,
        //     self.current_token_precedence(),
        //     self.current,
        //     self.current != Token::Semicolon && precedence < self.current_token_precedence()
        // );
        // infix
        while self.current != Token::Semicolon && precedence < self.current_token_precedence() {
            match self.current.clone() {
                Token::Star
                | Token::Slash
                | Token::Plus
                | Token::Minus
                | Token::EqualEqual
                | Token::Equal
                | Token::BangEqual
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual => {
                    left = self.parse_infix_expr(left.unwrap());
                }
                _ => return left,
            }
        }

        left
    }

    fn parse_grouped_expr(&mut self) -> Option<ExprType> {
        self.next();

        let expr = self.parse_expr(Precedence::Lowest);
        if self.current == Token::RightParen {
            self.next();
        } else {
            self.lex
                .log_error(self.current.clone(), "Expect expression");
            return None;
        }
        match expr {
            Some(expr) => Some(ExprType::GroupingExpr(Box::new(expr))),
            None => {
                self.lex
                    .log_error(self.current.clone(), "Expect expression");
                return None;
            }
        }
    }

    fn parse_prefix_expr(&mut self) -> Option<ExprType> {
        let op = self.current.clone();
        self.next();
        let right = self.parse_expr(Precedence::Prefix);
        if let Some(right) = right {
            return Some(ExprType::PrefixExpr(op, Box::new(right)));
        }
        self.lex
            .log_error(self.current.clone(), "Expect expression");
        return None;
    }

    fn parse_number_literal(&mut self) -> Option<ExprType> {
        return match self.current.clone() {
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                self.next();
                Some(ExprType::Literal(Literal::Number(num)))
            }
            _ => {
                panic!("Unexpected");
            }
        };
    }

    fn parse_infix_expr(&mut self, left: ExprType) -> Option<ExprType> {
        let op = self.current.clone();
        let mut precedence = self.current_token_precedence();
        self.next();
        if op == token::Token::Equal {
            precedence = self.current_token_precedence();
        }

        // println!("parse_infix_expr: {:?} {:?} {:?} {:?}", left, op, self.current, precedence);

        if let Some(right) = self.parse_expr(precedence) {
            return Some(ExprType::InfixExpr(Box::new(left), op, Box::new(right)));
        }
        self.lex
            .log_error(self.current.clone(), "Expect expression");
        return None;
    }

    fn next(&mut self) -> Token {
        let next = self.lex.next();
        self.prev = self.current.clone();
        self.current = self.next.clone();
        self.next = next;
        let tok = self.current.clone();
        match tok {
            Token::Comment(_) => self.next(),
            _ => tok,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.lex.has_errors()
    }

    // fn peek(&self) -> Token {
    //     self.next.clone()
    // }
}

#[cfg(test)]
mod test {

    use crate::ast::{ExprType, Ident, Literal, Stmt};
    use crate::lexer::Lexing;
    use crate::parser::Parser;
    use crate::token::Token;

    #[test]
    fn test_var() {
        let input = "var a = 10;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Var(
                Ident(String::from("a")),
                ExprType::Literal(Literal::Number(10.0))
            ),]
        );
    }

    #[test]
    fn test_nagetive_number() {
        let input = "var a = -10;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Var(
                Ident(String::from("a")),
                ExprType::PrefixExpr(
                    Token::Minus,
                    Box::new(ExprType::Literal(Literal::Number(10.0)))
                )
            ),]
        );
    }

    #[test]
    fn test_parser() {
        let input = "1 + 2 * 3;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        println!("lex: {:?}", lex.errors);
        let mut parse = Parser::new(lex);
        let program = parse.parse();
        assert_eq!(program.len(), 1);
        assert_eq!(
            program,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::Literal(Literal::Number(1.0))),
                Token::Plus,
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(2.0))),
                    Token::Star,
                    Box::new(ExprType::Literal(Literal::Number(3.0)))
                ))
            ))]
        )
    }

    #[test]
    fn test_grouped() {
        let input = "(24 * -74 / (61 * 77))".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        println!("{:?}", progam[0]);
    }

    #[test]
    fn test_arithmetic_operators_1() {
        let input = "16 * 38 / 58".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(16.0))),
                    Token::Star,
                    Box::new(ExprType::Literal(Literal::Number(38.0))),
                )),
                Token::Slash,
                Box::new(ExprType::Literal(Literal::Number(58.0)))
            ))]
        );
    }

    #[test]
    fn test_arithmetic_operators_2() {
        let input = "(11 * -77 / (98 * 67))".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        // assert_eq!(
        //     progam,
        //     vec![Stmt::Expr(ExprType::InfixExpr(
        //         Box::new(ExprType::InfixExpr(
        //             Box::new(ExprType::Literal(Literal::Number(11.0))),
        //             Token::Star,
        //             Box::new(ExprType::Literal(Literal::Number(38.0))),
        //         )),
        //         Token::Slash,
        //         Box::new(ExprType::Literal(Literal::Number(58.0)))
        //     ))]
        // );
    }

    #[test]
    fn test_double_minutes() {
        let input = "var a = 10; var b = 20; var c = a - -b;".to_string();
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 3);
        assert_eq!(
            progam,
            vec![
                Stmt::Var(
                    Ident(String::from("a")),
                    ExprType::Literal(Literal::Number(10.0))
                ),
                Stmt::Var(
                    Ident(String::from("b")),
                    ExprType::Literal(Literal::Number(20.0))
                ),
                Stmt::Var(
                    Ident(String::from("c")),
                    ExprType::InfixExpr(
                        Box::new(ExprType::Ident(Ident(String::from("a")))),
                        Token::Minus,
                        Box::new(ExprType::PrefixExpr(
                            Token::Minus,
                            Box::new(ExprType::Ident(Ident(String::from("b"))))
                        ))
                    )
                )
            ]
        );
    }

    #[test]
    fn test_bang_true() {
        let input = "!true";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::PrefixExpr(
                Token::Bang,
                Box::new(ExprType::Literal(Literal::Bool(true)))
            ))]
        );
    }

    #[test]
    fn test_grouped_string() {
        let input = "(\"foo\")";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::GroupingExpr(Box::new(
                ExprType::Literal(Literal::String(String::from("foo")))
            )))]
        );
    }

    #[test]
    fn test_grouped_nil() {
        let input = "(nil)";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::GroupingExpr(Box::new(
                ExprType::Literal(Literal::Nil)
            )))]
        );
    }

    #[test]
    fn test_arithmetic_operators_3() {
        let input = "52 + 80 - 94";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(52.0))),
                    Token::Plus,
                    Box::new(ExprType::Literal(Literal::Number(80.0))),
                )),
                Token::Minus,
                Box::new(ExprType::Literal(Literal::Number(94.0)))
            ))]
        );
    }

    #[test]
    fn test_arithmetic_issue_2() {
        let input = "(-43 + 95) * (68 * 80) / (55 + 75)";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::PrefixExpr(
                            Token::Minus,
                            Box::new(ExprType::Literal(Literal::Number(43.0)))
                        )),
                        Token::Plus,
                        Box::new(ExprType::Literal(Literal::Number(95.0))),
                    )),
                    Token::Star,
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::Literal(Literal::Number(68.0))),
                        Token::Star,
                        Box::new(ExprType::Literal(Literal::Number(80.0))),
                    )),
                )),
                Token::Slash,
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::InfixExpr(
                        Box::new(ExprType::Literal(Literal::Number(55.0))),
                        Token::Plus,
                        Box::new(ExprType::Literal(Literal::Number(75.0))),
                    )),
                    Token::Plus,
                    Box::new(ExprType::Literal(Literal::Number(75.0))),
                )),
            ))]
        );
    }

    #[test]
    fn test_comparison_operator() {
        let input = "83 < 99 < 115";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        assert_eq!(progam.len(), 1);
        assert_eq!(
            progam,
            vec![Stmt::Expr(ExprType::InfixExpr(
                Box::new(ExprType::InfixExpr(
                    Box::new(ExprType::Literal(Literal::Number(83.0))),
                    Token::Less,
                    Box::new(ExprType::Literal(Literal::Number(99.0))),
                )),
                Token::Less,
                Box::new(ExprType::Literal(Literal::Number(115.0)))
            ))]
        );
    }

    #[test]
    fn test_output_error() {
        let input = "(foo";
        let lex: Lexing<'_> = Lexing::new(&input);
        let mut parse = Parser::new(lex);
        let progam = parse.parse();
        for error in parse.lex.errors {
            eprintln!("{}", error);
        }
        assert_eq!(progam.len(), 0);
    }
}

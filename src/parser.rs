use std::process::exit;

use crate::ast::{ExprType, Ident, Literal, Precedence, Progam, Stmt};
use crate::lexer::Lexing;
use crate::token::Token;

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
                None => {}
            };
        }
        for stmt in &program {
            println!("{}", stmt);
        }
        program
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current.clone() {
            Token::Eof => {
                return None;
            }
            Token::Var => self.parse_var_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        match self.parse_expr(Precedence::Lowest) {
            Some(expr) => {
                if self.current != Token::Semicolon {
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
                eprintln!("Unexpected token: {:?}", self.current);
                exit(0);
            }
        };
        if self.current != Token::Equal {
            eprintln!("Unexpected token: {:?}", self.current);
            exit(0);
        }
        self.next();
        let expr = self.parse_expr(Precedence::Lowest).unwrap();
        if self.current != Token::Semicolon {
            eprintln!("Unexpected token: {:?}", self.current);
            exit(0);
        }
        self.next();
        Some(Stmt::Var(ident, expr))
    }

    fn next_token_precedence(&self) -> Precedence {
        self.token_precedence(self.next.clone())
    }

    fn current_token_precedence(&self) -> Precedence {
        self.token_precedence(self.current.clone())
    }

    fn token_precedence(&self, token: Token) -> Precedence {
        match token {
            Token::Star | Token::Slash => Precedence::Star,
            Token::Plus | Token::Minus => Precedence::Plus,
            _ => Precedence::Lowest,
        }
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<ExprType> {
        // prefix
        let mut left = match self.current.clone() {
            Token::Bang | Token::Plus | Token::Minus => self.parse_prefix_expr().unwrap(),
            Token::LeftParen => self.parse_grouped_expr().unwrap(),
            Token::Number(_) => self.parse_number_literal().unwrap(),
            _ => {
                self.next();
                return None;
            }
        };

        // println!(
        //     "left: {:?}, token: {:?}, precedence: {:?}, next: {:?}, next_precedence: {:?}",
        //     left,
        //     self.current,
        //     precedence,
        //     self.next,
        //     self.current_token_precedence()
        // );
        // infix
        while self.current != Token::Semicolon && precedence < self.current_token_precedence() {
            match self.current.clone() {
                Token::Star | Token::Slash | Token::Plus | Token::Minus => {
                    left = self.parse_infix_expr(left).unwrap();
                }
                _ => {
                    break;
                }
            }
        }
        println!("infix left: {:?}", left);

        Some(left)
    }

    fn parse_grouped_expr(&mut self) -> Option<ExprType> {
        self.next();

        let expr = self.parse_expr(Precedence::Lowest);
        println!("grouped expr: {:?}", expr);
        if self.next == Token::RightParen {
            self.next();
        } else {
            return None;
        }
        match expr {
            Some(expr) => Some(ExprType::GroupingExpr(Box::new(expr))),
            None => {
                return None;
            }
        }
    }

    fn parse_prefix_expr(&mut self) -> Option<ExprType> {
        let op = self.current.clone();
        self.next();
        let right = self.parse_expr(Precedence::Lowest);
        return Some(ExprType::PrefixExpr(op, Box::new(right.unwrap())));
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

        self.next();
        let precedence = self.next_token_precedence();

        let right = self.parse_expr(precedence).unwrap();
        return Some(ExprType::InfixExpr(Box::new(left), op, Box::new(right)));
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
}

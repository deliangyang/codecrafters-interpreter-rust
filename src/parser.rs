use crate::ast::{ExprType, Progam, Stmt};
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
        let prev = Token::EOF;
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
        while self.current != Token::EOF {
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
            Token::EOF => {
                return None;
            }
            _ => {
                let expr = self.parse_expr(1).unwrap();
                return Some(Stmt::Expr(expr));
            }
        }
    }

    fn parse_expr(&mut self, _: usize) -> Option<ExprType> {
        // prefix
        let mut left = match self.current.clone() {
            Token::Number(_) => self.parse_number_literal().unwrap(),
            Token::Bang | Token::Star | Token::Slash => self.parse_prefix_expr().unwrap(),
            _ => {
                return None;
            }
        };

        while self.current != Token::EOF {
            match self.current.clone() {
                Token::Star | Token::Slash => {
                    left = self.parse_infix_expr(left).unwrap();
                }
                _ => {
                    break;
                }
            }
        }

        println!("left: {:?}", left);
        Some(left)
    }

    fn parse_prefix_expr(&mut self) -> Option<ExprType> {
        let _tok = self.current.clone();
        self.next();
        let right = self.parse_expr(1);
        return match right {
            Some(ExprType::NumberLiteral(n)) => Some(ExprType::NumberLiteral(n)),
            Some(ExprType::UnaryExpr(_, _)) => right,
            Some(ExprType::BinaryExpr(_, _, _)) => right,
            _ => None,
        };
    }

    fn parse_number_literal(&mut self) -> Option<ExprType> {
        return match self.current.clone() {
            Token::Number(n) => {
                let num = n.parse::<f64>().unwrap();
                self.next();
                Some(ExprType::NumberLiteral(num))
            }
            _ => {
                panic!("Unexpected");
            }
        };
    }

    fn parse_infix_expr(&mut self, left: ExprType) -> Option<ExprType> {
        let op = self.current.clone();
        self.next();

        return match self.parse_expr(0) {
            Some(right) => match right {
                _ => {
                    Some(ExprType::BinaryExpr(Box::new(left), op, Box::new(right)))
                },
            },
            None => Some(left),
        }
    }

    #[allow(dead_code)]
    fn parse_literal_expr(&mut self) -> Option<ExprType> {
        match self.current.clone() {
            Token::EOF => {
                return None;
            }
            Token::TRUE => {
                self.next();
                return Some(ExprType::BoolExpr(true));
            }
            Token::FALSE => {
                self.next();
                return Some(ExprType::BoolExpr(false));
            }
            Token::NIL => {
                self.next();
                return Some(ExprType::NilLiteral);
            }
            Token::STRING(s) => {
                self.next();
                return Some(ExprType::StringLiteral(s));
            }
            Token::Bang => {
                self.next();
                let unary =
                    ExprType::UnaryExpr(self.prev.clone(), Box::new(self.parse_expr(1).unwrap()));
                return Some(unary);
            }
            Token::Minus => {
                self.next();
                let unary =
                    ExprType::UnaryExpr(self.prev.clone(), Box::new(self.parse_expr(1).unwrap()));
                return Some(unary);
            }
            Token::LeftParen => {
                let mut group = Vec::new();
                self.next();

                while self.current != Token::RightParen {
                    group.push(Box::new(self.parse_expr(1).unwrap()));
                }
                self.next();
                return Some(ExprType::GroupingExpr(group));
            }
            _ => {
                panic!("Unexpected token: {:?}", self.current);
            }
        }
    }

    // fn parse_expression(&mut self) -> Option<Box<ExprType>> {
    //     let expr = self.next_expr().unwrap();
    //     Some(expr)
    // }

    fn next(&mut self) -> Token {
        let next = self.lex.next();
        self.prev = self.current.clone();
        self.current = self.next.clone();
        self.next = next;
        let tok = self.current.clone();
        match tok {
            Token::Comment(_) => {
                self.next()
            }
            _ => tok
        }
    }

    pub fn has_errors(&self) -> bool {
        self.lex.has_errors()
    }

    // fn peek(&self) -> Token {
    //     self.next.clone()
    // }
}

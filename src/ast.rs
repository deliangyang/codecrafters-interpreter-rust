
use std::fmt::Display;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident (pub String);

pub enum Stmt {
    Expr(ExprType),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
        }
    }
}

pub type BlockStmt = Vec<Stmt>;

pub type Progam = BlockStmt;

#[derive(Debug)]
pub enum ExprType {
    StringLiteral(String),
    NumberLiteral(f64),
    NilLiteral,
    BoolExpr(bool),
    GroupingExpr(Vec<Box<ExprType>>),
    UnaryExpr(Token, Box<ExprType>),                // prefix unary parse
    BinaryExpr(Box<ExprType>, Token, Box<ExprType>),    // infix binary parse
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::StringLiteral(s) => write!(f, "{}", s),
            ExprType::NumberLiteral(n) => {
                let inum = (*n as i64) as f64;
                if *n > inum {
                    write!(f, "{}", n)
                } else {
                    write!(f, "{}.0", n)
                }
            }
            ExprType::NilLiteral => write!(f, "nil"),
            ExprType::BoolExpr(b) => write!(f, "{}", b),
            ExprType::GroupingExpr(g) => {
                let mut s = String::new();
                for e in g {
                    s.push_str(&format!("{}, ", e));
                }
                write!(f, "(group {})", s)
            }
            ExprType::UnaryExpr(t, e) => match *t {
                Token::Minus => write!(f, "(- {})", e),
                Token::Bang => write!(f, "(! {})", e),
                Token::Star => write!(f, "(* {})", e),
                Token::Slash => write!(f, "(/ {})", e),
                Token::AND => write!(f, "(+ {})", e),
                _ => write!(f, "({} {})", t, e),
            },
            ExprType::BinaryExpr(l, t, r) => match *t {
                Token::Star => write!(f, "(* {} {})", l, r),
                Token::Slash => write!(f, "(/ {} {})", l, r),
                Token::Minus => write!(f, "(- {} {})", l, r),
                Token::Plus => write!(f, "(+ {} {})", l, r),
                _ => write!(f, "({} {} {})", l, t, r),
            },
        }
    }
}

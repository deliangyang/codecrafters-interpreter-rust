use crate::token::Token;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Var(Ident, ExprType),
    Expr(ExprType),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Var(i, e) => write!(f, "var {} = {}", i.0, e),
        }
    }
}

pub type BlockStmt = Vec<Stmt>;

pub type Progam = BlockStmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Ident(Ident),
    Literal(Literal),
    GroupingExpr(Box<ExprType>),
    UnaryExpr(Token, Box<ExprType>), // prefix unary parse
    BinaryExpr(Box<ExprType>, Token, Box<ExprType>), // infix binary parse
    PrefixExpr(Token, Box<ExprType>),
    InfixExpr(Box<ExprType>, Token, Box<ExprType>),
}

impl Display for ExprType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprType::UnaryExpr(t, e) => match *t {
                Token::Minus => write!(f, "(- {})", e),
                Token::Bang => write!(f, "(! {})", e),
                Token::Star => write!(f, "(* {})", e),
                Token::Slash => write!(f, "(/ {})", e),
                Token::And => write!(f, "(+ {})", e),
                _ => write!(f, "({} {})", t, e),
            },
            ExprType::BinaryExpr(l, t, r) => match *t {
                Token::Star => write!(f, "(* {} {})", l, r),
                Token::Slash => write!(f, "(/ {} {})", l, r),
                Token::Minus => write!(f, "(- {} {})", l, r),
                Token::Plus => write!(f, "(+ {} {})", l, r),
                _ => write!(f, "({} {} {})", l, t, r),
            },
            ExprType::Ident(v) => write!(f, "{}", v.0),
            ExprType::Literal(literal) => {
                match literal {
                    Literal::Number(n) => {
                        let inum = (*n as i64) as f64;
                        if *n > inum {
                            write!(f, "{}", n)
                        } else {
                            write!(f, "{}.0", inum)
                        }
                    },
                    Literal::String(s) => write!(f, "{}", s),
                    Literal::Bool(b) => write!(f, "{}", b),
                    Literal::Nil => write!(f, "nil"),
                }
            },
            ExprType::GroupingExpr(expr) => {
                write!(f, "(group {})", expr)
            },
            ExprType::PrefixExpr(token, expr) => {
                let op = match token {
                    Token::Minus => "-",
                    Token::Bang => "!",
                    _ => panic!("Invalid prefix operator"),
                };
                write!(f, "({} {})", op, expr)
            },
            ExprType::InfixExpr(left, token, right) => {
                let op = match token {
                    Token::EqualEqual => "==",
                    Token::BangEqual => "!=",
                    Token::Less => "<",
                    Token::LessEqual => "<=",
                    Token::Greater => ">",
                    Token::GreaterEqual => ">=",
                    Token::Plus => "+",
                    Token::Minus => "-",
                    Token::Star => "*",
                    Token::Slash => "/",
                    _ => panic!("Invalid infix operator"),
                };
                write!(f, "({} {} {})", op, left, right)
            },
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Plus,        // +
    Star,        // *
    Prefix,      // -X or !X
    Call,        // myFunction(x)
    Index,       // array[index]
}

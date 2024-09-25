use crate::token::Token;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Ident(pub String);

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Blank,
    Var(Ident, ExprType),
    Expr(ExprType),
    Block(Vec<Stmt>),
    Return(ExprType),
    Function(Ident, Vec<Ident>, BlockStmt),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expr(e) => write!(f, "{}", e),
            Stmt::Var(i, e) => write!(f, "var {} = {}", i.0, e),
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    writeln!(f, "{}", stmt)?;
                }
                Ok(())
            },
            Stmt::Return(e) => write!(f, "return {}", e),
            Stmt::Blank => write!(f, ""),
            Stmt::Function(name, params, body) => {
                write!(f, "fun {}(", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") {{\n")?;
                for stmt in body {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            },
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
    PrintExpr(Box<ExprType>),
    If {
        condition: Box<ExprType>,
        elseif: Vec<(Box<ExprType>, BlockStmt)>,
        then_branch: BlockStmt,
        else_branch: BlockStmt,
    },
    Function {
        params: Vec<Ident>,
        body: BlockStmt,
    },
    Call {
        callee: Box<ExprType>,
        args: Vec<ExprType>,
    },
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
                    Token::Equal => "=",
                    _ => panic!("Invalid infix operator"),
                };
                write!(f, "({} {} {})", op, left, right)
            },
            ExprType::PrintExpr(expr) => {
                write!(f, "(print {})", expr)
            },
            ExprType::If { condition, elseif, then_branch, else_branch } => {
                write!(f, "if {} {{\n", condition)?;
                for stmt in then_branch {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")?;
                for (cond, block) in elseif {
                    write!(f, " else if {} {{\n", cond)?;
                    for stmt in block {
                        writeln!(f, "\t{}", stmt)?;
                    }
                    write!(f, "}}")?;
                }
                if !else_branch.is_empty() {
                    write!(f, " else {{\n")?;
                    for stmt in else_branch {
                        writeln!(f, "\t{}", stmt)?;
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            },
            ExprType::Function { params, body } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") {{\n")?;
                for stmt in body {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "\n}}\n")
            },
            ExprType::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
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

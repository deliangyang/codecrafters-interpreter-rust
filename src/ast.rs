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
    Switch(ExprType, Vec<Stmt>),
    Case(ExprType, BlockStmt),
    Default(BlockStmt),
    While(ExprType, BlockStmt),
    Import(String),
    ClassStmt {
        name: Ident,
        properties: Vec<Stmt>,
    },
    For {
        init: Box<Stmt>,
        conditions: Box<ExprType>,
        step: Box<Stmt>,
        block: BlockStmt,
    },
    ForIn {
        var: Box<Stmt>,
        iter: Box<ExprType>,
        block: BlockStmt,
    },
    ClassInit(Ident, Vec<ExprType>),
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
            }
            Stmt::Import(s) => write!(f, "import {:?}", s),
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
            }
            Stmt::Switch(e, cases) => {
                write!(f, "switch {} {{\n", e)?;
                for stmt in cases {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::Case(e, block) => {
                write!(f, "case {}: \n", e)?;
                for stmt in block {
                    writeln!(f, "\t{}", stmt)?;
                }
                Ok(())
            }
            Stmt::Default(block) => {
                write!(f, "default:\n")?;
                for stmt in block {
                    writeln!(f, "\t{}", stmt)?;
                }
                Ok(())
            }
            Stmt::While(expr, block) => {
                write!(f, "while ({}) {{\n", expr)?;
                for stmt in block {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::ClassStmt { name, properties } => {
                write!(f, "class {} {{\n", name)?;
                for stmt in properties {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::For {
                init,
                conditions,
                step,
                block,
            } => {
                write!(f, "for ({}; {}; {}) {{\n", init, conditions, step)?;
                for stmt in block {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::ClassInit(name, args) => {
                write!(f, "new {}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Stmt::ForIn { var, iter, block } => {
                write!(f, "for ({} in {}) {{\n", var, iter)?;
                for stmt in block {
                    writeln!(f, "\t{}", stmt)?;
                }
                write!(f, "}}")
            }
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
    Index(usize),
    Array(Vec<ExprType>),
    Hash(Vec<(ExprType, ExprType)>),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Ident(Ident),
    ThisExpr(Ident),
    Literal(Literal),
    GroupingExpr(Box<ExprType>),
    UnaryExpr(Token, Box<ExprType>), // prefix unary parse
    PrefixExpr(Token, Box<ExprType>),
    InfixExpr(Box<ExprType>, Token, Box<ExprType>),
    PrintExpr(Box<Vec<ExprType>>),
    IndexExpr(Box<ExprType>, Box<ExprType>),
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
    ClassInit {
        name: Ident,
        args: Vec<ExprType>,
    },
    ClassCall {
        callee: Ident,
        method: Ident,
        args: Vec<ExprType>,
    },
    ClassGet {
        callee: Ident,
        prop: Ident,
    },
    ThisCall {
        method: Ident,
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
            ExprType::Ident(v) => write!(f, "{}", v.0),
            ExprType::Literal(literal) => match literal {
                Literal::Number(n) => {
                    let inum = (*n as i64) as f64;
                    if *n > inum {
                        write!(f, "{}", n)
                    } else {
                        write!(f, "{}.0", inum)
                    }
                }
                Literal::Index(i) => write!(f, "{}", i),
                Literal::String(s) => write!(f, "{}", s),
                Literal::Bool(b) => write!(f, "{}", b),
                Literal::Nil => write!(f, "nil"),
                Literal::Array(arr) => {
                    write!(f, "[")?;
                    for (i, expr) in arr.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", expr)?;
                    }
                    write!(f, "]")
                }
                Literal::Hash(hash) => {
                    write!(f, "{{")?;
                    for (i, (key, value)) in hash.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}: {}", key, value)?;
                    }
                    write!(f, "}}")
                }
            },
            ExprType::GroupingExpr(expr) => {
                write!(f, "(group {})", expr)
            }
            ExprType::ThisCall { method, args } => {
                write!(f, "this.{}(", method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprType::PrefixExpr(token, expr) => {
                let op = match token {
                    Token::Minus => "-",
                    Token::Bang => "!",
                    _ => panic!("Invalid prefix operator"),
                };
                write!(f, "({} {})", op, expr)
            }
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
                    Token::And => "&&",
                    Token::Or => "||",
                    Token::Mod => "%",
                    Token::ModSelf => "%=",
                    Token::PlusSelf => "+=",
                    Token::MinusSelf => "-=",
                    Token::StarSelf => "*=",
                    Token::SlashSelf => "/=",
                    _ => panic!("Invalid infix operator {:?}", token),
                };
                write!(f, "({} {} {})", op, left, right)
            }
            ExprType::PrintExpr(expr) => {
                let expr = expr
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "(print {})", expr)
            }
            ExprType::If {
                condition,
                elseif,
                then_branch,
                else_branch,
            } => {
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
            }
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
            }
            ExprType::Call { callee, args } => {
                write!(f, "{}(", callee)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprType::ThisExpr(ident) => write!(f, "this.{}", ident.0),
            ExprType::ClassInit { name, args } => {
                write!(f, "new {}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprType::ClassCall {
                callee,
                method,
                args,
            } => {
                write!(f, "{}.{}(", callee, method)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ExprType::ClassGet { callee, prop } => write!(f, "{}.{}", callee, prop),
            ExprType::IndexExpr(left, right) => {
                write!(f, "{}[{}]", left, right)
            }
        }
    }
}

#[derive(PartialEq, PartialOrd, Debug, Clone)]
pub enum Precedence {
    Lowest,
    And,        // &&
    Equals,      // ==
    LessGreater, // > or <
    OpSelfSum,      // += -=
    OpSelfMul,      // *= /=
    Plus,        // +
    Star,        // *
    PlusPlus,    // ++ --
    Prefix,      // -X or !X
    Call,        // myFunction(x)
    Index,       // array[index] hash[index]
    Class,       // instance.property
}

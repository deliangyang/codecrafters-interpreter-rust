use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::ast;

pub type BuiltinFunc = fn(Vec<Object>) -> Object;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Builtin(i32, BuiltinFunc),
    Function(Vec<ast::Ident>, ast::BlockStmt),
    Class(String, Vec<ast::Stmt>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::Number(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "{}", s),
            Object::Builtin(c, func) => write!(f, "builtin function: {} {:?}", c, func),
            Object::Function(params, body) => {
                let mut params_str = String::new();
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        params_str.push_str(", ");
                    }
                    params_str.push_str(&param.0);
                }
                write!(f, "fn({}) {:?}", params_str, body)
            },
            Object::Class(name, properties) => {
                write!(f, "class {} {{\n", name)?;
                for prop in properties {
                    writeln!(f, "\t{}", prop)?;
                }
                write!(f, "}}")
            },
        }
    }
}
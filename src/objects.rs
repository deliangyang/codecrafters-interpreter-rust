use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::ast;

pub type BuiltinFunc = fn(Vec<Object>) -> Object;

#[derive(PartialEq, Clone, Debug)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    Index(usize),
    String(String),
    Array(Vec<Object>),
    ReturnValue(Box<Object>),
    Hash(Rc<RefCell<HashMap<Object, Object>>>),
    Builtin(String, i32, BuiltinFunc),
    Function(Vec<ast::Ident>, ast::BlockStmt),
    Class(String, Vec<ast::Stmt>),
    ClassInstance {
        name: String,
        fields: Rc<RefCell<HashMap<String, Object>>>,
        properties: Rc<RefCell<HashMap<String, Object>>>,
    },
}

impl Eq for Object {}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Object::Index(ref i) => i.hash(state),
            Object::Boolean(ref b) => b.hash(state),
            Object::String(ref s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::Number(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "{}", s),
            Object::Builtin(s, c, func) => write!(f, "builtin function: {} {} {:?}", s, c, func),
            Object::Function(params, body) => {
                let mut params_str = String::new();
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        params_str.push_str(", ");
                    }
                    params_str.push_str(&param.0);
                }
                write!(f, "fn({}) {:?}", params_str, body)
            }
            Object::ReturnValue(obj) => write!(f, "{}", obj),
            Object::Array(elements) => {
                let mut elements_str = String::new();
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        elements_str.push_str(", ");
                    }
                    elements_str.push_str(&elem.to_string());
                }
                write!(f, "[{}]", elements_str)
            }
            Object::Hash(hash) => {
                let mut hash_str = String::new();
                for (key, value) in hash.borrow().iter() {
                    hash_str.push_str(&format!("{}: {}, ", key, value));
                }
                write!(f, "{{{}}}", hash_str)
            }
            Object::Index(i) => write!(f, "{}", i),
            Object::Class(name, properties) => {
                write!(f, "class {} {{\n", name)?;
                for prop in properties {
                    writeln!(f, "\t{}", prop)?;
                }
                write!(f, "}}")
            }
            Object::ClassInstance {
                name,
                fields,
                properties,
            } => {
                write!(f, "instance of class {} {{\n", name)?;
                for (key, value) in fields.borrow().iter() {
                    writeln!(f, "\t{}: {}", key, value)?;
                }
                for (key, value) in properties.borrow().iter() {
                    writeln!(f, "\t{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

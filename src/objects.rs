use std::fmt::Display;

pub type BuiltinFunc = fn(Vec<Object>) -> Object;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
    Builtin(i32, BuiltinFunc),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::Number(n) => write!(f, "{}", n),
            Object::String(s) => write!(f, "{}", s),
            Object::Builtin(c, func) => write!(f, "builtin function: {} {:?}", c, func),
        }
    }
}
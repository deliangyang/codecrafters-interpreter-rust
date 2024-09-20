use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Nil => write!(f, "nil"),
            Object::Number(n) => write!(f, "{}", n),
        }
    }
}
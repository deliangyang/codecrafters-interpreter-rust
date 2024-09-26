use std::collections::HashMap;

use crate::objects::Object;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Object::Builtin(1, x_print));
    builtins.insert("len".to_string(), Object::Builtin(1, len));
    builtins
}

fn x_print(args: Vec<Object>) -> Object {
    for arg in args {
        print!("{}", arg);
    }
    Object::Nil
}

fn len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    match &args[0] {
        Object::String(s) => Object::Number(s.len() as f64),
        Object::Array(a) => Object::Number(a.len() as f64),
        _ => Object::Nil,
    }
}
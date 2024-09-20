use std::collections::HashMap;

use crate::objects::Object;

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtins.insert("print".to_string(), Object::Builtin(1, x_print));
    builtins
}

fn x_print(args: Vec<Object>) -> Object {
    for arg in args {
        print!("{}", arg);
    }
    Object::Nil
}
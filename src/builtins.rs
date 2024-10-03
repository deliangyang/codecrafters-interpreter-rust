use std::collections::HashMap;

use crate::objects::Object;

pub struct Builtins {
    pub builtins: HashMap<String, Object>,
    sorted: HashMap<String, usize>,
    indexs: Vec<String>,
}

impl Builtins {
    
    pub fn new() -> Self {
        let builtins = new_builtins();
        let mut sorted = HashMap::new();
        let mut keys = builtins.keys().collect::<Vec<&String>>();
        let mut indexs = vec![];
        keys.sort();
        for (i, k) in keys.iter().enumerate() {
            sorted.insert(k.to_string(), i);
            indexs.push(k.to_string());
        }
        Builtins {
            builtins: builtins,
            sorted: sorted,
            indexs: indexs,
        }
    }

    pub fn get_index(&self, name: &str) -> Option<usize> {
        self.sorted.get(name).cloned()
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        self.builtins.get(name).cloned()
    }

    pub fn get_by_index(&self, index: usize) -> Option<Object> {
        if index < self.indexs.len() {
            self.builtins.get(&self.indexs[index]).cloned()
        } else {
            None
        }
    }

    pub fn get_name(&self, index: usize) -> Option<String> {
        if index < self.indexs.len() {
            Some(self.indexs[index].clone())
        } else {
            None
        }
    }
}

macro_rules! builtin_insert {
    ($builtins:ident, $name:expr, $count: expr, $func:expr) => {
        $builtins.insert(
            $name.to_string(),
            Object::Builtin($name.to_string(), $count, $func),
        );
    };
    () => {};
}

pub fn new_builtins() -> HashMap<String, Object> {
    let mut builtins = HashMap::new();
    builtin_insert!(builtins, "print", 1, x_print);
    builtin_insert!(builtins, "len", 1, len);
    builtin_insert!(builtins, "start_with", 2, start_with);
    builtin_insert!(builtins, "println", -1, x_println);
    builtin_insert!(builtins, "substr", 3, substr);
    builtin_insert!(builtins, "typeis", 1, typeis);
    builtin_insert!(builtins, "append", -1, append);
    builtin_insert!(builtins, "intval", 1, intval);
    builtin_insert!(builtins, "is_str", 1, is_str);
    builtin_insert!(builtins, "is_number", 1, is_number);
    builtin_insert!(builtins, "strval", 1, strval);
    builtin_insert!(builtins, "trim", 1, trim);
    builtin_insert!(builtins, "type", 1, x_type);
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
        Object::Hash(h) => Object::Number(h.borrow().len() as f64),
        _ => Object::Nil,
    }
}

fn start_with(args: Vec<Object>) -> Object {
    if args.len() != 2 {
        return Object::Nil;
    }
    match (&args[0], &args[1]) {
        (Object::String(s), Object::String(p)) => Object::Boolean(s.starts_with(p)),
        _ => Object::Nil,
    }
}

fn x_println(args: Vec<Object>) -> Object {
    let str = args
        .iter()
        .map(|x| format!("{}", x))
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", str);
    Object::Nil
}

fn substr(args: Vec<Object>) -> Object {
    if args.len() != 3 {
        return Object::Nil;
    }
    match (&args[0], &args[1], &args[2]) {
        (Object::String(s), Object::Number(start), Object::Number(end)) => {
            let start = start.floor() as usize;
            let end = end.floor() as usize;
            if start > s.len() || end > s.len() {
                return Object::Nil;
            }
            if end <= 0 {
                return Object::String(s[start..].to_string());
            }
            Object::String(s[start..end].to_string())
        }
        (Object::String(s), Object::Index(start), Object::Index(end)) => {
            if *start > s.len() || *end > s.len() {
                return Object::Nil;
            }
            Object::String(s[*start..*end].to_string())
        }
        _ => Object::Nil,
    }
}

fn typeis(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    match &args[0] {
        // Object::Hash(v) => Object::String(format!("{:?}", v.borrow())),
        _ => Object::String(format!("{:?}", args[0])),
    }
}

fn append(args: Vec<Object>) -> Object {
    if args.len() < 2 {
        return args[0].clone();
    }
    let mut arr = match &args[0] {
        Object::Array(a) => a.clone(),
        _ => vec![],
    };
    arr.extend(args[1..].iter().cloned());
    Object::Array(arr)
}

fn intval(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    match &args[0] {
        Object::Number(n) => Object::Index(*n as usize),
        Object::String(s) => {
            let s = s.trim();
            match s.parse::<usize>() {
                Ok(n) => Object::Index(n),
                Err(_) => Object::Nil,
            }
        }
        _ => Object::Nil,
    }
}

fn is_str(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    if let Object::String(s) = &args[0] {
        let mut iter = s.chars();
        let c = iter.next().unwrap();
        Object::Boolean(c == '"' || c == '\'')
    } else {
        Object::Boolean(false)
    }
}

fn is_number(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    let n = intval(args);
    if let Object::Index(_) = n {
        return Object::Boolean(true);
    }
    return Object::Boolean(false);
}

fn strval(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    Object::String(format!("{}", args[0]))
}

fn trim(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    if let Object::String(s) = &args[0] {
        Object::String(s.trim().to_string())
    } else {
        Object::Nil
    }
}

fn x_type(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Nil;
    }
    match &args[0] {
        Object::String(_) => Object::String("string".to_string()),
        Object::Number(_) => Object::String("number".to_string()),
        Object::Boolean(_) => Object::String("boolean".to_string()),
        Object::Nil => Object::String("nil".to_string()),
        Object::Array(_) => Object::String("array".to_string()),
        Object::Hash(_) => Object::String("object".to_string()),
        Object::Index(_) => Object::String("number".to_string()),
        Object::Builtin(_, _, _) => Object::String("builtin".to_string()),
        Object::Function(_, _) => Object::String("function".to_string()),
        Object::ReturnValue(_) => Object::String("return_value".to_string()),
        Object::Class(_, _) => Object::String("class".to_string()),
        Object::ClassInstance { .. } => Object::String("class_instance".to_string()),
    }
}

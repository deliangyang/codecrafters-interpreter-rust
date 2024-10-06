use std::{cell::RefCell, collections::HashMap, process::exit, rc::Rc};

use crate::{
    ast::{ExprType, Literal, Program, Stmt},
    builtins,
    envs::Env,
    objects::Object,
    token::Token,
};

pub struct Evaluator {
    output: bool,
    pub ast: Program,
    builtins: HashMap<String, Object>,
    envs: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new(ast: Program, output: bool) -> Self {
        Self {
            ast: ast,
            builtins: builtins::new_builtins(),
            output: output,
            envs: Rc::new(RefCell::new(Env::new())),
        }
    }

    pub fn evaluate(&mut self) {
        for stmt in self.ast.clone() {
            self.evaluate_stmt(&stmt);
        }
    }

    fn evaluate_stmt(&mut self, stmt: &Stmt) -> Option<Object> {
        match stmt {
            Stmt::Var(ident, expr) => {
                let name = ident.0.clone();
                let object = self.evaluate_expr(expr).unwrap();
                self.envs.borrow_mut().set_store(name, &object);
            }
            Stmt::Expr(expr) => {
                let object = self.evaluate_expr(expr);
                if let Some(object) = object {
                    if let Object::ReturnValue(obj) = object {
                        return Some(*obj);
                    }
                }
            }
            Stmt::Block(stmts) => {
                let current_env = Rc::clone(&self.envs);
                let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                self.envs = Rc::new(RefCell::new(pre_envs));
                for stmt in stmts {
                    match stmt {
                        Stmt::Return(expr) => {
                            let object = self.evaluate_expr(expr).unwrap();
                            self.envs = current_env;
                            return Some(object);
                        }
                        _ => {
                            let result = self.evaluate_stmt(stmt);
                            if result.is_some() {
                                self.envs = current_env;
                                return result;
                            }
                        }
                    }
                }
                self.envs = current_env;
            }
            Stmt::Return(expr) => {
                let object = self.evaluate_expr(expr).unwrap();
                if self.output {
                    println!("{}", object);
                }
                return Some(object);
            }
            Stmt::Function(ident, args, body) => {
                let name = ident.0.clone();
                let object = Object::Function(args.clone(), body.clone());
                self.envs.borrow_mut().set_store(name, &object);
            }
            Stmt::Blank => {}
            Stmt::Switch(expr, cases) => {
                let result = self.evaluate_expr(expr).unwrap();
                for stmt in cases {
                    match stmt {
                        Stmt::Case(expr, block) => {
                            let case = self.evaluate_expr(expr).unwrap();
                            if case == result {
                                for stmt in block {
                                    let result = self.evaluate_stmt(stmt);
                                    if result.is_some() {
                                        return result;
                                    }
                                }
                                break;
                            }
                        }
                        Stmt::Default(block) => {
                            for stmt in block {
                                let result = self.evaluate_stmt(stmt);
                                if result.is_some() {
                                    return result;
                                }
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            Stmt::While(expr, block) => {
                let mut condition = true;
                let current_env = Rc::clone(&self.envs);
                let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                self.envs = Rc::new(RefCell::new(pre_envs));
                while condition {
                    let result = self.evaluate_expr(expr).unwrap();
                    if let Object::Boolean(result) = result {
                        if result {
                            for stmt in block {
                                let result = self.evaluate_stmt(stmt);
                                if result.is_some() {
                                    self.envs = current_env;
                                    return result;
                                }
                            }
                        } else {
                            condition = false;
                        }
                    }
                }
                self.envs = current_env;
            }
            Stmt::ClassStmt { name, properties } => {
                let object = Object::Class(name.to_string().clone(), properties.clone());
                self.envs
                    .borrow_mut()
                    .set(name.to_string().clone(), &object);
            }
            Stmt::For {
                init,
                conditions,
                step,
                block,
            } => self.evaluate_for(init, conditions, step, block),
            Stmt::ForIn { var, iter, block } => {
                let iter = self.evaluate_expr(iter).unwrap();
                let ident = match var.as_ref() {
                    Stmt::Var(ident, _) => ident,
                    _ => unimplemented!(),
                };
                if let Object::Hash(ref hash) = iter {
                    for (key, _) in hash.borrow().iter() {
                        let current_env = Rc::clone(&self.envs);
                        let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                        self.envs = Rc::new(RefCell::new(pre_envs));
                        self.envs.borrow_mut().set(ident.0.clone(), &key);
                        for stmt in block {
                            let result = self.evaluate_stmt(stmt);
                            if result.is_some() {
                                self.envs = current_env;
                                return result;
                            }
                        }
                        self.envs = current_env;
                    }
                }
            }
            Stmt::Import(_) => {}
            Stmt::Assert { condition, message } => {
                let result = self.evaluate_expr(condition).unwrap();
                if let Object::Boolean(result) = result {
                    if !result {
                        println!("Assertion failed: {} {}", condition, message);
                        exit(70);
                    }
                }
            }
            Stmt::Assign(ident, right) => match ident {
                ExprType::Ident(ident) => {
                    let name = ident.0.clone();
                    let object = self.evaluate_expr(right).unwrap();
                    self.envs.borrow_mut().set(name, &object);
                    return Some(object);
                }
                ExprType::IndexExpr(ident, expr) => {
                    if let ExprType::Ident(ident) = ident.as_ref() {
                        let hash = self.envs.borrow_mut().get(ident.0.clone());
                        if hash.is_none() {
                            panic!("not found {:?}", ident);
                        }
                        let hash_object = hash.unwrap();
                        if let Object::Hash(ref hash) = hash_object {
                            let index = self.evaluate_expr(expr).unwrap();
                            let object = self.evaluate_expr(right).unwrap();
                            hash.borrow_mut().insert(index, object.clone());
                            self.envs
                                .borrow_mut()
                                .set(ident.0.clone(), &Object::Hash(hash.clone()));
                            return Some(object);
                        }
                    }
                }
                ExprType::ThisExpr(ident) => {
                    let object = self.envs.borrow_mut().get_current_class().unwrap();
                    if let Object::ClassInstance {
                        name: _,
                        ref fields,
                        properties: _,
                    } = object.clone()
                    {
                        let object = self.evaluate_expr(right).unwrap();
                        fields.borrow_mut().insert(ident.0.clone(), object.clone());
                        return Some(object);
                    }
                    return Some(object);
                }
                _ => unimplemented!("not found {:?}", ident),
            },
            _ => unimplemented!(),
        }
        None
    }

    fn evaluate_for(&mut self, init: &Stmt, conditions: &ExprType, step: &Stmt, block: &Vec<Stmt>) {
        self.evaluate_stmt(init);
        let mut condition = true;
        while condition {
            let result = self.evaluate_expr(conditions).unwrap();
            if let Object::Boolean(result) = result {
                if result {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));
                    for stmt in block {
                        self.evaluate_stmt(stmt);
                    }
                    self.envs = current_env;
                    self.evaluate_stmt(step);
                } else {
                    condition = false;
                }
            }
        }
    }

    fn evaluate_expr(&mut self, expr: &ExprType) -> Option<Object> {
        match expr {
            ExprType::Literal(lit) => match lit {
                Literal::Number(n) => Some(Object::Number(*n)),
                Literal::Bool(v) => Some(Object::Boolean(*v)),
                Literal::Nil => Some(Object::Nil),
                Literal::String(s) => Some(Object::String(s.clone())),
                Literal::Array(arr) => {
                    let mut elements = Vec::new();
                    for elem in arr {
                        elements.push(self.evaluate_expr(elem).unwrap());
                    }
                    Some(Object::Array(elements))
                }
                Literal::Hash(hash) => {
                    let mut hash_map = HashMap::new();
                    for (key, value) in hash {
                        let key = self.evaluate_expr(key).unwrap();
                        let value = self.evaluate_expr(value).unwrap();
                        hash_map.insert(key, value);
                    }
                    Some(Object::Hash(Rc::new(RefCell::new(hash_map))))
                }
                Literal::Index(index) => Some(Object::Index(*index)),
            },
            ExprType::Ident(v) => {
                if let Some(builtin) = self.builtins.get(&v.0) {
                    return Some(builtin.clone());
                } else if let Some(env) = self.envs.borrow_mut().get(v.0.clone()) {
                    return Some(env.clone());
                }
                println!("Undefined variable '{}'.", v.0);
                exit(70);
            }
            ExprType::GroupingExpr(expr) => self.evaluate_expr(expr),
            ExprType::PrefixExpr(op, expr) => {
                let expr = self.evaluate_expr(expr).unwrap();
                match op {
                    Token::Minus => {
                        if let Object::Number(expr) = expr {
                            return Some(Object::Number(-expr));
                        }
                        exit(70);
                    }
                    Token::Bang => {
                        if let Object::Boolean(expr) = expr {
                            return Some(Object::Boolean(!expr));
                        } else if let Object::Nil = expr {
                            return Some(Object::Boolean(true));
                        } else if let Object::Number(expr) = expr {
                            return Some(Object::Boolean(expr == 0.0));
                        } else if let Object::String(expr) = expr {
                            return Some(Object::Boolean(expr.is_empty()));
                        } else if let Object::ReturnValue(expr) = expr {
                            match *expr {
                                Object::Boolean(expr) => return Some(Object::Boolean(!expr)),
                                Object::Nil => return Some(Object::Boolean(true)),
                                Object::Number(expr) => return Some(Object::Boolean(expr == 0.0)),
                                Object::String(expr) => {
                                    return Some(Object::Boolean(expr.is_empty()))
                                }
                                _ => unimplemented!(),
                            }
                        }
                        return None;
                    }
                    _ => unimplemented!(),
                }
            }
            ExprType::InfixExpr(left, op, right) => {
                if let Token::Equal = op {
                    match left.as_ref() {
                        ExprType::Ident(ident) => {
                            let name = ident.0.clone();
                            let object = self.evaluate_expr(right).unwrap();
                            self.envs.borrow_mut().set(name, &object);
                            return Some(object);
                        }
                        ExprType::IndexExpr(ident, expr) => {
                            if let ExprType::Ident(ident) = ident.as_ref() {
                                let hash = self.envs.borrow_mut().get(ident.0.clone());
                                if hash.is_none() {
                                    panic!("not found {:?}", ident);
                                }
                                let hash_object = hash.unwrap();
                                if let Object::Hash(ref hash) = hash_object {
                                    let index = self.evaluate_expr(expr).unwrap();
                                    let object = self.evaluate_expr(right).unwrap();
                                    hash.borrow_mut().insert(index, object.clone());
                                    self.envs
                                        .borrow_mut()
                                        .set(ident.0.clone(), &Object::Hash(hash.clone()));
                                    return Some(object);
                                }
                            }
                        }
                        ExprType::ThisExpr(ident) => {
                            let object = self.envs.borrow_mut().get_current_class().unwrap();
                            if let Object::ClassInstance {
                                name: _,
                                ref fields,
                                properties: _,
                            } = object.clone()
                            {
                                let object = self.evaluate_expr(right).unwrap();
                                fields.borrow_mut().insert(ident.0.clone(), object.clone());
                                return Some(object);
                            }
                            return Some(object);
                        }
                        _ => {}
                    }
                }

                match &op {
                    Token::MinusSelf => {
                        if let ExprType::Ident(ident) = *left.clone() {
                            let object = self.evaluate_expr(right).unwrap();
                            let object = match object {
                                Object::Number(n) => n,
                                _ => unimplemented!(),
                            };
                            let object = match self.envs.borrow_mut().get(ident.0.clone()) {
                                Some(Object::Number(n)) => Object::Number(n - object.clone()),
                                _ => unimplemented!(),
                            };
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        return Some(Object::Nil);
                    }
                    Token::PlusSelf => {
                        if let ExprType::Ident(ident) = *left.clone() {
                            let object = self.evaluate_expr(right).unwrap();
                            let object = match object {
                                Object::Number(n) => n,
                                _ => unimplemented!(),
                            };
                            let object = match self.envs.borrow_mut().get(ident.0.clone()) {
                                Some(Object::Number(n)) => Object::Number(n + object.clone()),
                                _ => unimplemented!(),
                            };
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        return Some(Object::Nil);
                    }
                    Token::StarSelf => {
                        if let ExprType::Ident(ident) = *left.clone() {
                            let object = self.evaluate_expr(right).unwrap();
                            let object = match object {
                                Object::Number(n) => n,
                                _ => unimplemented!(),
                            };
                            let object = match self.envs.borrow_mut().get(ident.0.clone()) {
                                Some(Object::Number(n)) => Object::Number(n * object.clone()),
                                _ => unimplemented!(),
                            };
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        return Some(Object::Nil);
                    }
                    Token::SlashSelf => {
                        if let ExprType::Ident(ident) = *left.clone() {
                            let object = self.evaluate_expr(right).unwrap();
                            let object = match object {
                                Object::Number(n) => n,
                                _ => unimplemented!(),
                            };
                            let object = match self.envs.borrow_mut().get(ident.0.clone()) {
                                Some(Object::Number(n)) => Object::Number(n / object.clone()),
                                _ => unimplemented!(),
                            };
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        return Some(Object::Nil);
                    }
                    Token::ModSelf => {
                        if let ExprType::Ident(ident) = *left.clone() {
                            let object = self.evaluate_expr(right).unwrap();
                            let object = match object {
                                Object::Number(n) => n,
                                _ => unimplemented!(),
                            };
                            let object = match self.envs.borrow_mut().get(ident.0.clone()) {
                                Some(Object::Number(n)) => Object::Number(n % object.clone()),
                                _ => unimplemented!(),
                            };
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        return Some(Object::Nil);
                    }
                    _ => {}
                }

                let left = self.evaluate_expr(left);
                let right = self.evaluate_expr(right);
                match op {
                    Token::EqualEqual => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left == right));
                            }
                        } else if let Object::Boolean(left) = left.clone().unwrap() {
                            if let Object::Boolean(right) = right.unwrap() {
                                return Some(Object::Boolean(left == right));
                            }
                        } else if let Object::String(left) = left.clone().unwrap() {
                            if let Object::String(right) = right.unwrap() {
                                return Some(Object::Boolean(left == right));
                            }
                        } else if let Object::Nil = left.clone().unwrap() {
                            if let Object::Nil = right.unwrap() {
                                return Some(Object::Boolean(true));
                            }
                        }
                        return Some(Object::Boolean(false));
                    }
                    Token::BangEqual => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left != right));
                            }
                        } else if let Object::Boolean(left) = left.clone().unwrap() {
                            if let Object::Boolean(right) = right.unwrap() {
                                return Some(Object::Boolean(left != right));
                            }
                        } else if let Object::String(left) = left.clone().unwrap() {
                            if let Object::String(right) = right.unwrap() {
                                return Some(Object::Boolean(left != right));
                            }
                        }
                        return Some(Object::Boolean(false));
                    }
                    Token::Less => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left < right));
                            }
                        }
                        exit(70);
                    }
                    Token::LessEqual => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left <= right));
                            }
                        }
                        exit(70);
                    }
                    Token::Greater => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left > right));
                            }
                        }
                        exit(70);
                    }
                    Token::GreaterEqual => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left >= right));
                            }
                        }
                        exit(70);
                    }
                    Token::Star => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Number(left * right));
                            }
                        }
                        exit(70);
                    }
                    Token::Slash => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Number(left / right));
                            }
                        }
                        exit(70);
                    }
                    Token::Minus => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Number(left - right));
                            }
                        }
                        exit(70);
                    }
                    Token::Plus => {
                        if let Object::Number(left) = left.clone().unwrap() {
                            if let Object::Number(right) = right.clone().unwrap() {
                                return Some(Object::Number(left + right));
                            }
                        } else if let Object::String(left) = left.clone().unwrap() {
                            if let Object::String(right) = right.clone().unwrap() {
                                return Some(Object::String(left + &right));
                            }
                        }
                        panic!("token plus not found {:?} {:?}", left, right);
                    }
                    Token::And => {
                        if let Object::Boolean(left) = left.unwrap() {
                            if let Object::Boolean(right) = right.unwrap() {
                                return Some(Object::Boolean(left && right));
                            }
                        }
                        exit(70);
                    }
                    Token::Or => {
                        if let Object::Boolean(left) = left.unwrap() {
                            if let Object::Boolean(right) = right.unwrap() {
                                return Some(Object::Boolean(left || right));
                            }
                        }
                        exit(70);
                    }
                    _ => {
                        println!("not found {:?} {:?} {:?}", left, op, right);
                        unimplemented!()
                    }
                }
            }
            ExprType::PrintExpr(expr) => {
                for expr in expr.iter() {
                    let object = self.evaluate_expr(expr).unwrap();
                    print!("{}", object);
                }
                return Some(Object::Nil);
            }
            ExprType::If {
                condition,
                elseif,
                then_branch,
                else_branch,
            } => {
                let condition = self.evaluate_expr(condition).unwrap();
                if let Object::Boolean(condition) = condition {
                    if condition {
                        for stmt in then_branch {
                            match stmt {
                                Stmt::Return(expr) => {
                                    let object = self.evaluate_expr(expr).unwrap();
                                    return Some(Object::ReturnValue(Box::new(object)));
                                }
                                _ => {
                                    let result = self.evaluate_stmt(stmt);
                                    if result.is_some() {
                                        return result;
                                    }
                                }
                            }
                        }
                    } else {
                        let mut condition = false;
                        for (cond, block) in elseif {
                            let cond = self.evaluate_expr(cond).unwrap();
                            if let Object::Boolean(cond) = cond {
                                if cond {
                                    for stmt in block {
                                        match stmt {
                                            Stmt::Return(expr) => {
                                                let object = self.evaluate_expr(expr).unwrap();
                                                return Some(Object::ReturnValue(Box::new(object)));
                                            }
                                            _ => {
                                                let result = self.evaluate_stmt(stmt);
                                                if result.is_some() {
                                                    return result;
                                                }
                                            }
                                        }
                                    }
                                    condition = true;
                                    break;
                                }
                            }
                        }
                        if !condition {
                            for stmt in else_branch {
                                match stmt {
                                    Stmt::Return(expr) => {
                                        let object = self.evaluate_expr(expr).unwrap();
                                        return Some(Object::ReturnValue(Box::new(object)));
                                    }
                                    _ => {
                                        if let Some(result) = self.evaluate_stmt(stmt) {
                                            return Some(result);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Some(Object::Nil);
            }
            ExprType::Function { params, body } => {
                return Some(Object::Function(params.clone(), body.clone()));
            }
            ExprType::ThisCall { method, args } => {
                let class = self.envs.borrow_mut().get_current_class();
                if class.is_none() {
                    panic!("not found this class call context: {:?}", method);
                }
                let class = class.unwrap();
                if let Object::ClassInstance {
                    name: _,
                    fields: _,
                    ref properties,
                } = class.clone()
                {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));

                    if let Object::Function(params, stmts) =
                        properties.borrow().get(&method.0).unwrap()
                    {
                        for (i, param) in params.iter().enumerate() {
                            let arg = self.evaluate_expr(&args[i]).unwrap();
                            self.envs.borrow_mut().set_store(param.0.clone(), &arg);
                        }
                        for stmt in stmts {
                            let result = self.evaluate_stmt(stmt);
                            if result.is_some() {
                                self.envs = current_env;
                                return result;
                            }
                        }
                    }
                    self.envs = current_env;
                }
                None
            }
            ExprType::Call { callee, args } => {
                let callee = self.evaluate_expr(&callee);
                match callee {
                    Some(Object::Builtin(name, argc, fun)) => {
                        let mut args_vec = Vec::new();
                        for arg in args {
                            let result = self.evaluate_expr(arg);
                            if result.is_some() {
                                args_vec.push(result.unwrap());
                            }
                        }
                        let real_argc = args_vec.len() as i32;
                        if argc != -1 && real_argc != argc {
                            println!(
                                "fun {}: Expected {} arguments but got {}.",
                                name, argc, real_argc
                            );
                            exit(70);
                        }
                        return Some(fun(args_vec));
                    }
                    Some(Object::Function(ident, stmts)) => {
                        let current_env = Rc::clone(&self.envs);
                        let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                        self.envs = Rc::new(RefCell::new(pre_envs));
                        let mut values = HashMap::new();
                        for (i, param) in ident.iter().enumerate() {
                            let arg: Object = self.evaluate_expr(&args[i]).unwrap();
                            values.insert(param.0.clone(), arg);
                        }
                        for (key, value) in values {
                            self.envs.borrow_mut().set_store(key, &value);
                        }

                        for stmt in stmts {
                            let block_result = self.evaluate_stmt(&stmt);
                            if block_result.is_some() {
                                self.envs = current_env;
                                return block_result;
                            }
                        }
                        self.envs = current_env;
                    }
                    _ => unimplemented!("not found {:?}({:?})", callee, args),
                }

                None
            }
            ExprType::ClassInit { name, args } => {
                // println!("{:?} {:?}", name, args);
                let class_name = name.clone().to_string();
                let class = self.envs.borrow_mut().get(class_name).unwrap();

                let mut props = HashMap::new();
                let mut fields = HashMap::new();
                if let Object::Class(_, properties) = class.clone() {
                    for property in properties.iter() {
                        match property {
                            Stmt::Var(ident, expr) => {
                                let name = ident.0.clone();
                                let object = self.evaluate_expr(&expr).unwrap();
                                fields.insert(name, object);
                            }
                            Stmt::Function(ident, params, body) => {
                                let name = ident.0.clone();
                                let object = Object::Function(params.clone(), body.clone());
                                props.insert(name, object);
                            }
                            _ => unimplemented!(),
                        };
                    }
                    let instance = Object::ClassInstance {
                        name: name.clone().to_string(),
                        fields: Rc::new(RefCell::new(fields)),
                        properties: Rc::new(RefCell::new(props.clone())),
                    };
                    if let Some(init_func) = props.clone().get("init") {
                        let current_env = Rc::clone(&self.envs);
                        let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                        self.envs = Rc::new(RefCell::new(pre_envs));
                        if let Object::Function(params, body) = init_func {
                            for (i, arg) in args.iter().enumerate() {
                                let arg: Object = self.evaluate_expr(&arg).unwrap();
                                let ident = params[i].0.clone();
                                self.envs.borrow_mut().set(ident, &arg);
                            }
                            if args.len() == 0 {
                                for param in params {
                                    self.envs.borrow_mut().set(param.0.clone(), &Object::Nil);
                                }
                            }
                            self.envs.borrow_mut().set_current_class(instance.clone());
                            for stmt in body {
                                self.evaluate_stmt(stmt);
                            }
                            self.envs.borrow_mut().reset_current_class();
                        }
                        self.envs = current_env;
                    }
                    return Some(instance);
                }

                return Some(Object::Nil);
            }
            ExprType::ClassCall {
                callee,
                method,
                args,
            } => {
                let class = self.envs.borrow_mut().get(callee.to_string()).unwrap();
                if let Object::ClassInstance {
                    name: _,
                    fields: _,
                    ref properties,
                } = class.clone()
                {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));

                    self.envs.borrow_mut().set_current_class(class.clone());

                    if let Object::Function(params, stmts) =
                        properties.borrow().get(&method.0).unwrap()
                    {
                        for (i, param) in args.iter().enumerate() {
                            let arg: Object = self.evaluate_expr(&param).unwrap();
                            self.envs.borrow_mut().set_store(params[i].0.clone(), &arg);
                        }
                        for stmt in stmts {
                            let result = self.evaluate_stmt(stmt);
                            if result.is_some() {
                                self.envs = current_env;
                                self.envs.borrow_mut().reset_current_class();
                                return result;
                            }
                        }
                    }
                    self.envs = current_env;
                    self.envs.borrow_mut().reset_current_class();
                }
                return Some(Object::Nil);
            }
            ExprType::ThisExpr(ident) => {
                let object = self.envs.borrow_mut().get_current_class();
                if object.is_none() {
                    panic!("not found this class context: {:?}", ident);
                }
                let object = object.unwrap();
                if let Object::ClassInstance {
                    name: _,
                    ref fields,
                    properties: _,
                } = object.clone()
                {
                    if let Some(object) = fields.borrow().get(&ident.0) {
                        return Some(object.clone());
                    }
                    return Some(Object::Nil);
                }
                return Some(Object::Nil);
            }
            ExprType::ClassGet { callee, prop } => {
                let class = self.envs.borrow_mut().get(callee.to_string()).unwrap();
                if let Object::ClassInstance {
                    name: _,
                    fields,
                    properties: _,
                } = class.clone()
                {
                    if let Some(object) = fields.borrow().get(&prop.0) {
                        return Some(object.clone());
                    }
                    return Some(Object::Nil);
                }
                return Some(class.clone());
            }
            ExprType::IndexExpr(ident, index) => {
                let left = self.evaluate_expr(ident).unwrap();
                let index = self.evaluate_expr(index).unwrap();
                return self.eval_index_expr(left, index);
            }
            _ => {
                println!("not found2 {:?}", expr);
                return Some(Object::Nil);
            }
        }
    }

    fn eval_index_expr(&mut self, left: Object, index: Object) -> Option<Object> {
        match left.clone() {
            Object::Array(arr) => {
                if let Object::Index(index) = index {
                    return Some(arr[index].clone());
                } else if let Object::Number(index) = index {
                    return Some(arr[index as usize].clone());
                }
                return Some(Object::Nil);
            }
            Object::Hash(hash) => {
                if let Some(value) = hash.borrow().get(&index) {
                    return Some(value.clone());
                }
                return Some(Object::Nil);
            }
            Object::String(s) => {
                if let Object::Number(index) = index {
                    if let Some(str) = s.chars().nth(index as usize) {
                        return Some(Object::String(str.to_string()));
                    }
                    return Some(Object::Nil);
                } else if let Object::Index(idx) = index {
                    return Some(Object::String(s.chars().nth(idx).unwrap().to_string()));
                } else if let Object::String(index) = index {
                    panic!("not support string index {:?}[{:?}]", left, index);
                }
                return Some(Object::Nil);
            }
            _ => unimplemented!(
                "index only support array, hash and string. not support {:?}, index: {:?}",
                left,
                index
            ),
        }
    }
}

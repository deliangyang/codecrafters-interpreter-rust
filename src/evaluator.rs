use std::{cell::RefCell, collections::HashMap, process::exit, rc::Rc};

use crate::{
    ast::{ExprType, Literal, Progam, Stmt},
    builtins,
    envs::Env,
    objects::Object,
    token::Token,
};

pub struct Evaluator {
    output: bool,
    pub ast: Progam,
    builtins: HashMap<String, Object>,
    envs: Rc<RefCell<Env>>,
}

impl Evaluator {
    pub fn new(ast: Progam, output: bool) -> Self {
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

    fn evaluate_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(ident, expr) => {
                let name = ident.0.clone();
                let object = self.evaluate_expr(expr).unwrap();
                self.envs.borrow_mut().set_store(name, &object);
            }
            Stmt::Expr(expr) => {
                let object = self.evaluate_expr(expr).unwrap();
                if self.output {
                    println!("{}", object);
                }
            }
            Stmt::Block(stmts) => {
                let current_env = Rc::clone(&self.envs);
                let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                self.envs = Rc::new(RefCell::new(pre_envs));
                for stmt in stmts {
                    self.evaluate_stmt(stmt);
                }
                self.envs = current_env;
            }
            Stmt::Return(expr) => {
                let object = self.evaluate_expr(expr).unwrap();
                if self.output {
                    println!("{}", object);
                }
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
                                    self.evaluate_stmt(stmt);
                                }
                                break;
                            }
                        }
                        Stmt::Default(block) => {
                            for stmt in block {
                                self.evaluate_stmt(stmt);
                            }
                        }
                        _ => unimplemented!(),
                    }
                }
            }
            Stmt::While(expr, block) => {
                let mut condition = true;
                while condition {
                    let result = self.evaluate_expr(expr).unwrap();
                    if let Object::Boolean(result) = result {
                        if result {
                            let current_env = Rc::clone(&self.envs);
                            let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                            self.envs = Rc::new(RefCell::new(pre_envs));
                            for stmt in block {
                                self.evaluate_stmt(stmt);
                            }
                            self.envs = current_env;
                        } else {
                            condition = false;
                        }
                    }
                }
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
                if let Object::Hash(hash) = iter {
                    for (key, _) in hash {
                        let current_env = Rc::clone(&self.envs);
                        let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                        self.envs = Rc::new(RefCell::new(pre_envs));
                        self.envs.borrow_mut().set(ident.0.clone(), &key);
                        for stmt in block {
                            self.evaluate_stmt(stmt);
                        }
                        self.envs = current_env;
                    }
                }
            }
            _ => unimplemented!(),
        }
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
                    Some(Object::Hash(hash_map))
                }
                Literal::Index(index) => Some(Object::Index(*index)),
            },
            ExprType::Ident(v) => {
                if let Some(builtin) = self.builtins.get(&v.0) {
                    return Some(builtin.clone());
                } else if let Some(env) = self.envs.borrow_mut().get(v.0.clone()) {
                    return Some(env.clone());
                }
                // println!("Undefined variable '{}'.", v.0);
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
                        ExprType::ThisExpr(ident) => {
                            let object = self.evaluate_expr(right).unwrap();
                            self.envs.borrow_mut().set(ident.0.clone(), &object);
                            return Some(object);
                        }
                        _ => {}
                    }
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
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Number(left + right));
                            }
                        } else if let Object::String(left) = left.unwrap() {
                            if let Object::String(right) = right.unwrap() {
                                return Some(Object::String(left + &right));
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
                let expr = self.evaluate_expr(expr).unwrap();
                println!("{}", expr);
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
                            self.evaluate_stmt(stmt);
                        }
                    } else {
                        let mut condition = false;
                        for (cond, block) in elseif {
                            let cond = self.evaluate_expr(cond).unwrap();
                            if let Object::Boolean(cond) = cond {
                                if cond {
                                    for stmt in block {
                                        self.evaluate_stmt(stmt);
                                    }
                                    condition = true;
                                    break;
                                }
                            }
                        }
                        if !condition {
                            for stmt in else_branch {
                                self.evaluate_stmt(stmt);
                            }
                        }
                    }
                }
                return Some(Object::Nil);
            }
            ExprType::Function { params, body } => {
                return Some(Object::Function(params.clone(), body.clone()));
            }
            ExprType::Call { callee, args } => {
                let mut result = Some(Object::Nil);
                let callee = self.evaluate_expr(&callee);
                match callee {
                    Some(Object::Builtin(argc, fun)) => {
                        let mut args_vec = Vec::new();
                        for arg in args {
                            args_vec.push(self.evaluate_expr(arg).unwrap());
                        }
                        if args_vec.len() as i32 != argc {
                            println!("Expected {} arguments but got {}.", argc, args_vec.len());
                            exit(70);
                        }
                        result = Some(fun(args_vec));
                    }
                    Some(Object::Function(ident, stmts)) => {
                        let current_env = Rc::clone(&self.envs);
                        let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                        self.envs = Rc::new(RefCell::new(pre_envs));
                        for (i, param) in ident.iter().enumerate() {
                            let arg: Object = self.evaluate_expr(&args[i]).unwrap();
                            self.envs.borrow_mut().set_store(param.0.clone(), &arg);
                        }
                        for stmt in stmts {
                            match stmt {
                                Stmt::Return(expr) => {
                                    result = self.evaluate_expr(&expr);
                                    break;
                                }
                                Stmt::Blank => {}
                                _ => {
                                    self.evaluate_stmt(&stmt);
                                }
                            }
                        }
                        self.envs = current_env;
                    }
                    _ => unimplemented!("not found {:?}", callee),
                }

                return result;
            }
            ExprType::ClassInit { name, args } => {
                // println!("{:?} {:?}", name, args);
                let class_name = name.clone().to_string();
                let class = self.envs.borrow_mut().get(class_name).unwrap();

                if let Object::Class(_, properties) = class.clone() {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));
                    let init_func = properties
                        .iter()
                        .find(|x| {
                            if let Stmt::Function(ident, _, _) = x {
                                if ident.0 == "init" {
                                    return true;
                                }
                            }
                            false
                        })
                        .unwrap();
                    if let Stmt::Function(_, params, block) = init_func {
                        for (i, arg) in args.iter().enumerate() {
                            let arg: Object = self.evaluate_expr(&arg).unwrap();
                            let ident = params[i].0.clone();
                            self.envs.borrow_mut().set(ident, &arg);
                        }
                        for stmt in block {
                            self.evaluate_stmt(stmt);
                        }
                    }
                    self.envs = current_env;
                }

                return Some(Object::ClassInstance {
                    name: name.clone().to_string(),
                    class: Rc::new(RefCell::new(class.clone())),
                    properties: Rc::new(RefCell::new(HashMap::new())),
                });
            }
            ExprType::ClassCall {
                callee,
                method,
                args,
            } => {
                let class = self.envs.borrow_mut().get(callee.to_string()).unwrap();
                if let Object::Class(_, stmts) = class {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));
                    for stmt in stmts {
                        match stmt {
                            Stmt::Function(ident, _, block) => {
                                if ident.0 == method.to_string() {
                                    for stmt in block {
                                        self.evaluate_stmt(&stmt);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    self.envs = current_env;
                }
                println!("------------>{} {:?} {:?}", callee, method, args);
                return Some(Object::Nil);
            }
            ExprType::ThisExpr(ident) => {
                let object = self.envs.borrow_mut().get(ident.0.clone()).unwrap();
                return Some(object.clone());
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
        match left {
            Object::Array(arr) => {
                if let Object::Index(index) = index {
                    return Some(arr[index].clone());
                } else if let Object::Number(index) = index {
                    return Some(arr[index as usize].clone());
                }
                return Some(Object::Nil)
            },
            Object::Hash(hash) => {
                return Some(hash.get(&index).unwrap().clone());
            },
            _ => unimplemented!("not found {:?}", left),
        }
    }
}

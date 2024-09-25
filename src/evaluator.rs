use std::{cell::RefCell, collections::HashMap, process::exit, rc::Rc};

use crate::{ast::{ExprType, Literal, Progam, Stmt}, builtins, envs::Env, objects::Object, token::Token};

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
            },
            Stmt::Block(stmts) => {
                let current_env = Rc::clone(&self.envs);
                let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                self.envs = Rc::new(RefCell::new(pre_envs));
                for stmt in stmts {
                    self.evaluate_stmt(stmt);
                }
                self.envs = current_env;
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
            ExprType::BinaryExpr(left, op, right) => {
                let left = self.evaluate_expr(left).unwrap();
                let right = self.evaluate_expr(right).unwrap();
                match op {
                    Token::Plus => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left + right));
                            }
                        }
                        return None;
                    }
                    Token::Minus => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left - right));
                            }
                        }
                       
                        return None;
                    }
                    Token::Star => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left * right));
                            }
                        }
                        return None;
                    }
                    Token::Slash => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left / right));
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
                            // println!("{:?} {:?} {:?}", left, op, right);
                            let object = self.evaluate_expr(right).unwrap();
                            self.envs.borrow_mut().set(name, &object);
                            // println!("{}", object);
                            return Some(object);
                        }
                        _ => {},
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
                        }else if let Object::Boolean(left) = left.clone().unwrap() {
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
                        println!("{:?} {:?} {:?}", left, op, right);
                        unimplemented!()
                    },
                }
            }
            ExprType::PrintExpr(expr) => {
                let expr = self.evaluate_expr(expr).unwrap();
                println!("{}", expr);
                return Some(Object::Nil);
            }
            ExprType::If { condition, elseif, then_branch, else_branch } => {
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
                let callee = self.evaluate_expr(&callee);
                if let Some(Object::Function(ident, stmts)) = callee {
                    let current_env = Rc::clone(&self.envs);
                    let pre_envs = Env::new_with_outer(Rc::clone(&current_env));
                    self.envs = Rc::new(RefCell::new(pre_envs));
                    for (i, param) in ident.iter().enumerate() {
                        let arg: Object = self.evaluate_expr(&args[i]).unwrap();
                        self.envs.borrow_mut().set_store(param.0.clone(), &arg);
                    }
                    for stmt in stmts {
                        self.evaluate_stmt(&stmt);
                    }
                    self.envs = current_env;
                }

                return Some(Object::Nil);
            }
            _ => {
                println!("{:?}", expr);
                return Some(Object::Nil);
            }
        }
    }
}

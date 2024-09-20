use std::{collections::HashMap, process::exit};

use crate::{ast::{ExprType, Literal, Progam, Stmt}, builtins, objects::Object, token::Token};

pub struct Evaluator {
    output: bool,
    pub ast: Progam,
    builtins: HashMap<String, Object>,
    envs: HashMap<String, Object>,
}

impl Evaluator {
    pub fn new(ast: Progam, output: bool) -> Self {
        Self { 
            ast: ast,
            builtins: builtins::new_builtins(),
            output: output,
            envs: HashMap::new(),
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
                self.envs.insert(name, object);
            }
            Stmt::Expr(expr) => {
                let object = self.evaluate_expr(expr).unwrap();
                if self.output {
                    println!("{}", object);
                }
            }
        }
    }

    fn evaluate_expr(&self, expr: &ExprType) -> Option<Object> {
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
                } else if let Some(env) = self.envs.get(&v.0) {
                    return Some(env.clone());
                }
                return None;
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
                    _ => unimplemented!(),
                }
            }
            ExprType::PrintExpr(expr) => {
                let expr = self.evaluate_expr(expr).unwrap();
                println!("{}", expr);
                return Some(Object::Nil);
            }
            _ => {
                println!("{:?}", expr);
                return Some(Object::Nil);
            }
        }
    }
}

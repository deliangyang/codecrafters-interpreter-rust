use crate::{ast::{ExprType, Literal, Progam, Stmt}, objects::Object};

pub struct Evaluator {
    pub ast: Progam,
}

impl Evaluator {
    pub fn new(ast: Progam) -> Self {
        Self { ast }
    }

    pub fn evaluate(&self) {
        for stmt in &self.ast {
            self.evaluate_stmt(stmt);
        }
    }

    fn evaluate_stmt(&self, stmt: &Stmt) {
        match stmt {
            Stmt::Var(ident, expr) => {
                println!("var {} = {}", ident.0, self.evaluate_expr(expr).unwrap());
            }
            Stmt::Expr(expr) => {
                println!("{}", self.evaluate_expr(expr).unwrap());
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
            ExprType::BinaryExpr(left, op, right) => {
                let left = self.evaluate_expr(left).unwrap();
                let right = self.evaluate_expr(right).unwrap();
                match op {
                    crate::token::Token::Plus => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left + right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::Minus => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left - right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::Star => {
                        if let Object::Number(left) = left {
                            if let Object::Number(right) = right {
                                return Some(Object::Number(left * right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::Slash => {
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
                    crate::token::Token::EqualEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left == right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::BangEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left != right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::Less => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left < right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::LessEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left <= right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::Greater => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left > right));
                            }
                        }
                        return None;
                    }
                    crate::token::Token::GreaterEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left >= right));
                            }
                        }
                        return None;
                    }
                    _ => unimplemented!(),
                }
            }
            _ => {
                println!("{:?}", expr);
                return Some(Object::Nil);
            }
        }
    }
}

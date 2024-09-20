use crate::{ast::{ExprType, Literal, Progam, Stmt}, token::Token, objects::Object};

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
            ExprType::GroupingExpr(expr) => self.evaluate_expr(expr),
            ExprType::PrefixExpr(op, expr) => {
                let expr = self.evaluate_expr(expr).unwrap();
                match op {
                    Token::Minus => {
                        if let Object::Number(expr) = expr {
                            return Some(Object::Number(-expr));
                        }
                        return None;
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
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left == right));
                            }
                        }
                        return None;
                    }
                    Token::BangEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left != right));
                            }
                        }
                        return None;
                    }
                    Token::Less => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left < right));
                            }
                        }
                        return None;
                    }
                    Token::LessEqual => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left <= right));
                            }
                        }
                        return None;
                    }
                    Token::Greater => {
                        if let Object::Number(left) = left.unwrap() {
                            if let Object::Number(right) = right.unwrap() {
                                return Some(Object::Boolean(left > right));
                            }
                        }
                        return None;
                    }
                    Token::GreaterEqual => {
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

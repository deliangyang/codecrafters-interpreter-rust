use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{ExprType, Literal, Program, Stmt},
    builtins,
    objects::Object,
    opcode::Opcode,
    symbol::SymbolTable,
    token::Token,
};

pub struct Compiler {
    program: Program,
    pub constants: Vec<Object>,
    pub instructions: Vec<Opcode>,
    pub builtins: HashMap<String, Object>,
    pub symbols: Rc<RefCell<SymbolTable>>,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        let symbols = Rc::new(RefCell::new(SymbolTable::new()));
        let mut constants = Vec::new();
        let builtins = builtins::new_builtins();
        let mut keys = builtins.keys().collect::<Vec<&String>>();
        keys.sort();
        for k in keys {
            let func = builtins.get(k).unwrap().clone();
            symbols.borrow_mut().define(k.clone());
            constants.push(func.clone());
        }

        Compiler {
            program,
            constants,
            instructions: Vec::new(),
            builtins: builtins::new_builtins(),
            symbols: symbols,
        }
    }

    pub fn compile(&mut self) {
        for statement in self.program.clone() {
            self.compile_statement(&statement);
        }
    }

    fn compile_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expression(expr);
            }
            Stmt::Var(ident, expr) => {
                self.compile_expression(expr);
                let symbol = self.symbols.borrow_mut().define(ident.0.clone());
                self.emit(Opcode::SetGlobal(symbol.index));
            }
            Stmt::Block(stmts) => {
                for stmt in stmts.iter() {
                    self.compile_statement(stmt);
                }
            }
            _ => unimplemented!("Statement not implemented: {:?}", stmt),
        }
    }

    fn compile_expression(&mut self, expr: &ExprType) {
        match expr {
            ExprType::InfixExpr(left, op, right) => {
                if *op == Token::Equal {
                    match left.as_ref() {
                        ExprType::Ident(ident) => {
                            let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                            if symbol.is_none() {
                                unimplemented!("Symbol not found: {:?}", ident);
                            }
                            self.compile_expression(right);
                            self.emit(Opcode::SetGlobal(symbol.unwrap().index));
                            return;
                        }
                        _ => unimplemented!("Left side of assignment not implemented: {:?}", left),
                    }
                }
                self.compile_expression(left);
                self.compile_expression(right);
                match op {
                    Token::Plus => self.emit(Opcode::Add),
                    Token::Star => self.emit(Opcode::Multiply),
                    Token::Slash => self.emit(Opcode::Divide),
                    Token::Mod => self.emit(Opcode::Mod),
                    Token::Minus => self.emit(Opcode::Minus),
                    Token::Greater => self.emit(Opcode::GreaterThan),
                    Token::Less => self.emit(Opcode::LessThan),
                    _ => unimplemented!("Operator not implemented: {:?}", op),
                }
            }
            ExprType::PrintExpr(expr) => {
                for expr in expr.iter() {
                    self.compile_expression(expr);
                }
                self.emit(Opcode::Print(expr.len()));
            }
            ExprType::PrefixExpr(op, expr) => {
                self.compile_expression(expr);
                match op {
                    Token::Minus => {
                        self.emit(Opcode::LoadConstant(0));
                        self.emit(Opcode::Nagetive);
                    }
                    _ => unimplemented!("prefix expr Operator not implemented: {:?}", op),
                }
            }
            ExprType::Literal(lit) => {
                let index = self.constants.len();
                match lit {
                    Literal::Number(n) => self.constants.push(Object::Number(*n)),
                    Literal::String(s) => self.constants.push(Object::String(s.clone())),
                    Literal::Bool(b) => self.constants.push(Object::Boolean(*b)),
                    Literal::Nil => self.constants.push(Object::Nil),
                    _ => unimplemented!("Literal not implemented: {:?}", lit),
                }
                self.emit_load_constant(index);
            }
            ExprType::Ident(ident) => {
                let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                if symbol.is_none() {
                    unimplemented!("Symbol not found: {:?}", ident);
                }
                self.emit(Opcode::GetGlobal(symbol.unwrap().index));
            }
            ExprType::Call { callee, args } => {
                match callee.as_ref() {
                    ExprType::Ident(ident) => {
                        if self.builtins.contains_key(&ident.0) {
                            let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                            let index = symbol.unwrap().index;
                            self.emit(Opcode::GetBuiltin(index));
                        }
                    }
                    _ => unimplemented!("Callee not implemented: {:?}", callee),
                };

                for arg in args.iter() {
                    self.compile_expression(arg);
                }
                self.emit(Opcode::Call(args.len()));
            }
            ExprType::If {
                condition,
                elseif,
                then_branch,
                else_branch,
            } => {
                self.compile_expression(condition);
                let jump_not_truthy = self.instructions.len();
                self.emit(Opcode::JumpIfFalse(0));
                self.compile_statement(&Stmt::Block(then_branch.clone()));

                let else_if_pos = self.instructions.len();
                if else_branch.len() > 0 {
                    self.emit(Opcode::Jump(0));
                }

                let jump = self.instructions.len();
                self.emit(Opcode::Jump(0));
                self.instructions[jump_not_truthy] = Opcode::JumpIfFalse(jump);
                for (condition, block) in elseif.iter() {
                    self.compile_expression(condition);
                    let jump_not_truthy = self.instructions.len();
                    self.emit(Opcode::JumpIfFalse(0));
                    self.compile_statement(&Stmt::Block(block.clone()));
                    let jump = self.instructions.len();
                    self.emit(Opcode::Jump(0));
                    self.instructions[jump_not_truthy] = Opcode::JumpIfFalse(jump);
                }

                if else_branch.len() > 0 {
                    self.compile_statement(&Stmt::Block(else_branch.clone()));
                    let pos = self.instructions.len();
                    self.instructions[else_if_pos] = Opcode::Jump(pos);
                }
                
            }
            _ => unimplemented!("Expression not implemented: {:?}", expr),
        }
    }

    pub fn emit(&mut self, op: Opcode) {
        self.instructions.push(op);
    }

    pub fn emit_load_constant(&mut self, index: usize) {
        self.emit(Opcode::LoadConstant(index));
    }

    pub fn emit_add(&mut self) {
        self.emit(Opcode::Add);
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexing, parser::Parser};

    use super::*;

    #[test]
    fn test_compiler() {
        let lexer = Lexing::new("1 + 2");
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        assert_eq!(compiler.instructions.len(), 3);
    }
}

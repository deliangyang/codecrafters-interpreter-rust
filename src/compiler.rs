use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{ExprType, Literal, Program, Stmt},
    builtins::Builtins,
    objects::Object,
    opcode::Opcode,
    symbol::{Scope, Symbol, SymbolTable},
    token::Token,
};

pub struct Compiler {
    program: Program,
    pub constants: Vec<Object>,
    pub instructions: Vec<Opcode>,
    pre_instructions: Vec<Opcode>,
    pub builtins: Builtins,
    pub symbols: Rc<RefCell<SymbolTable>>,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        Compiler {
            program,
            constants: Vec::new(),
            instructions: Vec::new(),
            builtins: Builtins::new(),
            pre_instructions: vec![],
            symbols: Rc::new(RefCell::new(SymbolTable::new())),
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
                if symbol.scope == Scope::Global {
                    self.emit(Opcode::SetGlobal(symbol.index));
                } else {
                    self.emit(Opcode::SetLocal(symbol.index));
                }
            }
            Stmt::Block(stmts) => {
                for stmt in stmts.iter() {
                    self.compile_statement(stmt);
                }
            }
            Stmt::Function(ident, args, body) => {
                let symbol = self.symbols.borrow_mut().define(ident.0.clone());

                self.enter_scope();

                self.emit(Opcode::SetGlobal(symbol.index));

                for arg in args.iter() {
                    let symbol = self.symbols.borrow_mut().define(arg.0.clone());
                    self.emit(Opcode::SetGlobal(symbol.index));
                }

                for stmt in body.iter() {
                    self.compile_statement(stmt);
                }

                let instraction = self.leave_scope();

                let num_locals = self.symbols.borrow().num_definitions;
                let num_parameters = args.len();

                let compiled_object = Object::CompiledFunction {
                    instructions: instraction,
                    num_locals,
                    num_parameters,
                };
                let index = self.constants.len();
                self.constants.push(compiled_object);
                self.emit_load_constant(index);
                self.emit(Opcode::Closure(index, num_locals));
                self.emit(Opcode::SetGlobal(symbol.index));
            }
            Stmt::For {
                init,
                conditions,
                step,
                block,
            } => {
                self.compile_statement(init);
                let start = self.instructions.len();
                self.compile_expression(conditions);
                let jump_not_truthy = self.emit_return_position(Opcode::JumpIfFalse(0));
                self.compile_statement(&Stmt::Block(block.clone()));
                self.compile_statement(step);
                self.emit(Opcode::Jump(start));
                let end = self.instructions.len();
                self.instructions[jump_not_truthy] = Opcode::JumpIfFalse(end);
            }
            Stmt::Assert { condition, message } => {
                self.compile_expression(condition);
                let msg = match message.as_ref() {
                    ExprType::Literal(Literal::String(msg)) => msg,
                    _ => "",
                };
                if msg.is_empty() {
                    self.emit(Opcode::Assert(0));
                    return;
                }
                let pos = self.emit_return_position(Opcode::Assert(0));
                self.compile_expression(message);
                self.emit(Opcode::Print(1));
                self.emit(Opcode::Exit(3));
                self.instructions[pos] = Opcode::Assert(self.instructions.len());
            }
            Stmt::Assign(ident, right) => match ident {
                ExprType::Ident(ident) => {
                    let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                    if symbol.is_none() {
                        unimplemented!("Symbol not found: {:?}", ident);
                    }
                    self.compile_expression(right);
                    self.load_symbol(symbol.unwrap().clone());
                    return;
                }
                _ => unimplemented!("Left side of assignment not implemented: {:?}", ident),
            },
            Stmt::Return(expr) => {
                self.compile_expression(expr);
                self.emit(Opcode::ReturnValue);
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
                            self.load_symbol(symbol.unwrap().clone());
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
                    Token::EqualEqual => self.emit(Opcode::EqualEqual),
                    Token::PlusSelf => {
                        self.emit(Opcode::Add);
                        match left.as_ref() {
                            ExprType::Ident(ident) => {
                                let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                                if symbol.is_none() {
                                    unimplemented!("Symbol not found: {:?}", ident);
                                }
                                self.emit(Opcode::SetGlobal(symbol.unwrap().index));
                            }
                            _ => unimplemented!(
                                "Left side of assignment not implemented: {:?}",
                                left
                            ),
                        }
                    }
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
                self.load_symbol(symbol.unwrap().clone());
            }
            ExprType::Call { callee, args } => {
                match callee.as_ref() {
                    ExprType::Ident(ident) => {
                        let index = self.builtins.get_index(ident.0.as_str());
                        if index.is_some() {
                            self.emit(Opcode::GetBuiltin(index.unwrap()));
                        } else {
                            let symbol = self.symbols.borrow_mut().resolve(ident.0.as_str());
                            if symbol.is_none() {
                                unimplemented!("Symbol not found: {:?}", ident);
                            }
                            self.emit(Opcode::GetGlobal(symbol.unwrap().index));
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
                // Compile condition
                self.compile_expression(condition);
                // If condition is false, jump to end of if-else
                let jump_not_truthy = self.emit_return_position(Opcode::JumpIfFalse(0));
                self.compile_statement(&Stmt::Block(then_branch.clone()));

                let mut endif = vec![];
                let exist_else = else_branch.len() > 0;

                if exist_else {
                    let pos = self.emit_return_position(Opcode::Jump(9999));
                    endif.push(pos);
                }

                let jump = self.instructions.len();
                self.instructions[jump_not_truthy] = Opcode::JumpIfFalse(jump);

                for (condition, block) in elseif.iter() {
                    self.compile_expression(condition);
                    let jump_not_truthy = self.emit_return_position(Opcode::JumpIfFalse(0));
                    self.compile_statement(&Stmt::Block(block.clone()));

                    if exist_else {
                        endif.push(self.instructions.len());
                        self.emit(Opcode::Jump(9999)); // Jump to end of if-else, will be replaced later
                    }

                    let jump = self.instructions.len();
                    self.instructions[jump_not_truthy] = Opcode::JumpIfFalse(jump);
                }

                if exist_else {
                    self.compile_statement(&Stmt::Block(else_branch.clone()));
                }

                for pos in endif.iter() {
                    self.instructions[*pos] = Opcode::Jump(self.instructions.len());
                }
            }
            _ => unimplemented!("Expression not implemented: {:?}", expr),
        }
    }

    pub fn emit(&mut self, op: Opcode) {
        self.instructions.push(op);
    }

    pub fn emit_return_position(&mut self, op: Opcode) -> usize {
        let pos = self.instructions.len();
        self.instructions.push(op);
        pos
    }

    pub fn emit_load_constant(&mut self, index: usize) {
        self.emit(Opcode::LoadConstant(index));
    }

    pub fn emit_add(&mut self) {
        self.emit(Opcode::Add);
    }

    fn enter_scope(&mut self) {
        let symbols = SymbolTable::new_enclosed(self.symbols.borrow().clone());
        self.symbols = Rc::new(RefCell::new(symbols));
        self.pre_instructions = self.instructions.clone();
        self.instructions = vec![];
    }

    fn leave_scope(&mut self) -> Vec<Opcode> {
        let symbols = self.symbols.borrow().outer.clone().unwrap();
        self.symbols = Rc::new(RefCell::new(*symbols));
        let instructions = self.instructions.clone();
        self.instructions = self.pre_instructions.clone();
        self.pre_instructions = vec![];
        instructions
    }

    fn load_symbol(&mut self, s: Symbol) {
        match s.scope {
            Scope::Global => self.emit(Opcode::GetGlobal(s.index)),
            Scope::Local => self.emit(Opcode::GetLocal(s.index)),
            Scope::Free => self.emit(Opcode::GetFree(s.index)),
            Scope::Builtin => self.emit(Opcode::GetBuiltin(s.index)),
            Scope::Function => self.emit(Opcode::CurrentClosure),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexing, parser::Parser};

    use super::*;

    #[test]
    fn test_compiler() {
        let instractions = test_compiler_code("1 + 2;");
        assert_eq!(instractions.len(), 3);
        assert_eq!(
            instractions,
            vec![
                Opcode::LoadConstant(0),
                Opcode::LoadConstant(1),
                Opcode::Add,
            ]
        );
    }

    #[test]
    fn test_print() {
        let ins = test_compiler_code("print 1, 2;");
        assert_eq!(ins.len(), 3);
        assert_eq!(
            ins,
            vec![
                Opcode::LoadConstant(0),
                Opcode::LoadConstant(1),
                Opcode::Print(2),
            ]
        );
    }

    #[test]
    fn test_builtin() {
        let ins = test_compiler_code("len('hello');");
        let builtins = Builtins::new();
        assert_eq!(ins.len(), 3);
        assert_eq!(
            ins,
            vec![
                Opcode::GetBuiltin(builtins.get_index("len").unwrap()),
                Opcode::LoadConstant(0),
                Opcode::Call(1),
            ]
        );
    }

    #[test]
    fn test_add() {
        let ins = test_compiler_code("1 + 2;");
        assert_eq!(ins.len(), 3);
        assert_eq!(
            ins,
            vec![
                Opcode::LoadConstant(0),
                Opcode::LoadConstant(1),
                Opcode::Add,
            ]
        );
    }

    #[test]
    fn test_if() {
        let ins = test_compiler_code("if (1 > 2) { print 'yes'; } else { print 'no'; }");
        let except = vec![
            Opcode::LoadConstant(0), // 1
            Opcode::LoadConstant(1), // 2
            Opcode::GreaterThan,     // 3
            Opcode::JumpIfFalse(7),  // 4
            Opcode::LoadConstant(2), // 5
            Opcode::Print(1),        // 6
            Opcode::Jump(9),         // 7
            Opcode::LoadConstant(3), // 8
            Opcode::Print(1),        // 9
        ];
        assert_eq!(ins.len(), except.len());
        assert_eq!(ins, except);
    }

    #[test]
    fn test_if_with_elseif() {
        let ins = test_compiler_code(
            "if (1 > 2) { print 'yes'; } else if (1 < 2) { print 'no'; } else { print 'no'; }",
        );
        let except = vec![
            Opcode::LoadConstant(0), // 1
            Opcode::LoadConstant(1), // 2
            Opcode::GreaterThan,     // 3
            Opcode::JumpIfFalse(7),  // 4
            Opcode::LoadConstant(2), // 5
            Opcode::Print(1),        // 6
            Opcode::Jump(16),        // 7
            Opcode::LoadConstant(3), // 8
            Opcode::LoadConstant(4), // 9
            Opcode::LessThan,        // 10
            Opcode::JumpIfFalse(14), // 11
            Opcode::LoadConstant(5), // 12
            Opcode::Print(1),        // 13
            Opcode::Jump(16),        // 14
            Opcode::LoadConstant(6), // 15
            Opcode::Print(1),        // 16
        ];
        assert_eq!(ins.len(), except.len());
        assert_eq!(ins, except);
    }

    #[test]
    fn test_assert() {
        let ins = test_compiler_code("assert 1 > 2, '1 is not greater than 2';");
        println!("{:?}", ins);
        let except = vec![
            Opcode::LoadConstant(0), // 1
            Opcode::LoadConstant(1), // 2
            Opcode::GreaterThan,     // 3
            Opcode::Assert(7),       // 4
            Opcode::LoadConstant(2), // 5
            Opcode::Print(1),        // 6
            Opcode::Exit(3),         // 7
        ];
        assert_eq!(ins.len(), except.len());
        assert_eq!(ins, except);
    }

    #[test]
    fn test_assert_with_var() {
        let ins = test_compiler_code("var a = 1; assert a > 2, '1 is not greater than 2';");
        let except = vec![
            Opcode::LoadConstant(0),
            Opcode::SetGlobal(0),
            Opcode::GetGlobal(0),    // 1
            Opcode::LoadConstant(1), // 2
            Opcode::GreaterThan,     // 3
            Opcode::Assert(9),       // 4
            Opcode::LoadConstant(2), // 5
            Opcode::Print(1),        // 6
            Opcode::Exit(3),         // 7
        ];
        assert_eq!(ins.len(), except.len());
        assert_eq!(ins, except);
    }

    fn test_compiler_code(code: &str) -> Vec<Opcode> {
        let lexer = Lexing::new(code);
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        compiler.instructions
    }
}

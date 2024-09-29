use crate::{
    ast::{ExprType, Literal, Program, Stmt},
    objects::Object,
    opcode::Opcode,
    token::Token,
};

pub struct Compiler {
    program: Program,
    pub constants: Vec<Object>,
    pub instructions: Vec<Opcode>,
}

impl Compiler {
    pub fn new(program: Program) -> Compiler {
        Compiler {
            program,
            constants: Vec::new(),
            instructions: Vec::new(),
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
                self.emit(Opcode::DefineGlobal(ident.0.clone()));
            }
            Stmt::Block(stmts) => {
                for stmt in stmts.iter() {
                    self.compile_statement(stmt);
                }
            }
            _ => unimplemented!(),
        }
    }

    fn compile_expression(&mut self, expr: &ExprType) {
        match expr {
            ExprType::InfixExpr(left, op, right) => {
                self.compile_expression(left);
                self.compile_expression(right);
                match op {
                    Token::Plus => self.emit(Opcode::Add),
                    _ => unimplemented!(),
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
                    _ => unimplemented!(),
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
                self.emit(Opcode::GetGlobal(0));
            }
            ExprType::Call { callee, args } => {
                for arg in args.iter() {
                    self.compile_expression(arg);
                }
                self.compile_expression(callee);
                self.emit(Opcode::Call(args.len()));
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

use std::{cell::RefCell, rc::Rc};

use crate::{objects::Object, opcode::Opcode};

pub struct VM {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    sp: usize, // stack pointer
    ip: usize, // instruction pointer
}

const NIL: Object = Object::Nil;
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl VM {
    pub fn new() -> VM {
        VM {
            constants: Vec::new(),
            stack: Vec::new(),
            globals: vec![NIL; GLOBALS_SIZE],
            sp: 0,
            ip: 0,
        }
    }

    pub fn run(&mut self, instructions: Vec<Opcode>) -> Object {
        let instractions = Rc::new(RefCell::new(instructions));
        let l = instractions.borrow().len();
        loop {
            if self.ip >= l {
                break;
            }
            let instruction = instractions.borrow()[self.ip].clone();
            self.execute(instruction);
            self.ip += 1;
        }
        self.pop()
    }

    fn push(&mut self, obj: Object) {
        self.stack.push(obj);
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        let obj = self.stack.pop().unwrap();
        self.sp -= 1;
        obj
    }

    fn execute(&mut self, instruction: Opcode) {
        match instruction {
            Opcode::Add
            | Opcode::Divide
            | Opcode::Minus
            | Opcode::Multiply
            | Opcode::Mod
            | Opcode::LessThan
            | Opcode::GreaterThan => {
                let right = self.pop();
                let left = self.pop();
                let result = match (left, right) {
                    (Object::Number(l), Object::Number(r)) => match instruction {
                        Opcode::Add => Object::Number(l + r),
                        Opcode::Divide => Object::Number(l / r),
                        Opcode::Minus => Object::Number(l - r),
                        Opcode::Multiply => Object::Number(l * r),
                        Opcode::Mod => Object::Number(l % r),
                        Opcode::GreaterThan => Object::Boolean(l > r),
                        Opcode::LessThan => Object::Boolean(l < r),
                        _ => Object::Nil,
                    },
                    _ => Object::Nil,
                };
                self.push(result);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    self.ip = pos;
                }
            }
            Opcode::Jump(pos) => {
                self.ip += pos;
            }
            Opcode::LoadConstant(index) => {
                self.push(self.constants[index].clone());
            }
            Opcode::Pop => {
                self.pop();
            }
            Opcode::Abs => {
                let obj = self.pop();
                self.push(Object::Number(match obj {
                    Object::Number(n) => n.abs(),
                    _ => 0.0,
                }));
            }
            Opcode::Nagetive => {
                let obj = self.pop();
                self.push(Object::Number(match obj {
                    Object::Number(n) => -n,
                    _ => 0.0,
                }));
            }
            Opcode::Print(n) => {
                for _ in 0..n {
                    let obj = self.pop();
                    print!("{:?} ", obj);
                }
            }
            Opcode::DefineGlobal(s) => {
                let obj = self.pop();
                println!("{} = {:?}", s, obj);
            }
            Opcode::GetGlobal(index) => {
                self.push(self.globals[index].clone());
            }
            Opcode::SetGlobal(index) => {
                let obj = self.pop();
                self.globals[index] = obj;
            }
            Opcode::GetBuiltin(index) => {
                self.push(self.constants[index].clone());
            }
            Opcode::Call(n) => {
                let mut args = Vec::new();
                for _ in 0..n {
                    args.push(self.pop());
                }
                let func = self.pop();
                match func {
                    Object::Builtin(_, _, f) => {
                        let result = f(args);
                        self.push(result);
                    }
                    _ => unimplemented!("unimplemented function: {:?}", func),
                }
            }
            _ => unimplemented!("unimplemented opcode: {:?}", instruction),
        }
    }

    pub fn define_constants(&mut self, constants: Vec<Object>) {
        self.constants = constants;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compiler::Compiler, lexer::Lexing, parser::Parser};

    #[test]
    fn test_vm() {
        let lexer = Lexing::new("1+2");
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut vm = VM::new();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        vm.define_constants(compiler.constants);
        let result = vm.run(compiler.instructions);
        assert_eq!(result, Object::Number(3.0));
    }

    #[test]
    fn test_nagetive_number() {
        let lexer = Lexing::new("-2;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut vm = VM::new();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        println!("{:?}", compiler.instructions);
        vm.define_constants(compiler.constants);
        let result = vm.run(compiler.instructions);
        assert_eq!(result, Object::Number(-2.0));
    }

    #[test]
    fn test_print() {
        let lexer = Lexing::new("print 1, 2;");
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut vm = VM::new();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        vm.define_constants(compiler.constants);
        let result = vm.run(compiler.instructions);
        assert_eq!(result, Object::Nil);
    }
}

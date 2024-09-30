use crate::{objects::Object, opcode::Opcode};

pub struct VM {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
}

// const NIL: Object = Object::Nil;
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl VM {
    pub fn new() -> VM {
        VM {
            constants: Vec::new(),
            stack: Vec::new(),
            globals: vec![Object::Nil; GLOBALS_SIZE],
        }
    }

    pub fn run(&mut self, instructions: Vec<Opcode>) -> Object {
        for instruction in instructions {
            match instruction {
                Opcode::Add => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Object::Number(match (left, right) {
                        (Object::Number(l), Object::Number(r)) => l + r,
                        _ => 0.0,
                    }));
                }
                Opcode::LoadConstant(index) => {
                    self.stack.push(self.constants[index].clone());
                }
                Opcode::Pop => {
                    self.stack.pop().unwrap();
                }
                Opcode::Multiply => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Object::Number(match (left, right) {
                        (Object::Number(l), Object::Number(r)) => l * r,
                        _ => 0.0,
                    }));
                }
                Opcode::Abs => {
                    let obj = self.stack.pop().unwrap();
                    self.stack.push(Object::Number(match obj {
                        Object::Number(n) => n.abs(),
                        _ => 0.0,
                    }));
                }
                Opcode::Nagetive => {
                    let obj = self.stack.pop().unwrap();
                    self.stack.push(Object::Number(match obj {
                        Object::Number(n) => -n,
                        _ => 0.0,
                    }));
                }
                Opcode::Print(n) => {
                    for _ in 0..n {
                        let obj = self.stack.pop().unwrap();
                        print!("{:?} ", obj);
                    }
                    self.stack.push(Object::Nil);
                }
                Opcode::DefineGlobal(s) => {
                    let obj = self.stack.pop().unwrap();
                    println!("{} = {:?}", s, obj);
                }
                Opcode::GetGlobal(index) => {
                    self.stack.push(self.globals[index].clone());
                }
                Opcode::SetGlobal(index) => {
                    let obj = self.stack.pop().unwrap();
                    self.globals[index] = obj;
                }
                Opcode::Call(n) => {
                    let mut args = Vec::new();
                    for _ in 0..n {
                        args.push(self.stack.pop().unwrap());
                    }
                    let func_index = self.stack.pop().unwrap();
                    let func_index = match func_index {
                        Object::Index(i) => i,
                        _ => 0,
                    };
                    let func = self.constants[func_index].clone();
                    match func {
                        Object::Builtin(_, _, f) => {
                            let result = f(args);
                            self.stack.push(result);
                        }
                        _ => unimplemented!("unimplemented function: {:?}", func),
                    }
                }
                _ => unimplemented!("unimplemented opcode: {:?}", instruction),
            }
        }
        self.stack.pop().unwrap()
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

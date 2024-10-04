use std::process::exit;

use crate::{builtins::Builtins, frame::Frame, objects::Object, opcode::Opcode};

pub struct VM {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    builtins: Builtins,
    sp: usize, // stack pointer
    frames: Vec<Frame>,
    frame_index: usize,
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
            builtins: Builtins::new(),
            sp: 0,
            frames: Vec::new(),
            frame_index: 0,
        }
    }

    pub fn run(&mut self, instructions: Vec<Opcode>) -> Object {
        self.push_frame(
            Object::CompiledFunction {
                instructions,
                num_locals: 0,
                num_parameters: 0,
            },
            0,
        );

        loop {
            let mut frame = self.current_frame();
            let instractions = frame.instructions();
            let l = instractions.len();
            if self.ip() >= l {
                self.pop_frame();
                if self.frames.is_empty() {
                    break;
                }
                continue;
            }
            println!("globals: {:?}", self.globals.iter().filter(|x| *x != &NIL).collect::<Vec<_>>());
            let instruction = &instractions[self.ip()];
            self.execute(instruction.clone());
            self.incr_ip();
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
            | Opcode::EqualEqual
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
                        Opcode::EqualEqual => Object::Boolean(l == r),
                        _ => Object::Nil,
                    },
                    _ => Object::Nil,
                };
                self.push(result);
            }
            Opcode::ReturnValue => {
                let obj = self.pop();
                self.pop_frame();
                self.push(obj);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    self.set_ip(pos - 1);
                }
            }
            Opcode::Exit(code) => {
                exit(code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    self.set_ip(pos - 1);
                }
            }
            Opcode::Jump(pos) => {
                self.set_ip(pos - 1);
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
                    print!("{} ", obj);
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
                let obj = self.builtins.get_by_index(index);
                if obj.is_none() {
                    unimplemented!("builtin not found: {:?}", index);
                }
                self.push(obj.unwrap().clone());
            }
            Opcode::Call(n) => {
                let mut args = Vec::new();
                for _ in 0..n {
                    args.push(self.pop());
                }
                args.reverse();
                let func = self.pop();
                match func {
                    Object::Builtin(_, _, f) => {
                        let result = f(args);
                        self.push(result);
                    }
                    Object::CompiledFunction {
                        instructions,
                        num_locals,
                        num_parameters,
                    } => {
                        let ip = self.ip();
                        self.push_frame(
                            Object::CompiledFunction {
                                instructions,
                                num_locals,
                                num_parameters,
                            },
                            ip - n,
                        );
                        for arg in args {
                            self.push(arg);
                        }
                    }
                    _ => unimplemented!("unimplemented function: {:?}", func),
                }
            }
            Opcode::Closure(index, _) => {
                let obj = self.constants[index].clone();
                self.push(obj);
            }
            _ => unimplemented!("unimplemented opcode: {:?}", instruction),
        }
    }

    pub fn define_constants(&mut self, constants: Vec<Object>) {
        self.constants = constants;
    }

    pub fn push_frame(&mut self, cl: Object, base_pointer: usize) {
        let frame = Frame::new(cl, 0, base_pointer);
        self.frames.push(frame);
        self.frame_index += 1;
    }

    pub fn current_frame(&mut self) -> Frame {
        self.frames[self.frame_index - 1].clone()
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
        self.frame_index -= 1;
    }

    pub fn incr_ip(&mut self) {
        self.frames[self.frame_index - 1].incr_ip();
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.frames[self.frame_index - 1].set_ip(ip);
    }

    pub fn ip(&mut self) -> usize {
        self.frames[self.frame_index - 1].ip()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compiler::Compiler, lexer::Lexing, parser::Parser};

    #[test]
    fn test_vm() {
        let result = test_vm_code("1 + 2;");
        assert_eq!(result, Object::Number(3.0));
    }

    #[test]
    fn test_nagetive_number() {
        let result = test_vm_code("-2;");
        assert_eq!(result, Object::Number(-2.0));
    }

    #[test]
    fn test_print() {
        let result = test_vm_code("print 1, 2;");
        assert_eq!(result, Object::Nil);
    }

    #[test]
    fn test_println() {
        let result = test_vm_code("println(1);");
        assert_eq!(result, Object::Nil);
    }

    #[test]
    fn test_println_var() {
        let result = test_vm_code("var a = 1; println(a);");
        assert_eq!(result, Object::Nil);
    }

    fn test_vm_code(code: &str) -> Object {
        let lexer = Lexing::new(code);
        let mut parser = Parser::new(lexer);
        let program = parser.parse();
        let mut vm = VM::new();
        let mut compiler = Compiler::new(program);
        compiler.compile();
        vm.define_constants(compiler.constants);
        vm.run(compiler.instructions)
    }
}

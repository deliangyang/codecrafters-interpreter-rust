use std::{process::exit, vec};

use crate::{builtins::Builtins, objects::Object, opcode::Opcode};

pub struct VM<'a> {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    closures: Vec<(usize, usize)>,
    builtins: Builtins,
    sp: usize, // stack pointer
    main_start: usize,
    instructions: Vec<&'a Opcode>,
    registers: Vec<(usize, usize, usize)>, // ip, free_start, free_len
    free_start: usize,
}

const NIL: Object = Object::Nil;
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl<'a> VM<'a> {
    pub fn new(ins: (usize, Vec<&'a Opcode>)) -> VM {
        VM {
            constants: Vec::new(),
            stack: Vec::new(),
            main_start: ins.0,
            instructions: ins.1,
            globals: vec![NIL; GLOBALS_SIZE],
            builtins: Builtins::new(),
            sp: 0,
            registers: Vec::with_capacity(1024),
            free_start: 0,
            closures: vec![(0, 0); GLOBALS_SIZE],
        }
    }

    pub fn run(&mut self) -> Object {
        let mut ip = self.main_start;
        let l = self.instructions.len();
        println!("ip, {:?}, l: {:?}", ip, l);
        while ip < l {
            let instruction: &Opcode = self.instructions[ip];
            //println!("ip: {:?}, {:?}  {:?} {:?}, free_start: {:?}", ip, instruction, self.registers, self.stack, self.free_start);
            ip = self.execute(instruction, ip, ip >= self.main_start);
        }

        if self.sp <= 0 {
            return NIL;
        }
        self.pop().clone()
    }

    fn push(&mut self, obj: Object) {
        if self.sp >= self.stack.len() {
            self.stack.push(obj);
        } else {
            self.stack[self.sp] = obj;
        }
        self.sp += 1;
    }

    fn pop(&mut self) -> &Object {
        self.sp -= 1;
        &self.stack[self.sp]
        // if self.sp == self.stack.len() {
        //     self.stack.pop().unwrap()
        // } else {
        //     self.stack[self.sp].clone()
        // }
        // let obj = self.stack.pop().unwrap();
        // obj
    }

    fn last(&mut self) -> &Object {
        &self.stack[self.sp - 1]
    }

    #[inline]
    fn execute(&mut self, instruction: &Opcode, ip: usize, is_main: bool) -> usize {
        match instruction {
            Opcode::Add
            | Opcode::Divide
            | Opcode::Minus
            | Opcode::Multiply
            | Opcode::Mod
            | Opcode::LessThan
            | Opcode::EqualEqual
            | Opcode::GreaterThan => {
                let right = &self.stack[self.sp - 1];
                self.sp -= 1;
                let left = &self.stack[self.sp - 1];
                self.stack[self.sp-1] = match (left, right) {
                    (Object::Number(l), Object::Number(r)) => match instruction {
                        Opcode::Add => Object::Number(*l + *r),
                        Opcode::Divide => Object::Number(*l / *r),
                        Opcode::Minus => Object::Number(*l - *r),
                        Opcode::Multiply => Object::Number(*l * *r),
                        Opcode::Mod => Object::Number(*l % *r),
                        Opcode::GreaterThan => Object::Boolean(*l > *r),
                        Opcode::LessThan => Object::Boolean(*l < *r),
                        Opcode::EqualEqual => Object::Boolean(*l == *r),
                        _ => Object::Nil,
                    },
                    _ => Object::Nil,
                };
                // self.push(result.clone());
                ip + 1
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if *obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    if !is_main {
                        *pos
                    } else {
                        self.main_start + *pos
                    }
                }
            }
            Opcode::Exit(code) => {
                exit(*code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if *condition == Object::Boolean(false) {
                    if !is_main {
                        *pos
                    } else {
                        self.main_start + *pos
                    }
                } else {
                    ip + 1
                }
            }
            Opcode::Jump(pos) => {
                if !is_main {
                    *pos
                } else {
                    self.main_start + *pos
                }
            }
            Opcode::LoadConstant(index) => {
                self.push(self.constants[*index].clone());
                ip + 1
            }
            Opcode::Pop => {
                self.pop();
                ip + 1
            }
            Opcode::Abs => {
                let obj = self.last();
                self.stack[self.sp - 1] = Object::Number(match obj {
                    Object::Number(n) => n.abs(),
                    _ => 0.0,
                });
                // self.push(Object::Number(match obj {
                //     Object::Number(n) => n.abs(),
                //     _ => 0.0,
                // }));
                ip + 1
            }
            Opcode::Nagetive => {
                let obj = self.last();
                self.stack[self.sp - 1] = Object::Number(match obj {
                    Object::Number(n) => -n,
                    _ => 0.0,
                });
                // self.push();
                ip + 1
            }
            Opcode::Print(n) => {
                for _ in 0..*n {
                    let obj = self.pop();
                    print!("{} ", *obj);
                }
                ip + 1
            }
            Opcode::DefineGlobal(s) => {
                let obj = self.pop();
                println!("{} = {:?}", s, obj);
                ip + 1
            }
            Opcode::GetGlobal(index) => {
                self.push(self.globals[*index].clone());
                ip + 1
            }
            Opcode::SetGlobal(index) => {
                let obj = self.pop();
                match obj {
                    Object::CompiledFunction {
                        start,
                        len: _,
                        num_locals: _,
                        num_parameters,
                    } => {
                        self.closures[*index] = (*start, *num_parameters);
                    }
                    _ => {
                        self.globals[*index] = obj.clone();
                    }
                }
                ip + 1
            }
            Opcode::GetBuiltin(index) => {
                let obj = self.builtins.get_by_index(*index);
                if obj.is_none() {
                    unimplemented!("builtin not found: {:?}", index);
                }
                self.push(obj.unwrap().clone());
                ip + 1
            }
            Opcode::Call(n) => {
                let func = self.pop().clone();
                // println!("----------> call: {:?}", func);
                match func {
                    Object::Builtin(_, _, f) => {
                        let args = self.stack.split_off(self.sp - n);
                        self.sp -= n;
                        let _ = f(args);
                        ip + 1
                        //self.push(result);
                    }
                    _ => unimplemented!("unimplemented function: {:?}", func),
                }
            }
            Opcode::Closure(index, free_count) => {
                let (start, _) = self.closures[*index];
                self.registers.push((ip, self.free_start, *free_count));
                self.free_start = self.sp - free_count;
                //println!("free_start: {:?}", self.free_start);
                return start;
            }
            Opcode::GetFree(index) => {
                let obj = self.stack[self.free_start + *index].clone();
                self.push(obj);
                ip + 1
            }
            Opcode::Return => {
                //let result = self.pop();
                // self.pop_frame();
                self.stack.drain(self.free_start..self.sp - 1);
                self.sp = self.free_start + 1;
                //self.push(result);
                // self.frees.pop();
                //println!("return ip: {:?}", ip)    ;
                let (ip, arg_start, _) = self.registers.pop().unwrap();
                //println!("return: ip: {:?}, stack: {:?}", ip + 1, self.stack);
                self.free_start = arg_start;
                ip + 1
            }
            Opcode::SetLocal(_index) => {
                // let obj = self.pop();
                // println!("set local: {:?}", index);
                ip + 1
            }
            Opcode::GetLocal(_index) => {
                ip + 1
                //println!("get local: {:?}", index);
                // let frame = self.current_frame();
                // let obj = frame.get_local(index);
                // self.push(obj);
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
        let mut compiler = Compiler::new(program);
        compiler.compile();
        let (l, codes) = compiler.get_instructions();
        let mut vm = VM::new((l, codes.iter().map(|x| x).collect()));
        vm.define_constants(compiler.constants);
        vm.run()
    }
}

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
    frees: Vec<Object>,
    free_top: usize,
    free_index: usize,
    registers: Vec<(usize, usize, usize)>,
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
            //end_ip: ins.0,
            sp: 0,
            frees: vec![],
            free_top: 0,
            free_index: 0,
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
            // println!("ip: {:?}, {:?}  {:?} free: {:?}", ip, instruction, self.registers, self.frees);
            ip = self.execute(instruction, ip, ip >= self.main_start);
        }

        if self.stack.is_empty() {
            return NIL;
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
                self.push(result.clone());
                ip + 1
            }
            Opcode::ReturnValue => {
                let obj = self.pop();
                // self.pop_frame();
                self.push(obj);
                let (ip, _, arg_len) = self.registers.pop().unwrap();
                self.free_index -= arg_len;
                return ip + 1;
                // println!("self.stack: {:?}", self.stack);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
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
                if condition == Object::Boolean(false) {
                    if !is_main {
                        // println!("---------> jump: {:?}", *pos);
                        *pos
                    } else {
                        // println!("jump: {:?}", *pos);
                        self.main_start + *pos
                    }
                } else {
                    ip + 1
                }
            }
            Opcode::Jump(pos) => {
                if !is_main {
                    // println!("---------> jump: {:?}", *pos);
                    *pos
                } else {
                    println!("jump: {:?}", self.main_start + *pos);

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
                let obj = self.pop();
                self.push(Object::Number(match obj {
                    Object::Number(n) => n.abs(),
                    _ => 0.0,
                }));
                ip + 1
            }
            Opcode::Nagetive => {
                let obj = self.pop();
                self.push(Object::Number(match obj {
                    Object::Number(n) => -n,
                    _ => 0.0,
                }));
                ip + 1
            }
            Opcode::Print(n) => {
                for _ in 0..*n {
                    let obj = self.pop();
                    print!("{} ", obj);
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
                // self.globals[*index] = obj;
                if let Object::CompiledFunction {
                    start,
                    len: _,
                    num_locals: _,
                    num_parameters,
                } = obj
                {
                    self.closures[*index] = (start, num_parameters);
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
                let func = self.pop();
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
                let free = self.stack.split_off(self.sp - free_count);
                self.sp -= free_count;
                let (start, _) = self.closures[*index];
                for f in free.iter() {
                    self.push_free(f.clone());
                }
                self.registers.push((ip, self.free_index, *free_count));
                return start;
            }
            Opcode::GetFree(index) => {
                let obj = self.frees[self.free_index + *index - 1].clone();
                self.push(obj);
                ip + 1
            }
            Opcode::TailCall(_) => {
                unimplemented!("unimplemented opcode: {:?}", instruction);
            }
            Opcode::Return => {
                let result = self.pop();
                // self.pop_frame();
                self.push(result);
                // self.frees.pop();
                let (ip, arg_start, arg_len) = self.registers.pop().unwrap();
                self.free_index -= arg_len;
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

    pub fn push_free(&mut self, obj: Object) {
        if self.free_top > self.free_index {
            self.frees[self.free_index] = obj;
        } else {
            self.frees.push(obj);
            self.free_top += 1;
        }
        self.free_index += 1;
    }

    pub fn pop_free(&mut self) {
        let _ = self.frees[self.free_index - 1].clone();
        self.free_index -= 1;
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

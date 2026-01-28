use std::{process::exit, rc::Rc, vec};

use crate::{builtins::Builtins, objects::Object, opcode::Opcode};

pub struct VM<'a> {
    constants: Vec<Rc<Object>>,
    stack: Vec<Rc<Object>>,
    globals: Vec<Rc<Object>>,
    closures: Vec<(usize, usize)>,
    builtins: Builtins,
    sp: usize, // stack pointer
    main_start: usize,
    instructions: Vec<&'a Opcode>,
    registers: Vec<(usize, usize, usize)>, // ip, free_start, free_len
    free_start: usize,
    stack_top: usize,
}

fn nil() -> Rc<Object> {
    Rc::new(Object::Nil)
}
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl<'a> VM<'a> {
    pub fn new(ins: (usize, Vec<&Opcode>)) -> VM<'_> {
        let nil_obj = nil();
        VM {
            constants: Vec::new(),
            stack: vec![nil_obj.clone(); 1024],
            main_start: ins.0,
            instructions: ins.1,
            globals: vec![nil(); GLOBALS_SIZE],
            builtins: Builtins::new(),
            sp: 0,
            registers: Vec::with_capacity(1024),
            free_start: 0,
            closures: vec![(0, 0); GLOBALS_SIZE],
            stack_top: 1024,
        }
    }

    pub fn run(&mut self) -> Object {
        let mut ip = self.main_start;
        let l = self.instructions.len();
        println!("ip, {:?}, l: {:?}", ip, l);
        while ip < l {
            let instruction: &Opcode = self.instructions[ip];
            // println!("ip: {:?}, {:?}  {:?}, free_start: {:?}", ip, instruction, self.registers, self.free_start);
            ip = self.execute(instruction, ip, l);
        }

        if self.sp <= 0 {
            return (*nil()).clone();
        }
        (*self.pop()).clone()
    }

    fn push(&mut self, obj: Rc<Object>) {
        if self.sp >= self.stack_top {
            self.stack.push(obj);
           // println!("stack: {:?}", self.stack);
            self.stack_top += 1;
        } else {
            //println!("push: {:?}, sp: {:?}", self.stack.len(), self.sp);
            self.stack[self.sp] = obj;
        }
        self.sp += 1;
    }

    fn pop(&mut self) -> Rc<Object> {
        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    fn last(&mut self) -> Rc<Object> {
        self.stack[self.sp - 1].clone()
    }

    #[inline]
    fn execute(&mut self, instruction: &Opcode, ip: usize, l: usize) -> usize {
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
                self.stack[self.sp-1] = Rc::new(match (left.as_ref(), right.as_ref()) {
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
                });
                ip + 1
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if *obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    if ip < l {
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
                    if ip < l {
                        *pos
                    } else {
                        self.main_start + *pos
                    }
                } else {
                    ip + 1
                }
            }
            Opcode::Jump(pos) => {
                if ip < l {
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
                self.stack[self.sp - 1] = Rc::new(match obj.as_ref() {
                    Object::Number(n) => Object::Number(n.abs()),
                    _ => Object::Number(0.0),
                });
                ip + 1
            }
            Opcode::Nagetive => {
                let obj = self.last();
                self.stack[self.sp - 1] = Rc::new(match obj.as_ref() {
                    Object::Number(n) => Object::Number(-n),
                    _ => Object::Number(0.0),
                });
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
                match obj.as_ref() {
                    Object::CompiledFunction {
                        start,
                        len: _,
                        num_locals: _,
                        num_parameters,
                    } => {
                        self.closures[*index] = (*start, *num_parameters);
                    }
                    _ => {
                        self.globals[*index] = obj;
                    }
                }
                ip + 1
            }
            Opcode::GetBuiltin(index) => {
                let obj = self.builtins.get_by_index(*index);
                if obj.is_none() {
                    unimplemented!("builtin not found: {:?}", index);
                }
                self.push(Rc::new(obj.unwrap().clone()));
                ip + 1
            }
            Opcode::Call(n) => {
                let func = self.pop();
                // println!("----------> call: {:?}", func);
                match func.as_ref() {
                    Object::Builtin(_, _, f) => {
                        let args: Vec<Object> = self.stack[self.sp-n..self.sp]
                            .iter()
                            .map(|obj| (*obj.as_ref()).clone())
                            .collect();
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
            Opcode::ReturnValue => {
                //let result = self.pop();
                // self.pop_frame();
                let last = self.last();
                self.stack[self.free_start] = last;
                //self.stack.drain(self.free_start..self.sp - 1);
                //println!("-------------------> return: {:?}", self.stack.len());
                self.sp = self.free_start + 1;
                //self.push(result);
                // self.frees.pop();
                //println!("return ip: {:?}", ip)    ;
                let (ip, arg_start, _) = self.registers.pop().unwrap();
                //println!("return: ip: {:?}, stack: {:?}", ip + 1, self.stack);
                self.free_start = arg_start;
                ip + 1
            }
            Opcode::Return => {
                //let result = self.pop();
                // self.pop_frame();
                // let last = self.last();
                // self.stack[self.free_start] = last.clone();
                //self.stack.drain(self.free_start..self.sp - 1);
                //println!("-------------------> return: {:?}", self.stack.len());
                self.sp = self.free_start;
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
            Opcode::Index => {
                let right = &self.stack[self.sp - 1];
                self.sp -= 1;
                let left = &self.stack[self.sp - 1];
                match (left.as_ref(), right.as_ref()) {
                    (Object::Array(arr), Object::Index(index)) => {
                        if let Some(obj) = arr.get(*index as usize) {
                            self.push(obj.clone());
                        } else {
                            self.push(nil());
                        }
                    }
                    (Object::Array(arr), Object::Number(index)) => {
                        if let Some(obj) = arr.get(*index as usize) {
                            self.push(obj.clone());
                        } else {
                            self.push(nil());
                        }
                    }
                    _ => unimplemented!("Index operation not implemented for: {:?} and {:?}", left, right),
                }
                ip + 1
            }
            _ => unimplemented!("unimplemented opcode: {:?}", instruction),
        }
    }

    pub fn define_constants(&mut self, constants: Vec<Object>) {
        self.constants = constants.into_iter().map(|obj| Rc::new(obj)).collect();
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

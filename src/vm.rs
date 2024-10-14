use std::{cell::RefCell, mem, process::exit, rc::Rc, vec};

use crate::{builtins::Builtins, frame::Frame, objects::Object, opcode::Opcode};

pub struct VM<'a> {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    builtins: Builtins,
    sp: usize, // stack pointer
    frames: Vec<Rc<RefCell<Frame>>>,
    frame_index: usize,
    frame_pool: Vec<Rc<RefCell<Frame>>>,
    frame_pool_top: usize,
    // count: usize,
    //current_frame: Option<*mut Frame>,
    main_len: usize,
    // end_ip: usize,
    instructions: Vec<&'a Opcode>,
    frees: Vec<Object>,
    free_top: usize,
    free_index: usize,
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
            main_len: ins.0,
            instructions: ins.1,
            globals: vec![NIL; GLOBALS_SIZE],
            builtins: Builtins::new(),
            //end_ip: ins.0,
            sp: 0,
            frames: Vec::with_capacity(1024),
            frame_index: 0,
            // count: 0,
            frees: vec![],
            free_top: 0,
            free_index: 0,
            frame_pool: Vec::with_capacity(1024),
            frame_pool_top: 0,
            // current_frame: None,
        }
    }

    fn current_frame(&mut self) -> Option<Rc<RefCell<Frame>>> {
        if self.frame_index == 0 {
            return None;
        }
        self.frames[self.frame_index - 1].clone().into()
    }

    fn push_frame(&mut self, frame: Rc<RefCell<Frame>>) {
        self.frames.push(frame);
        self.frame_index += 1;
    }

    fn pop_frame(&mut self) {
        // let poped_frame = self.frames.pop();
        // let _ = self.frees.split_off(self.free_index - free_len);
        let frame = self.frames.pop().unwrap();
        //println!("??????????????? after pop ???????????????????\npop frame: {:?} {:?} {:?}", frame, self.frame_pool_top, self.frame_pool.len());
        self.frame_pool_top += 1;
        let _ = mem::replace(&mut self.frame_pool[self.frame_pool_top - 1], frame);
        self.frame_index -= 1;

        // if self.frame_index > 0 {
        //     self.end_ip = self.current_frame().unwrap().end_ip;
        // }
        // println!(
        //     "pop frame {:?}\n----------------------------------->",
        //     self.current_frame()
        // );
        // for (i, frame) in self.frames.iter().enumerate() {
        //     println!("{} {:?}", i, frame);
        // }
        // println!("-----------------");
    }

    pub fn run(&mut self) -> Object {
        for _ in 0..1024 {
            let frame = Frame {
                const_index: 0,
                is_main: false,
                ip: 0,
                end_ip: 0,
                base_pointer: 0,
                free_start: 0,
                free_len: 0,
                fress: vec![],
            };
            self.frame_pool.push(Rc::new(RefCell::new(frame)));
        }
        self.frame_pool_top = 1024;
        let frame = self.get_frame_from_pool();
        frame.borrow_mut().end_ip = self.main_len;
        frame.borrow_mut().const_index = 9999999;
        frame.borrow_mut().is_main = true;
        self.push_frame(frame);

        let mut count = 1;
        while let Some(frame) = self.current_frame() {
            // let f = frame.borrow_mut();
            // let mut ip = f.ip;
            // let end_ip = f.end_ip;
            let mut ip = frame.borrow_mut().ip;
            let end_ip = frame.borrow_mut().end_ip;
            let is_main = frame.borrow().is_main;

            while ip < end_ip {
                let instruction: &Opcode = self.instructions[ip];
                // println!("ip: {:?}, {:?} {:?}", ip, instruction, is_main);
                ip = self.execute(instruction, ip, is_main);
                match instruction {
                    Opcode::Closure(_, _) => {
                        break;
                    }
                    _ => {}
                }
            }
            frame.borrow_mut().ip = ip;
            //println!("--------------> ip: {:?} end_ip: {:?}, frames len: {:?} {:?}", ip, end_ip, self.frames, self.frames.len());
            if ip >= end_ip {
                if self.frames.is_empty() {
                    break;
                }
                if self.frame_index == 0 {
                    break;
                }
                self.pop_frame();
                // if self.frames.is_empty() {
                //     break;
                // }
            }
            if count > 10 {
                // break;
            }
            count += 1;
        }

        // println!("-----------> count: {:?}", self.count);
        if self.stack.is_empty() {
            return NIL;
        }
        self.pop()
    }

    fn get_frame_from_pool(&mut self) -> Rc<RefCell<Frame>> {
        let frame = self.frame_pool[self.frame_pool_top - 1].clone();
        self.frame_pool_top -= 1;
        frame
    }

    fn push(&mut self, obj: Object) {
        self.stack.push(obj);
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        // println!("pop: {:?}", self.stack);
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
                ip + 1
                // println!("self.stack: {:?}", self.stack);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    if is_main {
                        *pos
                    } else {
                        self.main_len + *pos
                    }
                }
            }
            Opcode::Exit(code) => {
                exit(*code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    if is_main {
                        *pos
                    } else {
                        self.main_len + *pos
                    }
                } else {
                    ip + 1
                }
            }
            Opcode::Jump(pos) => {
                if is_main {
                    *pos
                } else {
                    self.main_len + *pos
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
                self.globals[*index] = obj;
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
                let obj = self.globals[*index].clone();
                if let Object::CompiledFunction {
                    start,
                    len,
                    num_locals: _,
                    num_parameters: _,
                } = obj
                {
                    let frame = self.get_frame_from_pool();
                    frame.borrow_mut().const_index = *index;
                    frame.borrow_mut().ip = self.main_len + start;
                    frame.borrow_mut().end_ip = self.main_len + start + len;
                    frame.borrow_mut().base_pointer = self.main_len + start;
                    frame.borrow_mut().free_start = self.free_index;
                    frame.borrow_mut().free_len = *free_count;
                    frame.borrow_mut().fress = free.clone();
                    self.push_frame(frame);

                    // println!("------------------------ after push -------------------------------------");
                    // for f in self.frames.iter() {
                    //     println!("frame: {:?}", f);
                    // }
                    // println!("----------------------------+---------------------------------");
                }
                ip + 1
            }
            Opcode::GetFree(index) => {
                let obj = self.current_frame().unwrap().borrow().fress[*index].clone();
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

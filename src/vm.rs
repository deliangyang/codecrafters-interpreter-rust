use std::{mem, process::exit, vec};

use crate::{
    builtins::Builtins,
    callstack::CallStack,
    frame::{Frame, FramePool},
    objects::Object,
    opcode::Opcode,
};

pub struct VM<'a> {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    builtins: Builtins,
    sp: usize, // stack pointer
    frames: Vec<*mut Frame>,
    frame_index: usize,
    count: usize,
    //current_frame: Option<*mut Frame>,
    main_len: usize,
    end_ip: usize,
    instructions: Vec<&'a Opcode>,
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
            end_ip: ins.0,
            sp: 0,
            frames: Vec::with_capacity(1024),
            frame_index: 0,
            count: 0,
            // current_frame: None,
        }
    }

    pub fn run(&mut self) -> Object {
        let mut frame_pool = FramePool::new();
        let mut call_stack = CallStack::new(&mut frame_pool);

        let mut frame = call_stack.pool.get_frame();
        frame.end_ip = self.main_len;
        frame.const_index = 9999999;

        // 保存当前栈帧指针
        let frame_ptr: *mut Frame = &mut frame;
        call_stack.push_frame(frame_ptr);

        println!("run: {:?}", self.end_ip);
        // if let Some(f) = self.current_frame() {
        //     self.current_frame = f;
        // }
        // println!("run: {:?}", self.instructions);
        // println!(
        //     "run: {:?}",
        //     self.frames
        //         .get(self.frame_index - 1)
        //         .map(|&x| unsafe { &mut *x })
        //         .unwrap()
        // );
        while let Some(frame) = call_stack.pop_frame() {
            let mut ip = unsafe { (*frame).ip };
            let end_ip = unsafe { (*frame).end_ip };
            let is_main = unsafe { (*frame).is_main };
            let const_index = unsafe { (*frame).const_index };
            
            while ip < end_ip {
                unsafe {
                    println!("{} {} {} {} {}", end_ip, const_index, ip, end_ip, is_main);
                };
                let instruction = self.instructions[ip];
                println!("ip: {:?}, instruction: {:?}", ip, instruction);
                ip = self.execute(&mut call_stack, frame, instruction, ip);
                match instruction {
                    Opcode::Closure(_, _) => {
                        unsafe { (*frame).ip = ip }
                        break;
                    }
                    _ => {}
                }
            }
            // call_stack.pool.return_frame(unsafe {
            //     mem::replace(
            //         &mut *frame,
            //         Frame {
            //             const_index: 0,
            //             is_main: false,
            //             ip: 0,
            //             end_ip: 0,
            //             base_pointer: 0,
            //             frees: vec![],
            //         },
            //     )
            // });
        }
        // loop {
        //     let frame = self.get_frame();
        //     println!("frame: {:?}", frame);
        //     println!("frame: {:?}", frame);
        //     if self.frame_index == 0 && frame.ip() >= self.end_ip {
        //         break;
        //     }
        //     let mut ip = frame.ip();
        //     let end_ip = frame.get_end_ip();
        //     while ip < end_ip {
        //         let instruction = self.instructions[ip];
        //         println!("ip: {:?}, instruction: {:?}", ip, instruction);
        //         ip = self.execute(instruction, ip);
        //     }
        //     // let ip = self.ip();
        //     // if ip >= self.end_ip {
        //     //     if self.frame_index == 1 {
        //     //         break;
        //     //     }
        //     //     self.pop_frame();
        //     //     if self.frame_index == 0 {
        //     //         break;
        //     //     }
        //     //     continue;
        //     // }
        //     // let instruction = self.instructions[ip];
        //     // println!("ip: {:?}, instruction: {:?}", ip, instruction);
        //     // self.execute(instruction);
        // }

        println!("count: {:?}", self.count);
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

    fn execute(
        &mut self,
        call_stack: &mut CallStack,
        frame: *mut Frame,
        instruction: &Opcode,
        ip: usize,
    ) -> usize {
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
                ip + 1
            }
            Opcode::ReturnValue => {
                let obj = self.pop();
                self.pop_frame();
                self.push(obj);
                ip + 1
                // println!("self.stack: {:?}", self.stack);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    *pos
                }
            }
            Opcode::Exit(code) => {
                exit(*code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    *pos
                } else {
                    ip + 1
                }
            }
            Opcode::Jump(pos) => *pos,
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
                let (obj, free) = self.push_closure(*index, *free_count);
                if let Object::CompiledFunction {
                    start,
                    len,
                    num_locals: _,
                    num_parameters: _,
                } = obj
                {
                    let mut frame = Frame::new(
                        *index,
                        false,
                        self.main_len + start,
                        self.main_len + start + len,
                        self.main_len + start,
                        free,
                    );
                    call_stack.push_frame(&mut frame);
                    self.count += 1;
                }
                ip + 1
            }
            Opcode::GetFree(index) => {
                let free = unsafe {
                    (*frame).frees
                        .get(*index)
                        .map(|x| x.clone())
                        .unwrap_or(Object::Nil)
                };
                self.push(free);
                ip + 1
            }
            Opcode::TailCall(_) => {
                unimplemented!("unimplemented opcode: {:?}", instruction);
            }
            Opcode::Return => {
                let result = self.pop();
                self.pop_frame();
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

    pub fn define_constants(&mut self, constants: Vec<Object>) {
        self.constants = constants;
    }

    pub fn push_frame(&mut self, frame: *mut Frame) {
        println!(
            "push frame len: {:?}, index: {:?}",
            self.frames.len(),
            self.frame_index
        );
        if self.frames.len() > self.frame_index {
            let _ = mem::replace(&mut self.frames[self.frame_index], frame);
            // self.frames.truncate(self.frame_index);
        } else {
            self.frames.push(frame);
        }
        self.frame_index += 1;
        self.end_ip = self.current_frame().unwrap().get_end_ip();
        // println!(
        //     "push frame: {:?}",
        //     self.current_frame(),
        // );
    }

    pub fn current_frame(&mut self) -> Option<&mut Frame> {
        self.frames
            .get(self.frame_index - 1)
            .map(|&x| unsafe { &mut *x })
    }

    pub fn pop_frame(&mut self) {
        // let poped_frame = self.frames.pop();
        self.frame_index -= 1;
        if self.frame_index > 0 {
            self.end_ip = self.current_frame().unwrap().get_end_ip();
        }
        println!(
            "pop frame {:?}\n----------------------------------->",
            self.current_frame()
        );
        for (i, frame) in self.frames.iter().enumerate() {
            println!("{} {:?}", i, frame);
        }
        println!("-----------------");
    }

    // pub fn get_frame(&mut self) -> Frame {
    //     self.frames
    //         .get(self.frame_index - 1)
    //         .map(|&x| unsafe { &mut *x })
    //         .unwrap()
    //         .clone()
    // }

    pub fn incr_ip(&mut self) {
        self.current_frame().unwrap().incr_ip();
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.current_frame().unwrap().set_ip(ip);
    }

    pub fn ip(&mut self) -> usize {
        self.current_frame().unwrap().ip()
    }

    pub fn push_closure(&mut self, const_index: usize, free_count: usize) -> (Object, Vec<Object>) {
        // println!("push closure: {:?}, {:?}, stack: {:?}", const_index, free_count, self.stack);
        let free = self.stack.split_off(self.sp - free_count);
        self.sp -= free_count;
        // println!("free: {}, {} {:?}", self.sp, free_count, free);
        // let closure = Object::Closure {
        //     func: Rc::new(self.globals[const_index].clone()),
        //     free,
        // };
        // self.push(closure);
        // self.incr_ip();
        let value = self.globals[const_index].clone();
        // println!("value: {:?}", value);
        (value, free)
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

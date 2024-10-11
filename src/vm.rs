use std::{borrow::Borrow, cell::RefCell, collections::HashMap, process::exit, vec};

use crate::{
    builtins::Builtins,
    compiler::Stack,
    frame::{Frame, StackFrames},
    objects::Object,
    opcode::Opcode,
};

pub struct VM<'a> {
    constants: Vec<&'a Object>,
    stack: Vec<Object>,
    globals: Vec<&'a Object>,
    builtins: Builtins,
    sp: usize, // stack pointer
    count: usize,
    closures: HashMap<usize, Stack>,
    stack_frames: StackFrames,
    ins_len: usize,
    current_frame_index: usize,
    current_closure: Stack,
}

const NIL: Object = Object::Nil;
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        VM {
            constants: Vec::new(),
            stack: Vec::new(),
            globals: vec![&NIL; GLOBALS_SIZE],
            builtins: Builtins::new(),
            sp: 0,
            count: 0,
            closures: HashMap::new(),
            stack_frames: StackFrames::new(),
            ins_len: 0,
            current_frame_index: 0,
            current_closure: Stack{
                closure: vec![],
                num_locals: 0,
                num_parameters: 0,
            }
        }
    }

    pub fn set_closures(&mut self, closures: HashMap<usize, Stack>) {
        self.closures = closures;
    }

    pub fn run(&mut self, stack: Stack) -> Object {
        self.current_frame_index = 999999;
        self.stack_frames.push_frame(Frame::new(
            self.current_frame_index,
            true,
            stack.num_locals,
            0,
            vec![],
        ));
        self.closures.insert(self.current_frame_index, stack);
        self.current_closure = self.closures.get(&self.current_frame_index).unwrap();

        loop {
            let ip = self.stack_frames.get_ip();
            if ip >= self.ins_len {
                if self.stack_frames.is_end() {
                    break;
                }
                continue;
            }
            self.execute(self.current_closure.closure[ip].clone());
        }

        println!("count: {:?}", self.count);
        if self.stack.is_empty() {
            return Object::Nil;
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

    fn operate(&mut self, op: Opcode) -> Object {
        let right = self.pop();
        let left = self.pop();
        let result = match (left, right) {
            (Object::Number(l), Object::Number(r)) => match op {
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
        return result;
    }

    #[inline]
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
                let result = self.operate(instruction);
                self.push(result);
                self.stack_frames.inc_ip();
            }
            Opcode::ReturnValue => {
                let obj = self.pop();
                self.stack_frames.inc_ip();
                self.stack_frames.pop_frame();
                self.push(obj);
                // println!("self.stack: {:?}", self.stack);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    self.stack_frames.set_ip(pos);
                }
            }
            Opcode::Exit(code) => {
                exit(code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    self.stack_frames.set_ip(pos);
                } else {
                    self.stack_frames.inc_ip();
                }
            }
            Opcode::Jump(pos) => {
                self.stack_frames.set_ip(pos);
            }
            Opcode::LoadConstant(index) => {
                self.push(self.constants[index].to_owned());
                self.stack_frames.inc_ip();
            }
            Opcode::Pop => {
                self.pop();
                self.stack_frames.inc_ip();
            }
            Opcode::Abs => {
                let x = self.pop();
                let obj = x.borrow();
                let number = match obj {
                    Object::Number(n) => n.abs(),
                    _ => 0.0,
                };
                self.push(Object::Number(number));
                self.stack_frames.inc_ip();
            }
            Opcode::Nagetive => {
                let x = self.pop();
                let obj = x.borrow();
                let number = Object::Number(match obj {
                    Object::Number(n) => -(*n),
                    _ => 0.0,
                });
                self.push(number);
                self.stack_frames.inc_ip();
            }
            Opcode::Print(n) => {
                for _ in 0..n {
                    let obj = self.pop();
                    print!("{} ", obj);
                }
                self.stack_frames.inc_ip();
            }
            Opcode::DefineGlobal(s) => {
                let obj = self.pop();
                println!("{} = {:?}", s, obj);
                self.stack_frames.inc_ip();
            }
            Opcode::GetGlobal(index) => {
                let result = self.globals[index].clone();
                self.push(result);
                self.stack_frames.inc_ip();
            }
            Opcode::SetGlobal(_) => {
                unimplemented!("unimplemented opcode: {:?}", instruction);
                //let obj = self.pop();
                //self.globals[*index] = &obj;
                // self.stack_frames.inc_ip();
            }
            // Opcode::SetClouse(constant_index, synbom_index) => {
            //     // let obj = self.pop();
            //     //self.closures[*index] = &obj;
            //     let obj = self.constants[constant_index];
            //     self.globals[synbom_index] = obj;
            //     self.stack_frames.inc_ip();
            // }
            Opcode::GetBuiltin(index) => {
                let obj = self.builtins.get_by_index(index);
                if obj.is_none() {
                    unimplemented!("builtin not found: {:?}", index);
                }
                self.push(obj.unwrap());
                self.stack_frames.inc_ip();
            }
            Opcode::Call(n) => {
                let x = self.pop();
                let func = x.borrow();
                // println!("----------> call: {:?}", func);
                match func {
                    Object::Builtin(_, _, f) => {
                        let args = self.stack.split_off(self.sp - n);
                        self.sp -= n;
                        let _ = f(args);
                        self.stack_frames.inc_ip();
                        //self.push(result);
                    }
                    _ => unimplemented!("unimplemented function: {:?}", func),
                }
            }
            Opcode::Closure(index, free_count) => self.push_closure(index, free_count),
            Opcode::GetFree(index) => {
                self.push(self.stack_frames.get_free(index));
                self.stack_frames.inc_ip();
            }
            Opcode::TailCall(_) => {
                unimplemented!("unimplemented opcode: {:?}", instruction);
            }
            Opcode::Return => {
                let result = self.pop();
                self.stack_frames.inc_ip();
                self.stack_frames.pop_frame();
                self.push(result);
            }
            Opcode::SetLocal(_index) => {
                // let obj = self.pop();
                // println!("set local: {:?}", index);
                self.stack_frames.inc_ip();
            }
            Opcode::GetLocal(_index) => {
                self.stack_frames.inc_ip();
                //println!("get local: {:?}", index);
                // let frame = self.current_frame();
                // let obj = frame.get_local(index);
                // self.push(obj);
            }
            _ => unimplemented!("unimplemented opcode: {:?}", instruction),
        }
    }

    pub fn define_constants(&mut self, constants: Vec<&'a Object>) {
        self.constants = constants;
    }

    // pub fn push_frame(&mut self, frame: Frame) {
    //     self.frames.borrow_mut().push(frame);
    //     self.frame_index += 1;
    //     self.current_frame = self.current_frame();
    // }

    // pub fn current_frame(&mut self) -> Frame {
    //     self.frames.borrow_mut()[self.frame_index - 1].clone()
    // }

    // pub fn pop_frame(&mut self) {
    //     self.frames.borrow_mut().pop();
    //     self.frame_index -= 1;
    //     if self.frame_index > 0 {
    //         self.current_frame = self.current_frame();
    //     }
    // }

    pub fn push_closure(&mut self, const_index: usize, free_count: usize) {
        // println!("push closure: {:?}, {:?}, stack: {:?}", const_index, free_count, self.stack);
        let free = self.stack.split_off(self.sp - free_count);
        self.sp -= free_count;
        // println!("free: {}, {} {:?}", self.sp, free_count, free);
        // let closure = Object::Closure {
        //     func: Rc::new(self.globals[const_index].clone()),
        //     free,
        // };
        // self.push(closure);
        // self.stack_frames.inc_ip();
        let ip = self.stack_frames.get_ip();
        self.stack_frames.inc_ip();
        let frame = Frame::new(const_index, false, 0, ip - free_count, free);
        self.stack_frames.push_frame(frame);
        self.count += 1;
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
        vm.define_constants(compiler.constants.iter().map(|x| x).collect());
        vm.set_closures(compiler.closures);
        vm.run(Stack {
            closure: compiler.instructions,
            num_locals: 0,
            num_parameters: 0,
        })
    }
}

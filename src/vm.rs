use std::{borrow::Borrow, cell::RefCell, process::exit, rc::Rc, vec};

use crate::{builtins::Builtins, container::Container, frame::Frame, objects::Object, opcode::Opcode};

pub struct VM<'a> {
    constants: Vec<&'a Object>,
    stack: Vec<Object>,
    globals: Vec<Rc<RefCell<Object>>>,
    builtins: Builtins,
    sp: usize, // stack pointer
    frames: RefCell<Vec<Frame<'a>>>,
    frame_index: usize,
    count: usize,
    current_frame: Frame<'a>,
    current_instructions: Vec<&'a Opcode>,
    main_closure: Vec<&'a Opcode>,
    closures: Vec<Vec<&'a Opcode>>,
}

const NIL: Object = Object::Nil;
// const TRUE: Object = Object::Boolean(true);
// const FALSE: Object = Object::Boolean(false);

const GLOBALS_SIZE: usize = 65536;

impl<'a> VM<'a> {
    pub fn new() -> VM<'a> {
        let frame: Frame<'a> = Frame::new(0, true, 0, 0, vec![]);
        VM {
            constants: Vec::new(),
            stack: Vec::new(),
            globals: vec![Rc::new(RefCell::new(NIL)); GLOBALS_SIZE],
            builtins: Builtins::new(),
            sp: 0,
            frames: RefCell::new(Vec::new()),
            frame_index: 0,
            count: 0,
            current_frame: frame,
            current_instructions: vec![],
            main_closure: vec![],
            closures: vec![vec![]; 2048],
        }
    }

    fn get_instructions(&self, cl: &'a Object) -> Vec<&'a Opcode> {
        return match cl {
            Object::Closure { func, .. } => match func.as_ref() {
                Object::CompiledFunction { instructions, .. } => {
                    instructions.iter().map(|x| x).collect()
                }
                _ => vec![],
            },
            Object::CompiledFunction { instructions, .. } => {
                let mut ins = vec![];
                for i in instructions.iter() {
                    ins.push(i);
                }
                ins.to_vec()
            }
            _ => vec![],
        };
    }

    pub fn run(&mut self, instructions: Vec<&'a Opcode>) -> Object {
        self.main_closure = instructions.iter().map(|x| *x).collect();
        self.current_instructions = instructions.iter().map(|x| *x).collect();
        let frame: Frame<'a> = Frame::new(0, true, 0, 0, vec![]);
        self.push_frame(frame);
        self.current_frame = self.current_frame();

        loop {
            let l = self.current_instructions.len();
            let ip = self.ip();
            if ip >= l {
                if self.frames.borrow_mut().len() == 1 {
                    break;
                }
                self.pop_frame();
                if self.frames.borrow_mut().is_empty() {
                    break;
                }
                continue;
            }
            let instruction = self.current_instructions.get(ip);
            // println!("ip: {:?}, instruction: {:?}", ip, instruction);
            if instruction.is_none() {
                break;
            }
            self.execute(instruction.unwrap());
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

    fn operate(&mut self, op: &Opcode) -> Object {
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

    fn execute(&mut self, instruction: &Opcode) {
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
                self.incr_ip();
            }
            Opcode::ReturnValue => {
                let obj = self.pop();
                self.incr_ip();
                self.pop_frame();
                self.push(obj);
                // println!("self.stack: {:?}", self.stack);
            }
            Opcode::Assert(pos) => {
                let obj = self.pop();
                if obj == Object::Boolean(false) {
                    panic!("assert failed");
                } else {
                    self.set_ip(*pos);
                }
            }
            Opcode::Exit(code) => {
                exit(*code as i32);
            }
            Opcode::JumpIfFalse(pos) => {
                let condition = self.pop();
                if condition == Object::Boolean(false) {
                    self.set_ip(*pos);
                } else {
                    self.incr_ip();
                }
            }
            Opcode::Jump(pos) => {
                self.set_ip(*pos);
            }
            Opcode::LoadConstant(index) => {
                self.push(self.constants[*index].clone());
                self.incr_ip();
            }
            Opcode::Pop => {
                self.pop();
                self.incr_ip();
            }
            Opcode::Abs => {
                let x = self.pop();
                let obj = x.borrow();
                let number = match obj {
                    Object::Number(n) => n.abs(),
                    _ => 0.0,
                };
                self.push(Object::Number(number));
                self.incr_ip();
            }
            Opcode::Nagetive => {
                let x = self.pop();
                let obj = x.borrow();
                let number = Object::Number(match obj {
                    Object::Number(n) => -(*n),
                    _ => 0.0,
                });
                self.push(number);
                self.incr_ip();
            }
            Opcode::Print(n) => {
                for _ in 0..*n {
                    let obj = self.pop();
                    print!("{} ", obj);
                }
                self.incr_ip();
            }
            Opcode::DefineGlobal(s) => {
                let obj = self.pop();
                println!("{} = {:?}", s, obj);
                self.incr_ip();
            }
            Opcode::GetGlobal(index) => {
                let result = self.globals[*index].clone();
                let obj = result.borrow_mut();
                self.push(obj.to_owned());
                self.incr_ip();
            }
            Opcode::SetGlobal(index) => {
                let obj = self.pop();
                self.globals[*index] = Rc::new(RefCell::new(obj));
                self.incr_ip();
            }
            Opcode::GetBuiltin(index) => {
                let obj = self.builtins.get_by_index(*index);
                if obj.is_none() {
                    unimplemented!("builtin not found: {:?}", index);
                }
                self.push(obj.unwrap());
                self.incr_ip();
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
                        self.incr_ip();
                        //self.push(result);
                    }
                    _ => unimplemented!("unimplemented function: {:?}", func),
                }
            }
            Opcode::Closure(index, free_count) => self.push_closure(*index, *free_count),
            Opcode::GetFree(index) => {
                let frame = self.current_frame();
                self.push(frame.get_free(*index).clone());
                self.incr_ip();
            }
            Opcode::TailCall(_) => {
                unimplemented!("unimplemented opcode: {:?}", instruction);
            }
            Opcode::Return => {
                let result = self.pop();
                self.incr_ip();
                self.pop_frame();
                self.push(result);
            }
            Opcode::SetLocal(_index) => {
                // let obj = self.pop();
                // println!("set local: {:?}", index);
                self.incr_ip();
            }
            Opcode::GetLocal(_index) => {
                self.incr_ip();
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

    pub fn push_frame(&mut self, frame: Frame<'a>) {
        self.frames.borrow_mut().push(frame);
        self.frame_index += 1;
        self.current_frame = self.current_frame();
        if self.current_frame.is_main() {
            self.current_instructions = self.main_closure.iter().map(|x| *x).collect();
        } else {
            let instructions = self.closures.get(self.current_frame.get_index());
            if instructions.unwrap().len() > 0 {
                self.current_instructions = instructions.unwrap().iter().map(|x| *x).collect();
            } else {
                let object = self.globals[self.current_frame.get_index()];
                let ins = self.get_instructions(object);
                // println!("closure: {:?}", object);
                self.closures[self.current_frame.get_index()] = ins.iter().map(|x| *x).collect();
                let instructions = self.closures.get(self.current_frame.get_index());
                self.current_instructions = instructions.unwrap().iter().map(|x| *x).collect();
            }
        }
        // println!("push frame: {:?}", self.current_instructions);
    }

    pub fn current_frame(&mut self) -> Frame<'a> {
        self.frames.borrow_mut()[self.frame_index - 1].clone()
    }

    pub fn pop_frame(&mut self) {
        self.frames.borrow_mut().pop();
        self.frame_index -= 1;
        if self.frame_index > 0 {
            self.current_frame = self.current_frame();
            if self.current_frame.is_main() {
                self.current_instructions = self.main_closure.iter().map(|x| *x).collect();
            } else {
                self.current_instructions = self.closures[self.current_frame.get_index()]
                    .iter()
                    .map(|x| *x)
                    .collect();
            }
        }
    }

    pub fn incr_ip(&mut self) {
        self.frames.borrow_mut()[self.frame_index - 1].incr_ip();
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.frames.borrow_mut()[self.frame_index - 1].set_ip(ip);
    }

    pub fn ip(&mut self) -> usize {
        self.frames.borrow_mut()[self.frame_index - 1].ip()
    }

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
        // self.incr_ip();
        let ip = self.ip();
        self.incr_ip();
        let frame = Frame::new(
            const_index,
            false,
            0,
            ip - free_count,
            free.iter().map(|x| x).collect(),
        );
        self.push_frame(frame);
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
        vm.define_constants(compiler.constants);
        vm.run(compiler.instructions)
    }
}

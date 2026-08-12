use std::{collections::HashMap, process::exit, vec};

use crate::{builtins::Builtins, objects::Object, opcode::Opcode};

pub struct VM {
    constants: Vec<Object>,
    stack: [Object; 65536],
    globals: Vec<Object>,
    closures: Vec<(usize, usize)>,
    builtins: Builtins,
    sp: usize,
    main_start: usize,
    // Flat instruction format: (opcode, arg1, arg2)
    ops: Vec<u8>,
    args1: Vec<usize>,
    args2: Vec<usize>,
    // registers: (return_ip, free_start, free_len, memo_key)
    // memo_key: (closure_index, arg_value) if this frame is memoized, else (usize::MAX, 0)
    registers: [(usize, usize, usize, (usize, u64)); 1024],
    reg_sp: usize,
    free_start: usize,
    // Memoization cache: (closure_index, arg) -> result
    memo_cache: HashMap<(usize, u64), Object>,
}

const NIL: Object = Object::Nil;
const GLOBALS_SIZE: usize = 65536;

// Opcode tags
const OP_ADD: u8 = 0;
const OP_MINUS: u8 = 34;
const OP_MULTIPLY: u8 = 37;
const OP_DIVIDE: u8 = 36;
const OP_MOD: u8 = 35;
const OP_LESS_THAN: u8 = 31;
const OP_GREATER_THAN: u8 = 30;
const OP_EQUAL_EQUAL: u8 = 44;
const OP_ASSERT: u8 = 43;
const OP_EXIT: u8 = 45;
const OP_JUMP_IF_FALSE: u8 = 28;
const OP_JUMP: u8 = 7;
const OP_LOAD_CONSTANT: u8 = 1;
const OP_POP: u8 = 3;
const OP_ABS: u8 = 4;
const OP_NAGETIVE: u8 = 5;
const OP_PRINT: u8 = 2;
const OP_DEFINE_GLOBAL: u8 = 42;
const OP_GET_GLOBAL: u8 = 11;
const OP_SET_GLOBAL: u8 = 10;
const OP_GET_BUILTIN: u8 = 14;
const OP_CALL: u8 = 18;
const OP_CLOSURE: u8 = 15;
const OP_TAIL_CALL: u8 = 19;
const OP_MEMO_CALL: u8 = 46;
const OP_GET_FREE: u8 = 16;
const OP_RETURN_VALUE: u8 = 9;
const OP_RETURN: u8 = 20;
const OP_SET_LOCAL: u8 = 12;
const OP_GET_LOCAL: u8 = 13;

impl VM {
    pub fn new(ins: (usize, Vec<Opcode>)) -> VM {
        let (main_start, opcodes) = ins;
        let mut ops = Vec::with_capacity(opcodes.len());
        let mut args1 = Vec::with_capacity(opcodes.len());
        let mut args2 = Vec::with_capacity(opcodes.len());

        for op in opcodes {
            ops.push(op.discriminant());
            args1.push(op.arg());
            args2.push(op.arg2());
        }

        VM {
            constants: Vec::new(),
            stack: [NIL; 65536],
            main_start,
            ops,
            args1,
            args2,
            globals: vec![NIL; GLOBALS_SIZE],
            builtins: Builtins::new(),
            sp: 0,
            registers: [(0, 0, 0, (usize::MAX, 0)); 1024],
            reg_sp: 0,
            free_start: 0,
            closures: vec![(0, 0); GLOBALS_SIZE],
            memo_cache: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> Object {
        let mut ip = self.main_start;
        let l = self.ops.len();
        while ip < l {
            let op = self.ops[ip];
            let arg = self.args1[ip];
            let arg2 = self.args2[ip];
            ip = match op {
                OP_ADD => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(*l + *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_MINUS => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(*l - *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_MULTIPLY => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(*l * *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_DIVIDE => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(*l / *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_MOD => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Number(*l % *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_LESS_THAN => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(*l < *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_GREATER_THAN => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(*l > *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_EQUAL_EQUAL => {
                    let right = &self.stack[self.sp - 1];
                    self.sp -= 1;
                    let left = &self.stack[self.sp - 1];
                    self.stack[self.sp - 1] = match (left, right) {
                        (Object::Number(l), Object::Number(r)) => Object::Boolean(*l == *r),
                        _ => Object::Nil,
                    };
                    ip + 1
                }
                OP_ASSERT => {
                    let obj = self.pop();
                    if *obj == Object::Boolean(false) {
                        panic!("assert failed");
                    } else {
                        let is_main = ip >= self.main_start;
                        if !is_main { arg } else { self.main_start + arg }
                    }
                }
                OP_EXIT => {
                    exit(arg as i32);
                }
                OP_JUMP_IF_FALSE => {
                    let condition = self.pop();
                    if *condition == Object::Boolean(false) {
                        let is_main = ip >= self.main_start;
                        if !is_main { arg } else { self.main_start + arg }
                    } else {
                        ip + 1
                    }
                }
                OP_JUMP => {
                    let is_main = ip >= self.main_start;
                    if !is_main { arg } else { self.main_start + arg }
                }
                OP_LOAD_CONSTANT => {
                    self.push(self.constants[arg].clone());
                    ip + 1
                }
                OP_POP => {
                    self.pop();
                    ip + 1
                }
                OP_ABS => {
                    let obj = self.last();
                    self.stack[self.sp - 1] = Object::Number(match obj {
                        Object::Number(n) => n.abs(),
                        _ => 0.0,
                    });
                    ip + 1
                }
                OP_NAGETIVE => {
                    let obj = self.last();
                    self.stack[self.sp - 1] = Object::Number(match obj {
                        Object::Number(n) => -n,
                        _ => 0.0,
                    });
                    ip + 1
                }
                OP_PRINT => {
                    for _ in 0..arg {
                        let obj = self.pop();
                        print!("{} ", obj);
                    }
                    ip + 1
                }
                OP_DEFINE_GLOBAL => {
                    let obj = self.pop();
                    println!("global = {:?}", obj);
                    ip + 1
                }
                OP_GET_GLOBAL => {
                    self.push(self.globals[arg].clone());
                    ip + 1
                }
                OP_SET_GLOBAL => {
                    let obj = self.pop().clone();
                    match &obj {
                        Object::CompiledFunction { start, num_parameters, .. } => {
                            self.closures[arg] = (*start, *num_parameters);
                        }
                        _ => {
                            self.globals[arg] = obj;
                        }
                    }
                    ip + 1
                }
                OP_GET_BUILTIN => {
                    let obj = self.builtins.get_by_index(arg);
                    if obj.is_none() {
                        unimplemented!("builtin not found: {:?}", arg);
                    }
                    self.push(obj.unwrap().clone());
                    ip + 1
                }
                OP_CALL => {
                    let func = self.pop().clone();
                    match func {
                        Object::Builtin(_, _, f) => {
                            let args = self.stack[self.sp - arg..self.sp].to_vec();
                            self.sp -= arg;
                            let _ = f(args);
                            ip + 1
                        }
                        _ => unimplemented!("unimplemented function: {:?}", func),
                    }
                }
                OP_CLOSURE => {
                    let (start, _) = self.closures[arg];
                    self.registers[self.reg_sp] = (ip, self.free_start, arg2, (usize::MAX, 0));
                    self.reg_sp += 1;
                    self.free_start = self.sp - arg2;
                    start
                }
                OP_TAIL_CALL => {
                    // Tail call: reuse current frame instead of pushing new one
                    let (start, _) = self.closures[arg];
                    // Don't increment reg_sp - reuse current frame
                    // Just update free_start for new arguments
                    self.free_start = self.sp - arg2;
                    start
                }
                OP_MEMO_CALL => {
                    // Memoized call: check cache before executing
                    let (start, _) = self.closures[arg];

                    // Check if we have a cached result for this call
                    if arg2 == 1 {
                        let arg_val = self.stack[self.sp - 1].clone();
                        if let Object::Number(n) = arg_val {
                            let key = (arg, n as u64);
                            if let Some(cached) = self.memo_cache.get(&key) {
                                // Cache hit: replace argument with result and skip function
                                self.stack[self.sp - 1] = cached.clone();
                                ip + 1
                            } else {
                                // Cache miss: record the key for later storage in registers
                                self.registers[self.reg_sp] = (ip, self.free_start, arg2, key);
                                self.reg_sp += 1;
                                self.free_start = self.sp - arg2;
                                start
                            }
                        } else {
                            // Non-numeric argument, can't memoize
                            self.registers[self.reg_sp] = (ip, self.free_start, arg2, (usize::MAX, 0));
                            self.reg_sp += 1;
                            self.free_start = self.sp - arg2;
                            start
                        }
                    } else {
                        // Multiple arguments, can't memoize with current scheme
                        self.registers[self.reg_sp] = (ip, self.free_start, arg2, (usize::MAX, 0));
                        self.reg_sp += 1;
                        self.free_start = self.sp - arg2;
                        start
                    }
                }
                OP_GET_FREE => {
                    let obj = self.stack[self.free_start + arg].clone();
                    self.push(obj);
                    ip + 1
                }
                OP_RETURN_VALUE => {
                    let result = std::mem::replace(&mut self.stack[self.sp - 1], NIL);

                    // Pop the register and check if this is a memoized call
                    self.reg_sp -= 1;
                    let (ret_ip, arg_start, _, memo_key) = self.registers[self.reg_sp];

                    // If memo_key is valid (not usize::MAX), store the result in cache
                    if memo_key.0 != usize::MAX {
                        self.memo_cache.insert(memo_key, result.clone());
                    }

                    self.stack[self.free_start] = result;
                    self.sp = self.free_start + 1;
                    self.free_start = arg_start;
                    ret_ip + 1
                }
                OP_RETURN => {
                    self.sp = self.free_start;
                    self.reg_sp -= 1;
                    let (ret_ip, arg_start, _, _) = self.registers[self.reg_sp];
                    self.free_start = arg_start;
                    ret_ip + 1
                }
                OP_SET_LOCAL => ip + 1,
                OP_GET_LOCAL => ip + 1,
                _ => unimplemented!("unimplemented opcode: {}", op),
            };
        }

        if self.sp <= 0 {
            return NIL;
        }
        self.pop().clone()
    }

    fn push(&mut self, obj: Object) {
        self.stack[self.sp] = obj;
        self.sp += 1;
    }

    fn pop(&mut self) -> &Object {
        self.sp -= 1;
        &self.stack[self.sp]
    }

    fn last(&mut self) -> &Object {
        &self.stack[self.sp - 1]
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
        let mut vm = VM::new((l, codes));
        vm.define_constants(compiler.constants);
        vm.run()
    }
}

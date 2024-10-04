use crate::{objects::Object, opcode::Opcode};


#[derive(Debug, Clone)]
pub struct Frame {
    cl: Object,
    ip: usize,
    base_pointer: usize,
}

impl Frame {
    pub fn new(cl: Object, bp: usize, ip: usize) -> Frame {
        Frame {
            cl,
            ip,
            base_pointer: bp,
        }
    }

    pub fn instructions(&mut self) -> Vec<Opcode> {
        match self.cl.clone() {
            Object::CompiledFunction { instructions, .. } => instructions,
            _ => panic!("instructions called on non-compiled function"),
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
    }

    pub fn base_pointer(&self) -> usize {
        self.base_pointer
    }

    pub fn set_base_pointer(&mut self, bp: usize) {
        self.base_pointer = bp;
    }

    pub fn cl(&self) -> Object {
        self.cl.clone()
    }

    pub fn num_locals(&self) -> usize {
        match self.cl.clone() {
            Object::CompiledFunction { num_locals, .. } => num_locals,
            _ => panic!("num_locals called on non-compiled function"),
        }
    }

    pub fn num_parameters(&self) -> usize {
        match self.cl.clone() {
            Object::CompiledFunction { num_parameters, .. } => num_parameters,
            _ => panic!("num_parameters called on non-compiled function"),
        }
    }

    pub fn push(&mut self) {
        self.ip += 1;
    }

    pub fn pop(&mut self) {
        self.ip -= 1;
    }

    pub fn last_instruction(&mut self) -> Opcode {
        let ins = self.instructions();
        ins[self.ip - 1].clone()
    }

    pub fn next_instruction(&mut self) -> Opcode {
        let ins = self.instructions();
        ins[self.ip].clone()
    }
}
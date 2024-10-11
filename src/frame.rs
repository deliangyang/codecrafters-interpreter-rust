use crate::objects::Object;

#[derive(Debug, Clone)]
pub struct Frame {
    const_index: usize,
    is_main: bool,
    ip: usize,           // ip is the index of the instruction to be executed
    end_ip: usize,
    base_pointer: usize, // base_pointer is the index of the first local variable in the stack
    frees: Vec<Object>,  // frees is a vector of free variables
}

impl Frame {
    pub fn new(
        const_index: usize,
        is_main: bool,
        ip: usize,
        end_ip: usize,
        base_pointer: usize,
        frees: Vec<Object>,
    ) -> Frame {
        // println!("base_pointer: {}", base_pointer);
        Frame {
            const_index,
            is_main,
            ip,
            end_ip,
            base_pointer,
            frees,
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = self.base_pointer + ip;
        // println!("ip: {} base_pointer: {}", self.ip, self.base_pointer);
    }

    pub fn incr_ip(&mut self) {
        self.ip += 1;
    }

    pub fn base_pointer(&self) -> usize {
        self.base_pointer
    }

    pub fn set_base_pointer(&mut self, bp: usize) {
        self.base_pointer = bp;
    }

    pub fn push(&mut self) {
        self.ip += 1;
    }

    pub fn pop(&mut self) {
        self.ip -= 1;
    }

    pub fn get_index(&self) -> usize {
        self.const_index
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn get_frees(&self) -> Vec<Object> {
        self.frees.clone()
    }

    pub fn get_free(&self, index: usize) -> Object {
        self.frees[index].clone()
    }

    pub fn get_end_ip(&self) -> usize {
        self.end_ip
    }
}

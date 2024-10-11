use crate::objects::Object;

#[derive(Debug, Clone)]
pub struct Frame {
    const_index: usize,
    is_main: bool,
    ip: usize,           // ip is the index of the instruction to be executed
    base_pointer: usize, // base_pointer is the index of the first local variable in the stack
    frees: Vec<Object>,  // frees is a vector of free variables
}

impl Frame {
    pub fn new(
        const_index: usize,
        is_main: bool,
        ip: usize,
        base_pointer: usize,
        frees: Vec<Object>,
    ) -> Frame {
        Frame {
            const_index,
            is_main,
            ip,
            base_pointer,
            frees,
        }
    }

    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.ip = ip;
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
}

pub struct StackFrames {
    frames: Vec<Frame>,
    frame_index: usize,
}

impl StackFrames {
    pub fn new() -> StackFrames {
        StackFrames {
            frames: Vec::with_capacity(1024),
            frame_index: 0,
        }
    }

    pub fn push_frame(&mut self, frame: Frame) {
        self.frames.push(frame);
        self.frame_index += 1;
    }

    pub fn pop_frame(&mut self) -> Option<Frame> {
        let frame = self.frames.pop();
        self.frame_index -= 1;
        frame
    }

    pub fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index - 1]
    }

    pub fn inc_ip(&mut self) {
        self.frames[self.frame_index - 1].incr_ip();
    }

    pub fn dec_ip(&mut self) {
        self.frames[self.frame_index - 1].pop();
    }

    pub fn set_ip(&mut self, ip: usize) {
        self.frames[self.frame_index - 1].set_ip(ip);
    }

    pub fn get_ip(&self) -> usize {
        self.frames[self.frame_index - 1].ip()
    }

    pub fn get_free(&self, index: usize) -> Object {
        self.frames[self.frame_index - 1].get_free(index)
    }

    pub fn is_end(&self) -> bool {
        self.frame_index == 0
    }
}

use crate::objects::Object;

#[derive(Debug, Clone)]
pub struct Frame {
    pub const_index: usize,
    pub is_main: bool,
    pub ip: usize, // ip is the index of the instruction to be executed
    pub end_ip: usize,
    pub base_pointer: usize, // base_pointer is the index of the first local variable in the stack
    pub free_start: usize,   // free_start is the index of the first free variable
    pub free_len: usize,     // free_len is the number of free variables
    pub fress: Vec<Object>,  // fress is the list of free variables
}

impl Frame {
    pub fn new(
        const_index: usize,
        is_main: bool,
        ip: usize,
        end_ip: usize,
        base_pointer: usize,
        free_start: usize,
        free_len: usize,
        frees: Vec<Object>,
    ) -> Frame {
        // println!("base_pointer: {}", base_pointer);
        Frame {
            const_index,
            is_main,
            ip,
            end_ip,
            base_pointer,
            free_start,
            free_len,
            fress: frees,
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

    pub fn get_end_ip(&self) -> usize {
        self.end_ip
    }
}

// impl Drop for Frame {
//     fn drop(&mut self) {
//         println!("----------------------------> Dropping frame: {:?}", self);
//     }
// }

pub struct FramePool {
    pub pool: Vec<Frame>,
}

impl FramePool {
    pub fn new() -> Self {
        FramePool { pool: Vec::new() }
    }

    pub fn get_frame(&mut self) -> Frame {
        self.pool.pop().unwrap_or_else(|| Frame {
            const_index: 0,
            is_main: false,
            ip: 0,
            end_ip: 0,
            base_pointer: 0,
            free_start: 0,
            free_len: 0,
            fress: Vec::new(),
        })
    }

    pub fn return_frame(&mut self, frame: Frame) {
        self.pool.push(frame);
        // for f in self.pool.iter() {
        //     println!("++++++++++++++++++++++++++++++frame: {:?}", f);
        // }
    }
}

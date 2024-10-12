use crate::frame::{Frame, FramePool};

pub struct CallStack<'a> {
    stack: Vec<*mut Frame>,
    pub pool: &'a mut FramePool,
}

impl<'a> CallStack<'a> {
    pub fn new(pool: &'a mut FramePool) -> Self {
        CallStack {
            stack: Vec::with_capacity(1024),
            pool,
        }
    }

    pub fn push_frame(&mut self, frame: *mut Frame) {
        self.stack.push(frame);
    }

    pub fn pop_frame(&mut self) -> Option<*mut Frame> {
        self.stack.pop()
    }

    pub fn current_frame(&self) -> Option<&mut Frame> {
        self.stack.last().map(|&ptr| unsafe { &mut *ptr })
    }

    pub fn get_pool(&mut self) -> &mut FramePool {
        self.pool
    }

}

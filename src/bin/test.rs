use std::mem;

#[derive(Debug)]
struct Frame {
    local_vars: Vec<X>, // 局部变量
    return_address: usize, // 返回地址
    // 其他必要的上下文信息
}

struct FramePool {
    pool: Vec<Frame>,
}

impl FramePool {
    fn new() -> Self {
        FramePool { pool: Vec::new() }
    }

    fn get_frame(&mut self) -> Frame {
        self.pool.pop().unwrap_or_else(|| Frame {
            local_vars: vec![], // 初始化局部变量
            return_address: 0,
        })
    }

    fn return_frame(&mut self, frame: Frame) {
        self.pool.push(frame);
    }
}

struct CallStack<'a> {
    stack: Vec<*mut Frame>,
    pool: &'a mut FramePool,
}

#[derive(Debug)]
struct X {
    // a: i32,
    // b: i32,
}

impl<'a> CallStack<'a> {
    fn new(pool: &'a mut FramePool) -> Self {
        CallStack {
            stack: Vec::with_capacity(1024),
            pool,
        }
    }

    fn push_frame(&mut self, frame: *mut Frame) {
        self.stack.push(frame);
    }

    fn pop_frame(&mut self) -> Option<*mut Frame> {
        self.stack.pop()
    }

    // fn current_frame(&self) -> Option<&mut Frame> {
    //     self.stack.last().map(|&ptr| unsafe { &mut *ptr })
    // }
}

fn recursive_function(call_stack: &mut CallStack, depth: i32) -> i32 {
    if depth == 0 {
        return 0;
    } else if depth == 1 {
        return 1;
    }

    // 获取或复用栈帧
    let mut frame = call_stack.pool.get_frame();
    frame.return_address = 0; // 设置返回地址（根据需要设置）
    frame.local_vars =  vec![X{
    }];
    // 保存当前栈帧指针
    let frame_ptr: *mut Frame = &mut frame;
    call_stack.push_frame(frame_ptr);

    // 递归调用
    let result = recursive_function(call_stack, depth - 1) + recursive_function(call_stack, depth - 2);

    // 弹出栈帧并复用
    if let Some(f) = call_stack.pop_frame() {
        call_stack.pool.return_frame(unsafe { mem::replace(&mut *f, Frame {
            local_vars: vec![X{
            }], // 初始化局部变量
            return_address: 0,
        })});
    }

    result
}

fn main() {
    let mut frame_pool = FramePool::new();
    let mut call_stack = CallStack::new(&mut frame_pool);

    let result = recursive_function(&mut call_stack, 35);
    println!("Result: {}", result);
}

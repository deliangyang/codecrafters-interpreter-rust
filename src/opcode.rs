#[derive(Debug, Clone, PartialEq)]
pub enum Opcode {
    Add,
    LoadConstant(usize),
    Print(usize),
    Pop,
    Abs,
    Nagetive,
    JumpNotTruthy(usize),
    Jump(usize),
    Null,
    ReturnValue,
    SetGlobal(usize),
    GetGlobal(usize),
    SetLocal(usize),
    GetLocal(usize),
    GetBuiltin(usize),
    Closure(usize, usize),
    GetFree(usize),
    CurrentClosure,
    Call(usize),
    TailCall(usize, usize), // (closure_index, nargs)
    MemoCall(usize, usize), // (closure_index, nargs) - memoized call
    Return,
    Array(usize),
    Hash(usize),
    Index,
    SetIndex,
    True,
    False,
    Equal,
    JumpIfFalse(usize),
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanEqual,
    LessThanEqual,
    Minus,
    Mod,
    Divide,
    Multiply,
    Remainder,
    Bang,
    MinusOne,
    PlusOne,
    DefineGlobal(String),
    Assert(usize),
    EqualEqual,
    Exit(usize),
}

impl Opcode {
    #[inline]
    pub fn arg(&self) -> usize {
        match self {
            Opcode::LoadConstant(n) | Opcode::Print(n) | Opcode::Jump(n)
            | Opcode::JumpNotTruthy(n) | Opcode::JumpIfFalse(n)
            | Opcode::SetGlobal(n) | Opcode::GetGlobal(n)
            | Opcode::SetLocal(n) | Opcode::GetLocal(n)
            | Opcode::GetBuiltin(n) | Opcode::Call(n)
            | Opcode::Array(n) | Opcode::Hash(n) | Opcode::Assert(n)
            | Opcode::Exit(n) | Opcode::GetFree(n) => *n,
            Opcode::Closure(a, _) | Opcode::TailCall(a, _) | Opcode::MemoCall(a, _) => *a,
            _ => 0,
        }
    }

    #[inline]
    pub fn arg2(&self) -> usize {
        match self {
            Opcode::Closure(_, b) | Opcode::TailCall(_, b) | Opcode::MemoCall(_, b) => *b,
            _ => 0,
        }
    }

    pub fn discriminant(&self) -> u8 {
        match self {
            Opcode::Add => 0,
            Opcode::LoadConstant(_) => 1,
            Opcode::Print(_) => 2,
            Opcode::Pop => 3,
            Opcode::Abs => 4,
            Opcode::Nagetive => 5,
            Opcode::JumpNotTruthy(_) => 6,
            Opcode::Jump(_) => 7,
            Opcode::Null => 8,
            Opcode::ReturnValue => 9,
            Opcode::SetGlobal(_) => 10,
            Opcode::GetGlobal(_) => 11,
            Opcode::SetLocal(_) => 12,
            Opcode::GetLocal(_) => 13,
            Opcode::GetBuiltin(_) => 14,
            Opcode::Closure(_, _) => 15,
            Opcode::GetFree(_) => 16,
            Opcode::CurrentClosure => 17,
            Opcode::Call(_) => 18,
            Opcode::TailCall(_, _) => 19,
            Opcode::MemoCall(_, _) => 46,
            Opcode::Return => 20,
            Opcode::Array(_) => 21,
            Opcode::Hash(_) => 22,
            Opcode::Index => 23,
            Opcode::SetIndex => 24,
            Opcode::True => 25,
            Opcode::False => 26,
            Opcode::Equal => 27,
            Opcode::JumpIfFalse(_) => 28,
            Opcode::NotEqual => 29,
            Opcode::GreaterThan => 30,
            Opcode::LessThan => 31,
            Opcode::GreaterThanEqual => 32,
            Opcode::LessThanEqual => 33,
            Opcode::Minus => 34,
            Opcode::Mod => 35,
            Opcode::Divide => 36,
            Opcode::Multiply => 37,
            Opcode::Remainder => 38,
            Opcode::Bang => 39,
            Opcode::MinusOne => 40,
            Opcode::PlusOne => 41,
            Opcode::DefineGlobal(_) => 42,
            Opcode::Assert(_) => 43,
            Opcode::EqualEqual => 44,
            Opcode::Exit(_) => 45,
        }
    }
}

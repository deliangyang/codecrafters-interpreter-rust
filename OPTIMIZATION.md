# VM 性能优化：消除不必要的 clone

## 优化概述

本次优化主要通过引入 `Rc<Object>`（引用计数智能指针）来替代直接的 `Object` 值传递，从而将深拷贝操作转换为轻量级的引用计数增加操作。

## 主要修改

### 1. VM 结构体 (`src/vm.rs`)

**修改前：**
```rust
pub struct VM<'a> {
    constants: Vec<Object>,
    stack: Vec<Object>,
    globals: Vec<Object>,
    ...
}
```

**修改后：**
```rust
pub struct VM<'a> {
    constants: Vec<Rc<Object>>,
    stack: Vec<Rc<Object>>,
    globals: Vec<Rc<Object>>,
    ...
}
```

### 2. Object 定义 (`src/objects.rs`)

**修改前：**
```rust
pub enum Object {
    ...
    Array(Vec<Object>),
    ...
}
```

**修改后：**
```rust
pub enum Object {
    ...
    Array(Vec<Rc<Object>>),
    ...
}
```

### 3. VM 方法签名更新

**修改前：**
```rust
fn push(&mut self, obj: Object)
fn pop(&mut self) -> &Object
fn last(&mut self) -> &Object
```

**修改后：**
```rust
fn push(&mut self, obj: Rc<Object>)
fn pop(&mut self) -> Rc<Object>
fn last(&mut self) -> Rc<Object>
```

## 性能改进

### 消除的 clone 操作

在优化前，以下场景都需要深拷贝：

1. **LoadConstant**: `self.constants[*index].clone()` - 从常量池加载
2. **GetGlobal**: `self.globals[*index].clone()` - 读取全局变量
3. **SetGlobal**: `obj.clone()` - 设置全局变量
4. **GetBuiltin**: `obj.unwrap().clone()` - 获取内置函数
5. **GetFree**: `self.stack[...].clone()` - 闭包自由变量
6. **ReturnValue**: `last.clone()` - 函数返回值
7. **Index**: `obj.clone()` - 数组索引访问

### 优化效果

优化后，上述所有场景的 `clone()` 操作都变成了 `Rc::clone()`，仅增加引用计数（单个原子操作），而不是深拷贝整个对象。

**性能提升：**
- String: 避免堆内存分配和字符串内容拷贝
- Array: 避免递归深拷贝所有元素
- Hash: 避免 HashMap 深拷贝
- 其他复杂对象：避免嵌套结构的递归拷贝

对于简单类型（Number, Boolean, Nil, Index），性能影响可忽略不计，但对于复杂类型（String, Array, Hash），性能提升显著。

## 相关文件修改列表

1. `src/vm.rs` - 核心虚拟机实现
2. `src/objects.rs` - Object 枚举定义
3. `src/compiler.rs` - 数组字面量编译
4. `src/builtins.rs` - append 内置函数
5. `src/evaluator.rs` - 数组求值和索引表达式求值

## 兼容性说明

此优化是内部实现优化，不影响外部 API 和语言语义。所有现有的测试用例应该保持通过。

## 潜在的进一步优化

1. 将 `HashMap<Object, Object>` 改为 `HashMap<Rc<Object>, Rc<Object>>`
2. 考虑使用 `Arc` 替代 `Rc` 以支持多线程（如果需要）
3. 对于小对象（如 Number, Boolean），考虑使用 Copy-on-Write 策略

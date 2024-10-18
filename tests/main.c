#include <stdio.h>
#include <stdlib.h>

// 定义一个结构体来模拟栈帧
typedef struct {
    int n;
    int state;
    int result;
} StackFrame;

// 使用显式栈模拟递归调用计算 Fibonacci 数列
int fibonacci_recursive(int n) {
    if (n <= 1) {
        return n;
    }

    StackFrame *stack = (StackFrame *)malloc(sizeof(StackFrame) * (n + 1));
    int top = 0;
    int result = 0;

    // 初始化第一个栈帧
    stack[top++] = (StackFrame){n, 0, 0};

    while (top > 0) {
        StackFrame *frame = &stack[--top];

        switch (frame->state) {
            case 0:
                if (frame->n <= 1) {
                    frame->result = frame->n;
                } else {
                    frame->state = 1;
                    stack[top++] = *frame;  // 重新压入当前帧
                    stack[top++] = (StackFrame){frame->n - 1, 0, 0};  // 压入新的帧
                }
                break;
            case 1:
                frame->result += result;
                frame->state = 2;
                stack[top++] = *frame;  // 重新压入当前帧
                stack[top++] = (StackFrame){frame->n - 2, 0, 0};  // 压入新的帧
                break;
            case 2:
                frame->result += result;
                result = frame->result;
                break;
        }
    }

    free(stack);
    return result;
}

int main() {
    int n = 10;
    printf("Fibonacci(%d) = %d\n", n, fibonacci_recursive(n));
    return 0;
}

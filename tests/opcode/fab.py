def fib(n, a, b):
    if n == 0:
        return a
    elif n == 1:
        return b
    else:
        return fib(n - 1, b, a + b)

print(fib(35, 0, 1))  # Example usage

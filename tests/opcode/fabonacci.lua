function fibonacci(x)
  if x == 0 then return 0 end
  if x == 1 then return 1 end
  return fibonacci(x - 1) + fibonacci(x - 2)
end

print(fibonacci(35))
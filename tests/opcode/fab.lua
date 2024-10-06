function fab(n, a, b)
    if n == 0 then return a end
    if n == 1 then return b end
    return fab(n-1, b, a+b)
  end

print(fab(35, 0, 1))
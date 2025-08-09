function fact(n)
  if n == 0 then
    return 1
  else
    m = fact(n-1)
    return n * m
  end
end

print(fact(10))

local a = 1;

if a > 3 then
    print("a > 3");
elseif a > 2 then
    print("a > 2");
elseif a > 1 then
    print("a > 1");
else
    print("a <= 1");
end


function test()
    if a > 3 then
        print("a > 3");
    elseif a > 2 then
        print("a > 2");
    elseif a > 1 then
        print("a > 1");
    else
        print("a <= 1");
    end
end

test();
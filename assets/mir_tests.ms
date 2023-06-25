let x = b + (2 + a)
if x == 0 then
    x = 2
end

main:
.L0:
    ld %0, a
    add %1, %0, 2
    ld %0, b
    add %0, %1, %0
    st %0, x
.L1:
    
.L2:
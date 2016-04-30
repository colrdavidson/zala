mov A, 3
mov B, 5
mov C, 0
mov D, 2

loop:
out D, 1
out B, 1
out A, C
add C, 5
out B, 0
out D, 0
jmp loop

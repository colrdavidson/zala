mov A, 2
mov B, 3
mov C, 5
mov D, 6
mov E, 0

out A, 1
out C, 1

clock:
out B, E
inc E
out D, 1
out D, 0
jmp clock

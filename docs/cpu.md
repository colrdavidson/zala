# CPU Specification

| TIME | ADDRESS | DESCRIPTION                      |
|------|---------|----------------------------------|
|  0   | 0x0-0x7 | General Purpose (A,B,C,D,E,X,Y,Z)|

```
ENCODING:          24 bits     4    4
	|........................|....|....| |................................|
	 OPCODE                  REG2 REG1     OPTIONAL DATA <- 32 bits ->
```

| PORT |        DEVICE | INPUT   |
|------|---------------|---------|
|  0   | term num      | u32 val |
|  1   | term ascii    | char    |
|  2   | engine on/off | 1 / 0   |
|  3   | turret rot +  | u32 val |
|  4   | turret rot -  | u32 val |
|  5   | turret on/off | 1 / 0   |

The program is contained in a large array, indexed by the PC.
When opcodes that use the optional data int are used, the PC gets incremented twice, once to load/run the intruction, and once to load the data.
In order to calculate jmp placements manually, value and mem using operations need to be counted as double PC increments.

| TIME | ADDRESS | OPCODES | INPUT     | DESCRIPTION                                                    |
|------|---------|---------|-----------|----------------------------------------------------------------|
|  1   |  0x0    | NOP     | (none)    | No Operation                                                   |
|  0   |  0x1    | JMP     | addr      | Jump to Address                                                |
|  1   |  0x2    | HLT     | (none)    | Halt                                                           |
|  0   |  0x3    | MOV     | r1, val   | Load Val/R2 into R1                                            |
|  0   |  0x4    | INC     | r1        | Increment R1                                                   |
|  0   |  0x5    | SHR     | r1, val   | (r1 >> val) -> r1                                              |
|  0   |  0x6    | SHL     | r1, val   | (r1 << val) -> r1                                              |
|  0   |  0x8    | ADD     | r1, r2    | (r1 + r2)   -> r1                                              |
|  0   |  0x9    | SUB     | r1, r2    | (r1 - r2)   -> r1                                              |
|  0   |  0xA    | MUL     | r1, r2    | (r1 * r2)   -> r1                                              |
|  0   |  0xB    | DIV     | r1, r2    | (r1 / r2)   -> r1                                              |
|  0   |  0xC    | IFE     | r1, r2    | if (r1 == r2): next instruction, else, (next-next) instruction |
|  0   |  0xD    | IFN     | r1, r2    | if (r1 != r2): next instruction, else, (next-next) instruction |
|  0   |  0xE    | MMOV    | r1, [mem] | Move u32 in memory address into R1                             |
|  0   |  0xF    | MSET    | [mem], r1 | Move u32 in R1 into memory address                             |
|  0   |  0x10   | XOR     | r1, r2    | (r1 ^ r2)   -> r1                                              |
|  0   |  0x11   | IN      | r1, [in]  | Move u32 from port [in] to r1                                  |
|  0   |  0x12   | OUT     | [out], r1 | Move u32 from r1 to port [out]                                 |
|  0   |  0x13   | PUSH    | r1        | Push u32 from r1 onto stack, incrementing SP                   |
|  0   |  0x14   | POP     | r1        | Pop u32 from stack and put into r1, decrementing SP            |

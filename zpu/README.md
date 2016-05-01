# CPU Specification

| ADDRESS | DESCRIPTION                      | 
|---------|----------------------------------|
|0x0-0x7  | General Purpose (A,B,C,D,E,X,Y,Z)|

| FLAGS    |
|----------|
| cmp_flag |
| zero_flag|


```
ENCODING:          24 bits     4    4
	|........................|....|....| |................................|
	 OPCODE                  REG2 REG1     OPTIONAL DATA <- 32 bits ->
```

The program is contained in a large array, indexed by the PC.
When opcodes that use the optional data int are used, the PC gets incremented twice, once to load/run the intruction, and once to load the data.
In order to calculate jmp placements manually, value and mem using operations need to be counted as double PC increments.

| ADDRESS | OPCODES | INPUT     | DESCRIPTION                                                    | IMPLEMENTED |
|---------|---------|-----------|----------------------------------------------------------------|-------------|
|  0x0    | NOP     | (none)    | No Operation                                                   | Y	       |
|  0x1    | JMP     | addr      | Jump to Address                                                | Y           |
|  0x2    | HLT     | (none)    | Halt                                                           | Y           |
|  0x3    | INC     | r1        | Increment R1                                                   | Y           |
|  0x4    | SHR     | r1, val   | (r1 >> val) -> r1                                              | Y           |
|  0x5    | SHL     | r1, val   | (r1 << val) -> r1                                              | Y           |
|  0x6    | MOV     | r1, val   | Load Val/R2 into R1                                            | Y           |
|  0x7    | ADD     | r1, r2    | (r1 + r2)   -> r1                                              | Y           |
|  0x8    | SUB     | r1, r2    | (r1 - r2)   -> r1                                              | Y           |
|  0x9    | MUL     | r1, r2    | (r1 * r2)   -> r1                                              | Y           |
|  0xA    | DIV     | r1, r2    | (r1 / r2)   -> r1                                              | Y           |
|  0xB    | JE      | addr      | if (cmp_flag == 0): jmp to addr, else, continue	         | Y           |
|  0xC    | JN      | addr      | if (cmp_flag != 0): jmp to addr, else, continue	         | Y           |
|  0xD    | MMOV    | r1, [mem] | Move u32 in memory address into R1                             | N           |
|  0xE    | MSET    | [mem], r1 | Move u32 in R1 into memory address                             | N           |
|  0xF    | XOR     | r1, r2    | (r1 ^ r2)   -> r1                                              | N           |
|  0x10   | IN      | r1, [in]  | Move u32 from port [in] to r1                                  | N           |
|  0x11   | OUT     | [out], r1 | Move u32 from r1 to port [out]                                 | Y           |
|  0x12   | PUSH    | r1        | Push u32 from r1 onto stack, incrementing SP                   | N           |
|  0x13   | POP     | r1        | Pop u32 from stack and put into r1, decrementing SP            | N           |
|  0x14   | JZ      | addr      | if (zero_flag == 1): jmp to addr, else, continue	         | Y           |
|  0x15   | JG      | addr      | if (cmp_flag > 0): jmp to addr, else, continue	         | Y           |
|  0x16   | JL      | addr      | if (cmp_flag < 0): jmp to addr, else, continue	         | Y           |
|  0x17   | CMP     | r1, r2    | set cmp_flag to 1 if r1 > r2; -1 if r1 < r2; cmp_flag to 0, zero_flag to 1 if r1 == r2 | Y           |


| PORT |        DEVICE | INPUT   |
|------|---------------|---------|
|  0   | term num      | u32 val |
|  1   | term ascii    | char    |
|  2   | engine on/off | 1 / 0   |
|  3   | turret rot +  | u32 val |
|  4   | turret rot -  | u32 val |
|  5   | turret on/off | 1 / 0   |

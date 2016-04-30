# Radar

Radar - Interrupt, Port, MMIO

Radar has two modes, new object mode, and constant update mode.
In new object (NO) mode, radar sends an interrupt whenever something new pops up, and not when it moves.
In constant update (CU) mode, radar pulls significantly more power, and interrupts every time something moves.

| MODE | VALUE |
|------|-------|
| OFF  |  0x0  |
| CU   |  0x1  |
| NO   |  0x2  |

# Hardware Interface

## Interrupts

The ZPU takes hardware interrupts, allowing it to act when a keyboard input has been recieved or reconfigure as new hardware has been added.

## IO Ports System

A simple piece of hardware, such as a light will recieve a port number upon attachment to the computer system.

Example:

| DEVICE | PORT |
|--------|------|
| Light  |  1   |

A port takes an address, and can send, or recieve a word.

## Memory Mapped IO (MMIO)

More complex devices can ask the cpu for a chunk of memory, allowing faster, direct access. (Ex. A monitor would require MMIO,
as its entire buffer needs to be able to be rapidly modified en masse for smooth visual updates)

Example:

| DEVICE  | ADDRESSES   |
|---------|-------------|
| Monitor | 0x500-0x600 |

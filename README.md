A simple RISC-based virtual machine written in Rust.

Labels
Any line can be prefixed with an identifier and used as a reference point in other instructions.

Instructions
Directives
.byte: Store an ASCII value as a byte of data

Example
.byte 'A' 
.word: Store a signed integer as a word of data

Example
.word 10
Jumps
JMP: Jump to a label

Example
JMP label
JMR: Jump to an address stored in a register

Example
JMR reg_1
!0: Jump to a label if the value stored in a register is not 0

Example
!0  reg_1 label
>0: Jump to a label if the value stored in a register is greater than 0

Example
>0  reg_1 label
<0: Jump to a label if the value stored in a register is greater than 0

Example
<0  reg_1 label
=0: Jump to a label if the value stored in a register is equal to 0

Example
=0  reg_1 label
Moves
MOV: Copy data from the second register into the first register

Example
MOV reg_1 reg_2
LDA: Load the address of a label into a register

Example
LDA reg_1 label
STW: Store a word of data at a label

Example
STW reg_1 label
LDW: Loads a word of data from a label into a register

Example
LDW reg_1 label
STB: Store a byte of data at a label

Example
STB reg_1 label
LDB: Load a byte of data from a label

Example
LDB reg_1 label
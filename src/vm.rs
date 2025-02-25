use std::io;
use std::io::Cursor;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use assembler::{Command, CommandType};
use tokenizer::*;

pub struct VM {
    registers: [i32; 13],
    memory: Vec<u8>
}

impl VM {
    pub fn new(code: Vec<u8>) -> VM {
        // Expand available memory
        const MAX_MEMORY: usize = 10_000_000; // 10MB
        let mut memory = vec![0; MAX_MEMORY];

        // Copy bytecode into memory
        let mut i = code.len();
        while i > 0 {
            i -= 1;
            memory[i] = code[i];
        }
        VM {
            registers: [0; 13],
            memory: memory
        }
    }

    pub fn run(&mut self, start_address: usize) {
        let pc = Register::PC.to_bytecode() as usize;
        self.registers[pc] = start_address as i32;

        loop {
            let address = self.registers[pc] as usize;
            let bytecode = {
                let mut memory = Cursor::new(&mut self.memory[address..]);
                [
                    memory.read_i32::<LittleEndian>().unwrap(),
                    memory.read_i32::<LittleEndian>().unwrap(),
                    memory.read_i32::<LittleEndian>().unwrap(),
                ]
            };

            let command = Command::from_bytecode(&bytecode);
            let running = match command.cmd_type {
                CommandType::Instruction(instruction) =>
                    self.execute(instruction, &bytecode),
                _ => false
            };
            if !running {
                break;
            }

            self.registers[pc] += 12;
        }
    }

    fn execute(&mut self, instruction: InstructionType, bytecode: &[i32; 3]) -> bool {
        use tokenizer::InstructionType::*;
        match instruction {
            // Add together two registers and store the result in the first
            Add => {
                let destination = bytecode[1] as usize;
                let source = bytecode[2] as usize;
                self.registers[destination] += self.registers[source];
            },

            // Add an immediate value to a register
            AddImmediate => {
                let register = bytecode[1] as usize;
                let value = bytecode[2];
                self.registers[register] += value;
            },

            // Perform a boolean AND on two registers
            And => {
                let reg1 = bytecode[1] as usize;
                let reg2 = bytecode[2] as usize;
                let reg1_value = self.registers[reg1];
                let reg2_value = self.registers[reg2];
                self.registers[reg1] = if reg1_value != 0 && reg2_value != 0 {
                    1
                } else {
                    0
                };
            },

            // Compares the contents of two registers
            // -1 if the first is less than the second
            // 1  if the first is greater than the second
            // 0  if they're equal
            Compare => {
                let reg1 = bytecode[1] as usize;
                let reg2 = bytecode[2] as usize;
                let val1 = self.registers[reg1];
                let val2 = self.registers[reg2];
                self.registers[reg1] = if val1 < val2 {
                    -1
                } else if val1 > val2 {
                    1
                } else {
                    0
                };
            },

            // Jumps to an address if the given register contains a zero value
            CompareZeroJump => {
                let register = bytecode[1] as usize;
                let address = bytecode[2];
                // Remove offset that will be automatically applied
                let address = address - 12;
                if self.registers[register] == 0 {
                    self.registers[Register::PC as usize] = address;
                }
            },

            // Converts the ASCII representation of a number to the equivalent integer
            // '5' => 5
            ConvertASCIIToInteger => {
                let mut ascii = self.registers[Register::IO as usize];
                ascii -= '0' as i32;
                self.registers[Register::IO as usize] = if ascii < 0 || ascii > 9 {
                    -1
                } else {
                    ascii
                };
            },

            // Converts an integer value to the equivalent ASCII character
            // 5 => '5'
            ConvertIntegerToASCII => {
                let mut integer = self.registers[Register::IO as usize];
                integer += '0' as i32;
                self.registers[Register::IO as usize] = if integer < 48 || integer > 57 {
                    48
                } else {
                    integer
                };
            },

            // Perform integer division between two registers
            Divide => {
                let destination = bytecode[1] as usize;
                let source = bytecode[2] as usize;
                self.registers[destination] /= self.registers[source];
            },

            // If the contents of a register are greater than 0
            // jump to the specified address
            GreaterThanZeroJump => {
                let register = bytecode[1] as usize;
                let address = bytecode[2];
                // Remove offset that will be automatically applied
                let address = address - 12;
                if self.registers[register] > 0 {
                    self.registers[Register::PC as usize] = address;
                }
            },

            // Take in a character from the user and store it in the IO register
            InputASCII => {
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let character = input.chars().nth(0).unwrap();
                        self.registers[Register::IO as usize] = character as i32;
                    },
                    Err(err) => println!("error: {}", err)
                }
            },

            // Take in a number from the user and store it in the IO register
            InputInteger => {
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let num = input.trim().parse::<i32>();
                        match num {
                            Ok(n) => self.registers[Register::IO as usize] = n,
                            Err(err) => println!("error: {}", err)
                        }
                    },
                    Err(err) => println!("error: {}", err)
                }
            },

            // Jump directly to an address
            Jump => {
                let address = bytecode[1];

                // Remove offset that will be automatically applied
                let address = address - 12;
                self.registers[Register::PC as usize] = address;
            },

            // Jumps to an address stored in a register
            JumpRelative => {
                let register = bytecode[1] as usize;
                let address = self.registers[register];

                // Remove offset that will be automatically applied
                let address = address - 12;
                self.registers[Register::PC as usize] = address;
            },

            // If the contents of a register are less than 0
            // jump to the specified address
            LessThanZeroJump => {
                let register = bytecode[1] as usize;
                let address = bytecode[2];
                // Remove offset that will be automatically applied
                let address = address - 12;
                if self.registers[register] < 0 {
                    self.registers[Register::PC as usize] = address;
                }
            },

            // Loads the address of a label into a register
            LoadAddress => {
                let register = bytecode[1] as usize;
                let address = bytecode[2];
                self.registers[register] = address;
            },

            // Load a byte of data from memory and place it into a register
            LoadByte => {
                let register = bytecode[1] as usize;
                let address = bytecode[2] as usize;
                let mut memory = Cursor::new(&mut self.memory[address..]);
                let value = memory.read_u8().unwrap();
                self.registers[register] = value as i32;
            },

            // Load a word of data from memory and place it into a register
            LoadWord => {
                let register = bytecode[1] as usize;
                let address = bytecode[2] as usize;
                let mut memory = Cursor::new(&mut self.memory[address..]);
                let value = memory.read_u16::<LittleEndian>().unwrap();
                self.registers[register] = value as i32;
            },

            // Copy a value from register B and place it in register A
            Move => {
                let reg_a = bytecode[1] as usize;
                let reg_b = bytecode[2] as usize;
                let val_b = self.registers[reg_b];
                self.registers[reg_a] = val_b;
            },

            // Multiply the values in two registers together and store it in the first
            Multiply => {
                let reg_a = bytecode[1] as usize;
                let reg_b = bytecode[2] as usize;
                let val_a = self.registers[reg_a];
                let val_b = self.registers[reg_b];
                self.registers[reg_a] = val_a * val_b;
            },

            // Jumps to an address if the given register contains a non-zero value
            NonZeroJump => {
                let register = bytecode[1] as usize;
                let address = bytecode[2];
                // Remove offset that will be automatically applied
                let address = address - 12;
                if self.registers[register] != 0 {
                    self.registers[Register::PC as usize] = address;
                }
            },

            // If one of the registers contains a non-zero value, store 1
            // Otherwise, store 0 in the first register
            Or => {
                let reg1 = bytecode[1] as usize;
                let reg2 = bytecode[2] as usize;
                let reg1_value = self.registers[reg1];
                let reg2_value = self.registers[reg2];
                self.registers[reg1] = if reg1_value != 0 || reg2_value != 0 {
                    1
                } else {
                    0
                };
            },

            // Print out an ASCII character to stdout
            OutputASCII => {
                print!("{}", (self.registers[Register::IO as usize] as u8) as char);
            },

            // Print out a signed integer to stdout
            OutputInteger => {
                print!("{}", self.registers[Register::IO as usize]);
            },

            // Stores a byte of data at a location
            StoreByte => {
                let register = bytecode[1] as usize;
                let address = bytecode[2] as usize;
                let value = self.registers[register] as u8;
                let mut memory = &mut self.memory[address..];
                let _ = memory.write_u8(value);
            },

            // Stores a word of data at a location
            StoreWord => {
                let register = bytecode[1] as usize;
                let address = bytecode[2] as usize;
                let value = self.registers[register] as u16;
                let mut memory = &mut self.memory[address..];
                let _ = memory.write_u16::<LittleEndian>(value);
            },

            // Subtracts the value in register A from register B
            // and stores it in register A
            Subtract => {
                let reg_a = bytecode[1] as usize;
                let reg_b = bytecode[2] as usize;
                let val_a = self.registers[reg_a];
                let val_b = self.registers[reg_b];
                self.registers[reg_a] = val_a - val_b;
            },

            // End the program
            End => return false
        };
        true
    }
}
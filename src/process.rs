use std::io::{self, Write};
use rustyline;
use regex::Regex;
use lazy_static::lazy_static;

pub mod state;
pub use crate::process::state::*;

mod opcode;
use crate::process::opcode::Opcode;

#[derive(Debug)]
enum TrapVector {
    GETC, OUT, PUTS, IN, PUTSP, HALT,
}

impl TrapVector {
    fn from_instruction(instruction: u16) -> Result<TrapVector, String> {
        let value = instruction & 0xFF;

        match value {
            0x20 => Ok(TrapVector::GETC),
            0x21 => Ok(TrapVector::OUT),
            0x22 => Ok(TrapVector::PUTS),
            0x23 => Ok(TrapVector::IN),
            0x24 => Ok(TrapVector::PUTSP),
            0x25 => Ok(TrapVector::HALT),
            _ => Err(format!("bad TRAP vector: {:x}", value)),
        }
    }
}

pub fn process(mut state: State) -> State {
    let instruction : u16 = state.read_memory(state.pc);
    let opcode = Opcode::from_instruction(instruction);

    state.pc = state.pc.wrapping_add(1);

    match opcode {
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            if (n == 1 && state.condition == Condition::N) ||
               (z == 1 && state.condition == Condition::Z) ||
               (p == 1 && state.condition == Condition::P) {
                   let pc_offset = instruction & 0x1ff;

                   state.pc = state.pc.wrapping_add(sign_extend(pc_offset, 9));
            }
        }

        Opcode::ADD => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

            if immediate_flag {
                let immediate_value = sign_extend(instruction & 0x1f, 5);

                state.registers[r0 as usize] = state.registers[r1 as usize].wrapping_add(immediate_value);
            } else {
                let r2 = instruction & 0x7;

                state.registers[r0 as usize] = state.registers[r1 as usize].wrapping_add(state.registers[r2 as usize]);
            }

            state = update_flags(state, r0);
        }

        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.registers[r0 as usize] = state.memory[address as usize];

            state = update_flags(state, r0);
        }

        Opcode::ST => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.memory[address as usize] = state.registers[r0 as usize];
        }

        Opcode::JSR => {
            let temp = state.pc;
            let use_pc_offset = (instruction >> 11) & 1;
            let pc_offset = instruction & 0x1ff;
            let r0 = (instruction >> 6) & 7;

            if use_pc_offset == 1 {
                state.pc = state.pc.wrapping_add(sign_extend(pc_offset, 9));
            } else {
                state.pc = state.registers[r0 as usize];
            }

            state.registers[7] = temp;
        }

        Opcode::AND => {
            let immediate_flag = ((instruction >> 5) & 1) == 1;
            let immediate_value = sign_extend(instruction & 0x1f, 5);

            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let r2 = (instruction) & 0x7;

            if immediate_flag {
                state.registers[r0 as usize] = state.registers[r1 as usize] & immediate_value;
            } else {
                state.registers[r0 as usize] = state.registers[r1 as usize] & state.registers[r2 as usize];
            }
        }

        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            let address = state.registers[r1 as usize].wrapping_add(sign_extend(offset, 6));
            state.registers[r0 as usize] = state.read_memory(address);

            state = update_flags(state, r0);
        }

        Opcode::STR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            let address = state.registers[r0 as usize];
            let value = state.registers[r1 as usize].wrapping_add(sign_extend(offset, 6));

            state.memory[address as usize] = value;
        }

        Opcode::UNUSED => {
            panic!("unused");
        }

        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            state.registers[r0 as usize] = !state.registers[r1 as usize];
            state = update_flags(state, r0);
        }

        Opcode::LDI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = sign_extend(instruction & 0x1ff, 9);
            let address = state.pc.wrapping_add(pc_offset);

            state.registers[r0 as usize] = state.read_memory(address);

            state = update_flags(state, r0);
        }

        Opcode::STI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.memory[state.read_memory(address) as usize] = state.registers[r0 as usize];
        }

        Opcode::JMP => {
            let r0 = (instruction >> 6) & 0xa;

            state.pc = state.registers[r0 as usize];
        }

        Opcode::RESERVED => {
            panic!("reserved");
        }

        Opcode::LEA => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            state.registers[r0 as usize] = state.pc.wrapping_add(sign_extend(pc_offset, 9));
        }

        Opcode::TRAP => {
            if let Ok(trap_vector) = TrapVector::from_instruction(instruction) {
                match trap_vector {
                    TrapVector::GETC => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::OUT => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::PUTS => {
                        let mut i : u16 = state.registers[0];

                        while state.read_memory(i) != 0 {
                            print!("{}", char::from(state.read_memory(i) as u8));
                            i += 1;
                        }

                        io::stdout().flush().unwrap();
                    }

                    TrapVector::IN => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::PUTSP => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::HALT => {
                        state.running = false;
                    }
                }
            }
        }
    }

    state
}

pub fn debug(mut state: State) -> State {
    let mut rl = rustyline::Editor::<()>::new();
    let readline = rl.readline(&format!("{:#04x}> ", state.pc));

    let instruction : u16 = state.read_memory(state.pc);
    let opcode = Opcode::from_instruction(instruction);

    lazy_static! {
        static ref READ_REGEX: Regex = Regex::new(r"^read 0x([a-f0-9]{1,4})$").unwrap();
        static ref BREAK_ADDRESS_REGEX: Regex = Regex::new(r"^break-address 0x([a-f0-9]{1,4})$").unwrap();
    }

    state.debug_continue = false;

    match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_ref());

            match line.as_ref() {
                "c" | "continue" => {
                    state.debug_continue = true;
                }

                "i" | "inspect" => {
                    println!("{:?}, op_code: {:?}, instruction: {:#4x}, {:#016b}", state, opcode, instruction, instruction);
                }

                "d" | "disassemble" => {
                    disassemble(instruction, opcode);
                }

                line if READ_REGEX.is_match(line) => {
                    if let Some(address) = READ_REGEX.captures(line).unwrap().get(1) {
                        let address = u16::from_str_radix(address.as_str(), 16).unwrap();
                        let value = state.read_memory(address);
                        println!("{:#04x}, {:#016b}", value, value);
                    }
                }

                line if BREAK_ADDRESS_REGEX.is_match(line) => {
                    if let Some(address) = BREAK_ADDRESS_REGEX.captures(line).unwrap().get(1) {
                        let address = u16::from_str_radix(address.as_str(), 16).unwrap();
                        state.break_address = Some(address);
                        println!("Break address set to {:#04x}", address);
                    }
                }

                "exit" => {
                    state.running = false;
                }

                _ => {
                    println!("Unknown command {:?}", line);
                }
            }
        },
        Err(rustyline::error::ReadlineError::Interrupted) => {
            state.running = false;
        },
        Err(rustyline::error::ReadlineError::Eof) => {
            state.running = false;
        },
        Err(err) => {
            println!("Error: {:?}", err);
            state.running = false;
        }
    }

    state
}

fn disassemble(instruction: u16, opcode: Opcode) {
    match opcode {
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            println!("{:?}, {:#016b}, n: {}, z: {}, p: {}", opcode, instruction, n, z, p);
        }

        Opcode::ADD => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

            println!("{:?}, {:#016b}, r0: {}, r1: {}, immediate_flag: {}", opcode, instruction, r0, r1, immediate_flag);
        }

        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("{:?}, {:#016b}, r0: {}, pc_offset: {}", opcode, instruction, r0, pc_offset);
        }

        Opcode::ST => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("{:?}, {:#016b}, r0: {}, pc_offset: {}", opcode, instruction, r0, pc_offset);
        }

        Opcode::JSR => {
            let use_pc_offset = (instruction >> 11) & 1;
            let pc_offset = instruction & 0x1ff;
            let r0 = (instruction >> 6) & 7;

            println!("{:?}, {:#016b}, use_pc_offset: {}, pc_offset: {}, r0: {}",
                     opcode, instruction, use_pc_offset, pc_offset, r0);
        }

        Opcode::AND => {
            let immediate_flag = ((instruction >> 5) & 1) == 1;
            let immediate_value = sign_extend(instruction & 0x1f, 5);

            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let r2 = (instruction) & 0x7;

            println!("{:?}, {:#016b}, immediate_flag: {}, immediate_value: {}, r0: {}, r1: {}, r2: {}",
                     opcode, instruction, immediate_flag, immediate_value, r0, r1, r2);
        }

        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            println!("{:?}, {:#016b}, r0: {}, r1: {}, offset: {}", opcode, instruction, r0, r1, offset);
        }

        Opcode::STR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            println!("{:?}, {:#016b}, r0: {}, r1: {}, offset: {}", opcode, instruction, r0, r1, offset);
        }

        Opcode::UNUSED => {
            panic!("unused");
        }

        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            println!("{:?}, {:#016b}, r0: {}, r1: {}", opcode, instruction, r0, r1);
        }

        Opcode::LDI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = sign_extend(instruction & 0x1ff, 9);

            println!("{:?}, {:#016b}, r0: {}, pc_offset: {}", opcode, instruction, r0, pc_offset);
        }

        Opcode::STI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("{:?}, {:#016b}, r0: {}, pc_offset: {}", opcode, instruction, r0, pc_offset);
        }

        Opcode::JMP => {
            let r0 = (instruction >> 6) & 0xa;

            println!("{:?}, {:#016b}, r0: {}", opcode, instruction, r0);
        }

        Opcode::RESERVED => {
            panic!("reserved");
        }

        Opcode::LEA => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            println!("{:?}, {:#016b}, r0: {}, pc_offset: {}", opcode, instruction, r0, pc_offset);
        }

        Opcode::TRAP => {
            if let Ok(trap_vector) = TrapVector::from_instruction(instruction) {
                println!("{:?}, {:#016b}, trap_vector: {:?}", opcode, instruction, trap_vector);
            }
        }
    }
}

fn sign_extend(mut value: u16, bit_count: u8) -> u16 {
    if ((value >> (bit_count - 1)) & 1) == 1 {
        value |= 0xFFFF << bit_count;
    }
    value
}

fn update_flags(mut state: State, r: u16) -> State {
    if state.registers[r as usize] == 0 {
        state.condition = Condition::Z;
    } else if (state.registers[r as usize] >> 15) == 1 {
        // NOTE: A 1 in the left-most bit indicates negative
        state.condition = Condition::N;
    } else {
        state.condition = Condition::P;
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_add_immediate() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0001_010_001_1_00001;
        //                       ^    ^   `r1 ^ ^
        //                       `add |       | ` literal 1
        //                            ` r2    `immediate

        state.registers[1] = 3;

        let state = process(state);

        assert_eq!(state.registers, [0, 3, 4, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_add_register() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0001_010_001_0_00_000;
        //                       ^    ^   `r1 ^ ^  `r0
        //                       `add |       | ` unused
        //                            |       `register
        //                            ` r2 (destination)

        state.registers[0] = 2;
        state.registers[1] = 3;

        let state = process(state);

        assert_eq!(state.registers, [2, 3, 5, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_ldi() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1010_000_000000001;
        //                       ^    `r0 ^
        //                       `LDI     `pc_offset

        state.memory[0x3001] = 0x3002;
        state.memory[0x3002] = 42;

        let state = process(state);

        assert_eq!(state.registers, [42, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_jmp() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1100_000_010_000000;
        //                       ^        `register
        //                       `JMP

        state.registers[2] = 5;

        let state = process(state);

        assert_eq!(state.pc, 5);
    }

    #[test]
    fn process_br_n_true() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0000_1_0_0_000000101;
        //                       ^    `n    `pc_offset (5)
        //                       `BR

        state.condition = Condition::N;

        let state = process(state);

        // incremented pc + 5
        assert_eq!(state.pc, 0x3006);
    }

    #[test]
    fn process_br_n_false() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0000_1_0_0_000000101;
        //                       ^    `n    `pc_offset (5)
        //                       `BR

        state.condition = Condition::P;

        let state = process(state);

        // incremented pc + 1 (ingores the pc_offset)
        assert_eq!(state.pc, 0x3001);
    }

    #[test]
    fn process_ld() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0010_011_000000101;
        //                       ^    `r3 ^
        //                       `LD      `pc_offset (5)

        state.memory[0x3000 + 1 + 5] = 42;

        state.condition = Condition::P;

        let state = process(state);

        assert_eq!(state.registers[3], 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_st() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0011_011_000000101;
        //                       ^    `r3 ^
        //                       `LD      `pc_offset (5)

        state.registers[3] = 42;
        state.condition = Condition::P;

        let state = process(state);

        assert_eq!(state.memory[0x3000 + 1 + 5], 42);
    }

    #[test]
    fn process_jsr() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0100_0_00_011_000000;
        //                       ^    ^ ^  `r3 `unused
        //                       `JSR | `unused
        //                            `use pc_offset

        state.registers[3] = 42;

        let state = process(state);

        assert_eq!(state.pc, 42);
        assert_eq!(state.registers[7], 0x3001);
    }

    #[test]
    fn process_jsr_use_pc_offset() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0100_1_00000000011;
        //                       ^    ^ `pc_offset (k)
        //                       `JSR |
        //                            `use pc_offset

        let state = process(state);

        assert_eq!(state.pc, 0x3000 + 1 + 3);
        assert_eq!(state.registers[7], 0x3001);
    }

    #[test]
    fn process_and() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0101_001_010_0_00_011;
        //                       ^    `r0 `r1 ^    `r2
        //                       `AND         `immediate_flag

        state.registers[2] = 3;
        state.registers[3] = 5;

        let state = process(state);

        assert_eq!(state.registers[1], 3 & 5);
    }

    #[test]
    fn process_and_immediate() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0101_001_010_1_00101;
        //                       ^    `r0 `r1 ^ `immediate_value (5)
        //                       `AND         `immediate_flag

        state.registers[2] = 3;

        let state = process(state);

        assert_eq!(state.registers[1], 3 & 5);
    }

    #[test]
    fn process_ldr() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0110_001_010_000011;
        //                       ^    `r0 `r1 `offset (3)
        //                       `AND

        state.registers[2] = 1;
        state.memory[1 + 3] = 42;

        let state = process(state);

        assert_eq!(state.registers[1], 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_ldr_memory_address_too_big() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0110_001_010_000001;
        //                       ^    `r0 `r1 `offset (1)
        //                       `AND

        state.registers[2] = std::u16::MAX - 1;

        let state = process(state);

        assert_eq!(state.registers[1], 0);
        assert_eq!(state.condition, Condition::Z);
    }

    #[test]
    fn process_str() {
        let mut state = new_state();

        state.memory[0x3000] = 0b0111_001_010_000011;
        //                       ^    `r0 `r1 `offset (3)
        //                       `AND

        state.registers[1] = 42;
        state.registers[2] = 2;

        let state = process(state);

        assert_eq!(state.memory[42], 2 + 3);
    }

    #[test]
    fn process_not() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1001_001_010_1_11111;
        //                       ^    `r0 `r1
        //                       `NOT

        state.registers[2] = 42;

        let state = process(state);

        assert_eq!(state.registers[1], !42);
        // TODO: Why is this Z?
        // assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_sti() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1011_001_000000010;
        //                       ^    `r0 `pc_offset (2)
        //                       `STI

        let address = 3;
        state.registers[1] = 42;
        state.memory[(state.pc + 1 + 2) as usize] = address;

        let state = process(state);

        assert_eq!(state.memory[address as usize], 42);
    }

    #[test]
    fn process_lea() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1110_001_000000010;
        //                       ^    `r0 `pc_offset (2)
        //                       `LEA

        let state = process(state);

        assert_eq!(state.registers[1], 0x3000 + 1 + 2);
    }

    #[test]
    fn process_trap_halt() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1111_0000_00100101;
        //                       ^         `HALT (0x25)
        //                       `TRAP

        let state = process(state);

        assert_eq!(state.running, false);
    }

    fn new_state() -> State {
        State::new(false)
    }
}
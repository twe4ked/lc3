use crate::opcode::Opcode;
use crate::state::{Condition, State};
use crate::trap_vector::TrapVector;
use crate::utilities::sign_extend;
use std::io::{self, Write};

pub(crate) fn process(mut state: State) -> State {
    let instruction: u16 = state.read_memory(state.pc);
    let opcode = Opcode::from_instruction(instruction);

    state.pc = state.pc.wrapping_add(1);

    match opcode {
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            if (n == 1 && state.condition == Condition::N)
                || (z == 1 && state.condition == Condition::Z)
                || (p == 1 && state.condition == Condition::P)
            {
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

                state.registers[r0 as usize] =
                    state.registers[r1 as usize].wrapping_add(immediate_value);
            } else {
                let r2 = instruction & 0x7;

                state.registers[r0 as usize] =
                    state.registers[r1 as usize].wrapping_add(state.registers[r2 as usize]);
            }

            state.update_flags(r0);
        }

        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(sign_extend(pc_offset, 9));

            state.registers[r0 as usize] = state.memory[address as usize];
            state.update_flags(r0);
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
            let pc_offset = instruction & 0x7ff;
            let r0 = (instruction >> 6) & 7;

            if use_pc_offset == 1 {
                state.pc = state.pc.wrapping_add(sign_extend(pc_offset, 11));
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
                state.registers[r0 as usize] =
                    state.registers[r1 as usize] & state.registers[r2 as usize];
            }
        }

        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            let address = state.registers[r1 as usize].wrapping_add(sign_extend(offset, 6));

            state.registers[r0 as usize] = state.read_memory(address);
            state.update_flags(r0);
        }

        Opcode::STR => {
            let sr = (instruction >> 9) & 0x7;
            let base_r = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            let address = state.registers[base_r as usize].wrapping_add(sign_extend(offset, 6));
            let value = state.registers[sr as usize];

            state.memory[address as usize] = value;
        }

        Opcode::UNUSED => {
            panic!("unused");
        }

        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            state.registers[r0 as usize] = !state.registers[r1 as usize];
            state.update_flags(r0);
        }

        Opcode::LDI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = sign_extend(instruction & 0x1ff, 9);
            let address = state.pc.wrapping_add(pc_offset);

            state.registers[r0 as usize] = state.read_memory(address);
            state.update_flags(r0);
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
                        // Read a single character from the keyboard. The character is not echoed
                        // onto the console. Its ASCII code is copied into R0. The high eight bits
                        // of R0 are cleared.
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::OUT => {
                        panic!("not implemented: {:?}", trap_vector);
                    }

                    TrapVector::PUTS => {
                        let mut i: u16 = state.registers[0];

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

        state.memory[0x3000] = 0b0100_1_10000000011;
        //                       ^    ^ `pc_offset (1027)
        //                       `JSR |
        //                            `use pc_offset

        let state = process(state);

        assert_eq!(state.pc, (0x3001 as u16).wrapping_add(0b11111100_00000011));
        //                      `incremented pc           ^
        //                                                `sign extended 1027
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

        assert_eq!(state.memory[2 + 3], 42);
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

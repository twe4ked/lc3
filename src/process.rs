use crate::opcode::Opcode;
use crate::state::{Condition, State};
use crate::trap_vector::TrapVector;
use crate::SignExtend;
use std::io::{self, Read, Write};

pub(crate) fn run(mut state: State) {
    while state.running {
        state = process(state)
    }
}

pub(crate) fn process(mut state: State) -> State {
    let instruction: u16 = state.read_memory(state.pc);
    let opcode = Opcode::from_instruction(instruction);

    state.pc = state.pc.wrapping_add(1);

    match opcode {
        // BR - Conditional Branch
        //
        // Assembler Formats
        //
        //      BRn     LABEL   BRzp    LABEL
        //      BRz     LABEL   BRnp    LABEL
        //      BRp     LABEL   BRnz    LABEL
        //      BR[1]   LABEL   BRnzp   LABEL
        //
        // Encoding
        //
        //      |0 0 0 0|0|0|0|0 0 0 0 0 0 0 0 0|
        //      |BR     |n|p|z|pc_offset_9      |
        //
        // Description
        //
        // The condition codes specified by the state of bits [11:9] are tested. If bit [11] is
        // set, N is tested; if bit [11] is clear, N is not tested. If bit [10] is set, Z is
        // tested, etc. If any of the condition codes tested is set, the program branches to the
        // location specified by adding the sign-extended PCoffset9 field to the incremented PC.
        //
        // Examples
        //
        //      BRzp LOOP    ; Branch to LOOP if the last result was zero or positive.
        //      BR[1] NEXT   ; Unconditionally branch to NEXT.
        //
        // [1]: The assembly language opcode BR is interpreted the same as BRnzp; that is, always
        // branch to the target address.
        Opcode::BR => {
            let n = (instruction >> 11) & 0x1;
            let z = (instruction >> 10) & 0x1;
            let p = (instruction >> 9) & 0x1;

            if (n == 1 && state.condition == Condition::N)
                || (z == 1 && state.condition == Condition::Z)
                || (p == 1 && state.condition == Condition::P)
            {
                let pc_offset = instruction & 0x1ff;

                state.pc = state.pc.wrapping_add(pc_offset.sign_extend(9));
            }
        }

        // ADD - Addition
        //
        // Assembler Formats
        //
        //      ADD DR, SR1, SR2
        //      ADD DR, SR1, imm5
        //
        // Encodings
        //
        //      |0 0 0 1|0 0 0|0 0 0|0|0 0|0 0 0|
        //      |ADD    |DR   |SR1  |x|   |SR2  |
        //
        //      |0 0 0 1|0 0 0|0 0 0|1|0 0|0 0 0|
        //      |ADD    |DR   |SR1  |x|imm_5    |
        //
        //      x: bit [5] (immidiate_flag)
        //
        // Description
        //
        // If bit [5] is 0, the second source operand is obtained from SR2. If bit [5] is 1, the
        // second source operand is obtained by sign-extending the imm5 field to 16 bits. In both
        // cases, the second source operand is added to the contents of SR1 and the result stored
        // in DR. The condition codes are set, based on whether the result is negative, zero, or
        // positive.
        //
        // Examples
        //
        //      ADD R2, R3, R4 ; R2 <- R3 + R4
        //      ADD R2, R3, #7 ; R2 <- R3 + 7
        Opcode::ADD => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

            if immediate_flag {
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                state.registers[r0 as usize] =
                    state.registers[r1 as usize].wrapping_add(immediate_value);
            } else {
                let r2 = instruction & 0x7;

                state.registers[r0 as usize] =
                    state.registers[r1 as usize].wrapping_add(state.registers[r2 as usize]);
            }

            state.update_flags(r0);
        }

        // LD - Load
        //
        // Assembler Format
        //
        //      LD DR, LABEL
        //
        // Encoding
        //
        //      |0 0 1 0|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |LD     |DR   |pc_offset_9      |
        //
        // Description
        //
        // An address is computed by sign-extending bits [8:0] to 16 bits and adding this value to
        // the incremented PC. The contents of memory at this address are loaded into DR. The
        // condition codes are set, based on whether the value loaded is negative, zero, or
        // positive.
        //
        // Example
        //
        //      LD R4, VALUE ; R4 <- mem[VALUE]
        Opcode::LD => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));

            state.registers[r0 as usize] = state.memory[address as usize];
            state.update_flags(r0);
        }

        // ST - Store
        //
        // Assembler Format
        //
        //      ST SR, LABEL
        //
        // Encoding
        //
        //      |0 0 1 1|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |ST     |DR   |pc_offset_9      |
        //
        // Description
        //
        // The contents of the register specified by SR are stored in the memory location whose
        // address is computed by sign-extending bits [8:0] to 16 bits and adding this value to the
        // incremented PC.
        //
        // Example
        //
        //      ST R4, HERE ; mem[HERE] <- R4
        Opcode::ST => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;
            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));

            state.memory[address as usize] = state.registers[r0 as usize];
        }

        // JSR - Jump to Subroutine
        // JSRR
        //
        // Assembler Formats
        //
        //      JSR LABEL
        //      JSRR BaseR
        //
        // Encoding
        //
        //      |0 1 0 0|1|0 0 0 0 0 0 0 0 0 0 0|
        //      |JSR    |x|pc_offset_11         |
        //
        //      |0 1 0 0|0|0 0|0 0 0 0 0 0 0 0 0|
        //      |JSRR   |x|   |BR   |           |
        //
        //      x: bit [11] (use_pc_offset)
        //
        // Description
        //
        // First, the incremented PC is saved in a temporary location. Then the PC is loaded with
        // the address of the first instruction of the subroutine, causing an unconditional jump to
        // that address. The address of the subroutine is obtained from the base register (if bit
        // [11] is 0), or the address is computed by sign-extending bits [10:0] and adding this
        // value to the incremented PC (if bit [11] is 1). Finally, R7 is loaded with the value
        // stored in the temporary location. This is the linkage back to the calling routine.
        //
        // Examples
        //
        //      JSR QUEUE    ; Put the address of the instruction following JSR into R7;
        //                   ; Jump to QUEUE.
        //      JSRR R3      ; Put the address following JSRR into R7; Jump to the
        //                   ; address contained in R3.
        Opcode::JSR => {
            let temp = state.pc;
            let use_pc_offset = (instruction >> 11) & 1;
            let pc_offset = instruction & 0x7ff;
            let r0 = (instruction >> 6) & 7;

            if use_pc_offset == 1 {
                state.pc = state.pc.wrapping_add(pc_offset.sign_extend(11));
            } else {
                state.pc = state.registers[r0 as usize];
            }

            state.registers[7] = temp;
        }

        // AND - Bit-wise Logical AND
        //
        // Assembler Formats
        //
        //      AND DR, SR1, SR2
        //      AND DR, SR1, imm5
        //
        // Encodings
        //
        //      |0 1 0 1|0 0 0|0 0 0|0|0 0|0 0 0|
        //      |AND    |DR   |SR1  |x|   |SR2  |
        //
        //      |0 1 0 1|0 0 0|0 0 0|1|0 0|0 0 0|
        //      |AND    |DR   |SR1  |x|imm_5    |
        //
        //      x: bit [5] (immidiate_flag)
        //
        // Description
        //
        // If bit [5] is 0, the second source operand is obtained from SR2. If bit [5] is 1, the
        // second source operand is obtained by sign-extending the imm5 field to 16 bits. In either
        // case, the second source operand and the contents of SR1 are bit-wise ANDed, and the
        // result stored in DR. The condition codes are set, based on whether the binary value
        // produced, taken as a 2’s complement integer, is negative, zero, or positive.
        //
        // Examples
        //
        //      AND R2, R3, R4 ;R2 <- R3 AND R4
        //      AND R2, R3, #7 ;R2 <- R3 AND 7
        Opcode::AND => {
            let immediate_flag = ((instruction >> 5) & 1) == 1;
            let immediate_value = (instruction & 0x1f).sign_extend(5);

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

        // LDR - Load Base+offset
        //
        // Assembler Format
        //
        //      LDR DR, BaseR, offset6
        //
        // Encoding
        //
        //      |0 1 1 0|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |LDR    |DR   |BR   |offset_6   |
        //
        // Description
        //
        // An address is computed by sign-extending bits [5:0] to 16 bits and adding this value to
        // the contents of the register specified by bits [8:6]. The contents of memory at this
        // address are loaded into DR. The condition codes are set, based on whether the value
        // loaded is negative, zero, or positive.
        //
        // Example
        //
        // LDR R4, R2, #−5 ; R4 <- mem[R2 − 5]
        Opcode::LDR => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;
            let offset = (instruction) & 0x3f;

            let address = state.registers[r1 as usize].wrapping_add(offset.sign_extend(6));

            state.registers[r0 as usize] = state.read_memory(address);
            state.update_flags(r0);
        }

        // STR - Store Base+offset
        //
        // Assembler Format
        //
        //      STR SR, BaseR, offset6
        //
        // Encoding
        //
        //      |0 1 1 1|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |STR    |SR   |BR   |offset_6   |
        //
        // Description
        //
        // The contents of the register specified by SR are stored in the memory location whose
        // address is computed by sign-extending bits [5:0] to 16 bits and adding this value to the
        // contents of the register specified by bits [8:6].
        //
        // Example
        //
        // STR R4, R2, #5 ; mem[R2 + 5] <- R4
        Opcode::STR => {
            let sr = (instruction >> 9) & 0x7;
            let base_r = (instruction >> 6) & 0x7;
            let offset = instruction & 0x3f;

            let address = state.registers[base_r as usize].wrapping_add(offset.sign_extend(6));
            let value = state.registers[sr as usize];

            state.memory[address as usize] = value;
        }

        Opcode::UNUSED => {
            unimplemented!();
        }

        // NOT - Bit-Wise Complement
        //
        // Assembler Format
        //
        //      NOT DR, SR
        //
        // Encoding
        //
        //      |1 0 0 1|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |NOT    |DR   |SR   |1|1 1 1 1 1|
        //
        // Description
        //
        // The bit-wise complement of the contents of SR is stored in DR. The condition codes are
        // set, based on whether the binary value produced, taken as a 2’s complement integer, is
        // negative, zero, or positive.
        //
        // Example
        //
        // NOT R4, R2 ; R4 <- NOT(R2)
        Opcode::NOT => {
            let r0 = (instruction >> 9) & 0x7;
            let r1 = (instruction >> 6) & 0x7;

            state.registers[r0 as usize] = !state.registers[r1 as usize];
            state.update_flags(r0);
        }

        // LDI - Load Indirect
        //
        // Assembler Format
        //
        //      LDI DR, LABEL
        //
        // Encoding
        //
        // Description
        //
        //      |1 0 1 0|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |LDI    |DR   |pc_offset_9      |
        //
        // An address is computed by sign-extending bits [8:0] to 16 bits and adding this value to
        // the incremented PC. What is stored in memory at this address is the address of the data
        // to be loaded into DR. The condition codes are set, based on whether the value loaded is
        // negative, zero, or positive.
        //
        // Example
        //
        //      LDI R4, ONEMORE ; R4 <- mem[mem[ONEMORE]]
        Opcode::LDI => {
            let dr = (instruction >> 9) & 0x7;
            let pc_offset = (instruction & 0x1ff).sign_extend(9);
            let address = state.read_memory(state.pc.wrapping_add(pc_offset));

            state.registers[dr as usize] = state.read_memory(address);
            state.update_flags(dr);
        }

        // STI - Store Indirect
        //
        // Assembler Format
        //
        //      STI SR, LABEL
        //
        // Encoding
        //
        //      |1 0 1 1|0 0 0|0 0 0 0 0 0 0 0 0|
        //      |STI    |SR   |pc_offset_9      |
        //
        // Description
        //
        // The contents of the register specified by SR are stored in the memory location whose
        // address is obtained as follows: Bits [8:0] are sign-extended to 16 bits and added to the
        // incremented PC. What is in memory at this address is the address of the location to
        // which the data in SR is stored.
        //
        // Example
        //
        // STI R4, NOT_HERE ; mem[mem[NOT_HERE]] <- R4
        Opcode::STI => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));

            state.memory[state.read_memory(address) as usize] = state.registers[r0 as usize];
        }

        // JMP - Jump
        // RET - Return from Subroutine
        //
        // Assembler Formats
        //
        //      JMP BaseR
        //      RET
        //
        // Encoding
        //
        //      |1 1 0 0|0 0 0|0 0 0|0 0 0 0 0 0|
        //      |JMP    |     |BR   |           |
        //
        //      |1 1 0 0|0 0 0|1 1 1|0 0 0 0 0 0|
        //      |RET    |     |1 1 1|           |
        //
        // Description
        //
        // The program unconditionally jumps to the location specified by the contents of
        // the base register. Bits [8:6] identify the base register.
        //
        // Examples
        //
        //      JMP R2 ; PC <0 R2
        //      RET ; PC <- R7
        //
        // Note
        //
        // The RET instruction is a special case of the JMP instruction. The PC is loaded with the
        // contents of R7, which contains the linkage back to the instruction following the
        // subroutine call instruction.
        Opcode::JMP => {
            let r0 = (instruction >> 6) & 0x7;

            state.pc = state.registers[r0 as usize];
        }

        Opcode::RESERVED => {
            unimplemented!();
        }

        // LEA - Load Effective Address
        //
        // Assembler Format
        //
        //      LEA DR, LABEL
        //
        // Encoding
        //
        //      |1 1 1 0|0 0 0|0 0 0|0 0 0 0 0 0|
        //      |LEA    |DR   |pc_offset_9      |
        //
        // Description
        //
        // An address is computed by sign-extending bits [8:0] to 16 bits and adding this value to
        // the incremented PC. This address is loaded into DR.[1] The condition codes are set,
        // based on whether the value loaded is negative, zero, or positive.
        //
        // [1]: The LEA instruction does not read memory to obtain the information to load into DR.
        // The address itself is loaded into DR.
        //
        // Example
        //
        // LEA R4, TARGET ; R4 <- address of TARGET.
        Opcode::LEA => {
            let r0 = (instruction >> 9) & 0x7;
            let pc_offset = instruction & 0x1ff;

            state.registers[r0 as usize] = state.pc.wrapping_add(pc_offset.sign_extend(9));
        }

        // TRAP - System Call
        //
        // Assembler Format
        //
        //      TRAP trapvector8
        //
        // Encoding
        //
        //      |1 1 1 1|0 0 0 0|0 0 0 0 0 0 0 0|
        //      |TRAP   |       |trap_vector_8  |
        //
        // Description
        //
        // First R7 is loaded with the incremented PC. (This enables a return to the instruction
        // physically following the TRAP instruction in the original program after the service
        // routine has completed execution.) Then the PC is loaded with the starting address of the
        // system call specified by trapvector8. The starting address is contained in the memory
        // location whose address is obtained by zero-extending trapvector8 to 16 bits.
        //
        // Example
        //
        //      TRAP x23    ; Directs the operating system to execute the IN system call.
        //                  ; The starting address of this system call is contained in
        //                  ; memory location x0023.
        //
        // Note
        //
        // Memory locations x0000 through x00FF, 256 in all, are available to contain starting
        // addresses for system calls specified by their corresponding trap vectors. This region of
        // memory is called the Trap Vector Table. Table A.2 describes the functions performed
        // by the service routines corresponding to trap vectors x20 to x25.
        Opcode::TRAP => {
            if let Ok(trap_vector) = TrapVector::from_instruction(instruction) {
                match trap_vector {
                    TrapVector::GETC => {
                        // Read a single character from the keyboard. The character is not echoed
                        // onto the console. Its ASCII code is copied into R0. The high eight bits
                        // of R0 are cleared.
                        let mut buffer = [0; 1];
                        io::stdin().read_exact(&mut buffer).unwrap();

                        state.registers[0] = u16::from(buffer[0]);
                    }

                    TrapVector::OUT => {
                        print!("{}", char::from(state.registers[0] as u8));
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
                        unimplemented!();
                    }

                    TrapVector::PUTSP => {
                        unimplemented!();
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
        state.memory[0x3002] = 0x3003;
        state.memory[0x3003] = 42;

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
    fn process_jmp_ret() {
        let mut state = new_state();

        state.memory[0x3000] = 0b1100_000_111_000000;
        //                       ^        `register
        //                       `JMP

        state.registers[7] = 42;

        let state = process(state);

        assert_eq!(state.pc, 42);
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
        State::new()
    }
}

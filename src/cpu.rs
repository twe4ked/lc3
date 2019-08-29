use crate::instruction::Register::*;
use crate::instruction::{Instruction, TrapVector};
use crate::state::{Condition, State};
use crate::SignExtend;
use std::io::{self, Read, Write};

pub fn execute(mut state: State, instruction: Instruction) -> State {
    state.pc = state.pc.wrapping_add(1);

    match instruction {
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
        Instruction::BR(condition, pc_offset) => {
            if (condition.n && state.condition == Condition::N)
                || (condition.z && state.condition == Condition::Z)
                || (condition.p && state.condition == Condition::P)
            {
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
        Instruction::ADD(r0, r1, r2) => {
            let value = state
                .registers
                .read(r1)
                .wrapping_add(state.registers.read(r2));

            state.registers.write(r0, value);
            state.update_flags(r0);
        }
        Instruction::ADDIMM(r0, r1, immediate_value) => {
            let value = state
                .registers
                .read(r1)
                .wrapping_add(immediate_value.sign_extend(5));

            state.registers.write(r0, value);
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
        Instruction::LD(r0, pc_offset) => {
            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));
            let value = state.memory.read(address);

            state.registers.write(r0, value);
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
        Instruction::ST(r0, pc_offset) => {
            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));

            state.memory.write(address, state.registers.read(r0));
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
        Instruction::JSR(pc_offset) => {
            let temp = state.pc;
            state.pc = state.pc.wrapping_add(pc_offset.sign_extend(11));
            state.registers.write(R7, temp);
        }
        Instruction::JSRR(r0) => {
            let temp = state.pc;
            state.pc = state.registers.read(r0);
            state.registers.write(R7, temp);
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
        Instruction::AND(r0, r1, r2) => {
            let value = state.registers.read(r1) & state.registers.read(r2);
            state.registers.write(r0, value);
        }
        Instruction::ANDIMM(immediate_value, r0, r1) => {
            let value = state.registers.read(r1) & immediate_value.sign_extend(5);
            state.registers.write(r0, value);
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
        Instruction::LDR(r0, r1, offset) => {
            let address = state.registers.read(r1).wrapping_add(offset.sign_extend(6));
            let value = state.memory.read(address);

            state.registers.write(r0, value);
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
        Instruction::STR(sr, base_r, offset) => {
            let address = state
                .registers
                .read(base_r)
                .wrapping_add(offset.sign_extend(6));
            let value = state.registers.read(sr);

            state.memory.write(address, value);
        }

        Instruction::UNUSED => {
            panic!("unused");
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
        Instruction::NOT(r0, r1) => {
            state.registers.write(r0, !state.registers.read(r1));
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
        Instruction::LDI(dr, pc_offset) => {
            let address = state
                .memory
                .read(state.pc.wrapping_add(pc_offset.sign_extend(9)));
            let value = state.memory.read(address);

            state.registers.write(dr, value);
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
        Instruction::STI(r0, pc_offset) => {
            let address = state.pc.wrapping_add(pc_offset.sign_extend(9));
            let address = state.memory.read(address);

            state.memory.write(address, state.registers.read(r0));
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
        Instruction::JMP(r0) => {
            state.pc = state.registers.read(r0);
        }

        Instruction::RESERVED => {
            panic!("reserved");
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
        Instruction::LEA(r0, pc_offset) => {
            state
                .registers
                .write(r0, state.pc.wrapping_add(pc_offset.sign_extend(9)));
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
        Instruction::TRAP(trap_vector) => {
            match trap_vector {
                // Read a single character from the keyboard. The character is not echoed
                // onto the console. Its ASCII code is copied into R0. The high eight bits
                // of R0 are cleared.
                TrapVector::GETC => {
                    let mut buffer = [0; 1];
                    io::stdin().read_exact(&mut buffer).unwrap();

                    state.registers.write(R0, u16::from(buffer[0]));
                }

                // Write a character in R0[7:0] to the console display.
                TrapVector::OUT => {
                    print!("{}", char::from(state.registers.read(R0) as u8));
                }

                // Write a string of ASCII characters to the console display. The characters
                // are contained in consecutive memory locations, one character per memory
                // location, starting with the address specified in R0. Writing terminates with
                // the occurrence of x0000 in a memory location.
                TrapVector::PUTS => {
                    let mut i: u16 = state.registers.read(R0);

                    while state.memory.read(i) != 0 {
                        print!("{}", char::from(state.memory.read(i) as u8));
                        i += 1;
                    }

                    io::stdout().flush().unwrap();
                }

                // Print a prompt on the screen and read a single character from the keyboard.
                // The character is echoed onto the console monitor, and its ASCII code is
                // copied into R0. The high eight bits of R0 are cleared.
                TrapVector::IN => {
                    unimplemented!();
                }

                // Write a string of ASCII characters to the console. The characters are
                // contained in consecutive memory locations, two characters per memory
                // location, starting with the address specified in R0. The ASCII code
                // contained in bits [7:0] of a memory location is written to the console
                // first. Then the ASCII code contained in bits [15:8] of that memory location
                // is written to the console. (A character string consisting of an odd number
                // of characters to be written will have x00 in bits [15:8] of the memory
                // location containing the last character to be written.) Writing terminates
                // with the occurrence of x0000 in a memory location.
                TrapVector::PUTSP => {
                    unimplemented!();
                }

                // Halt execution and print a message on the console.
                TrapVector::HALT => {
                    state.running = false;
                }
            }
        }
    }

    state
}

#[cfg(test)]
mod tests {
    use super::Instruction::*;
    use super::*;
    use crate::instruction;

    #[test]
    fn process_addimm() {
        let mut state = new_state();
        state.registers.write(R1, 3);

        state = execute(state, ADDIMM(R2, R1, 1));

        assert_eq!(state.registers.read(R2), 4);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_add() {
        let mut state = new_state();
        state.registers.write(R0, 2);
        state.registers.write(R1, 3);

        state = execute(state, ADD(R2, R1, R0));

        assert_eq!(state.registers.read(R2), 5);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_ldi() {
        let mut state = new_state();
        state.memory.write(0x3001, 0x3002);
        state.memory.write(0x3002, 0x3003);
        state.memory.write(0x3003, 42);

        state = execute(state, LDI(R0, 1));

        assert_eq!(state.registers.read(R0), 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_jmp() {
        let mut state = new_state();
        state.registers.write(R2, 5);

        state = execute(state, JMP(R2));

        assert_eq!(state.pc, 5);
    }

    #[test]
    fn process_jmp_ret() {
        let mut state = new_state();
        state.registers.write(R7, 42);

        state = execute(state, JMP(R7));

        assert_eq!(state.pc, 42);
    }

    #[test]
    fn process_br_n_true() {
        let mut state = new_state();
        state.condition = Condition::N;

        let condition = instruction::Condition {
            n: true,
            z: false,
            p: false,
        };
        state = execute(state, BR(condition, 5));

        // incremented pc + 5
        assert_eq!(state.pc, 0x3006);
    }

    #[test]
    fn process_br_n_false() {
        let mut state = new_state();
        state.condition = Condition::P;

        let condition = instruction::Condition {
            n: false,
            z: false,
            p: false,
        };
        state = execute(state, BR(condition, 5));

        // incremented pc + 1 (ingores the pc_offset)
        assert_eq!(state.pc, 0x3001);
    }

    #[test]
    fn process_br_any() {
        let mut state = new_state();
        state.condition = Condition::Z;

        let condition = instruction::Condition {
            n: false,
            z: false,
            p: false,
        };
        state = execute(state, BR(condition, 5));

        // incremented pc + 1 (ingores the pc_offset)
        assert_eq!(state.pc, 0x3001);
    }

    #[test]
    fn process_ld() {
        let mut state = new_state();
        state.condition = Condition::P;
        state.memory.write(0x3000 + 1 + 5, 42);

        state = execute(state, LD(R3, 5));

        assert_eq!(state.registers.read(R3), 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_st() {
        let mut state = new_state();
        state.registers.write(R3, 42);
        state.condition = Condition::P;

        state = execute(state, ST(R3, 5));

        assert_eq!(state.memory.read(0x3000 + 1 + 5), 42);
    }

    #[test]
    fn process_jsrr() {
        let mut state = new_state();
        state.registers.write(R3, 42);

        state = execute(state, JSRR(R3));

        assert_eq!(state.pc, 42);
        assert_eq!(state.registers.read(R7), 0x3001);
    }

    #[test]
    fn process_jsr() {
        let mut state = new_state();

        state = execute(state, JSR(0b10000000011)); // 1027

        assert_eq!(state.pc, (0x3001 as u16).wrapping_add(0b11111100_00000011));
        //                      `incremented pc           ^
        //                                                `sign extended 1027
        assert_eq!(state.registers.read(R7), 0x3001);
    }

    #[test]
    fn process_and() {
        let mut state = new_state();
        state.registers.write(R2, 3);
        state.registers.write(R3, 5);

        state = execute(state, AND(R1, R2, R3));

        assert_eq!(state.registers.read(R1), 3 & 5);
    }

    #[test]
    fn process_andimm() {
        let mut state = new_state();
        state.registers.write(R2, 3);

        state = execute(state, ANDIMM(5, R1, R2));

        assert_eq!(state.registers.read(R1), 3 & 5);
    }

    #[test]
    fn process_ldr() {
        let mut state = new_state();
        state.registers.write(R2, 1);
        state.memory.write(1 + 3, 42);

        state = execute(state, LDR(R1, R2, 3));

        assert_eq!(state.registers.read(R1), 42);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_str() {
        let mut state = new_state();
        state.registers.write(R1, 42);
        state.registers.write(R2, 2);

        state = execute(state, STR(R1, R2, 3));

        assert_eq!(state.memory.read(2 + 3), 42);
    }

    #[test]
    fn process_not() {
        let mut state = new_state();
        let a = 0b11111111_11010110; // -42
        state.registers.write(R2, a);

        state = execute(state, NOT(R1, R2));

        assert_eq!(state.registers.read(R1), !a);
        assert_eq!(state.registers.read(R1), 0b00000000_00101001);
        assert_eq!(state.condition, Condition::P);
    }

    #[test]
    fn process_sti() {
        let mut state = new_state();
        let address = 3;
        state.registers.write(R1, 42);
        state.memory.write(state.pc + 1 + 2, address);

        state = execute(state, STI(R1, 2));

        assert_eq!(state.memory.read(address), 42);
    }

    #[test]
    fn process_lea() {
        let mut state = new_state();

        state = execute(state, LEA(R1, 2));

        assert_eq!(state.registers.read(R1), 0x3000 + 1 + 2);
    }

    #[test]
    fn process_trap_halt() {
        let mut state = new_state();

        state = execute(state, TRAP(TrapVector::HALT));

        assert_eq!(state.running, false);
    }

    fn new_state() -> State {
        let mut state = State::new();
        state.pc = 0x3000;
        state
    }
}

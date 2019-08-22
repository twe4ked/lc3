use crate::trap_vector::TrapVector;
use crate::SignExtend;

/// These instruction types don't map directly to the 4-bit opcodes.
/// Some have been split into multiple enum variants for better ergonimics.
#[derive(Debug, PartialEq)]
pub enum Instruction {
    BR(Condition, u16),
    ADD(Register, Register, Register),
    ADDIMM(Register, Register, u16),
    LD(Register, u16),
    ST(Register, u16),
    JSR(u16),
    JSRR(Register),
    AND(Register, Register, Register),
    ANDIMM(u16, Register, Register),
    LDR(Register, Register, u16),
    STR(Register, Register, u16),
    UNUSED,
    NOT(Register, Register),
    LDI(Register, u16),
    STI(Register, u16),
    JMP(Register),
    RESERVED,
    LEA(Register, u16),
    TRAP(TrapVector),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
}

impl Register {
    fn from(n: u16) -> Register {
        match n {
            0 => Register::R0,
            1 => Register::R1,
            2 => Register::R2,
            3 => Register::R3,
            4 => Register::R4,
            5 => Register::R5,
            6 => Register::R6,
            7 => Register::R7,
            _ => panic!("bad register"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Condition {
    pub p: bool,
    pub z: bool,
    pub n: bool,
}

impl Instruction {
    pub fn decode(instruction: u16) -> Self {
        let value = instruction >> 12;

        match value {
            0x00 => {
                let n = ((instruction >> 11) & 0x1) == 1;
                let z = ((instruction >> 10) & 0x1) == 1;
                let p = ((instruction >> 9) & 0x1) == 1;
                let pc_offset = instruction & 0x1ff;

                Instruction::BR(Condition { n, z, p }, pc_offset)
            }

            0x01 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);
                let r2 = Register::from(instruction & 0x7);
                let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                if immediate_flag {
                    Instruction::ADDIMM(r0, r1, immediate_value)
                } else {
                    Instruction::ADD(r0, r1, r2)
                }
            }

            0x02 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let pc_offset = instruction & 0x1ff;

                Instruction::LD(r0, pc_offset)
            }

            0x03 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let pc_offset = instruction & 0x1ff;

                Instruction::ST(r0, pc_offset)
            }

            0x04 => {
                let use_pc_offset = ((instruction >> 11) & 1) == 1;
                let r0 = Register::from((instruction >> 6) & 7);
                let pc_offset = instruction & 0x7ff;

                if use_pc_offset {
                    Instruction::JSR(pc_offset)
                } else {
                    Instruction::JSRR(r0)
                }
            }

            0x05 => {
                let immediate_flag = ((instruction >> 5) & 1) == 1;
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);
                let r2 = Register::from((instruction) & 0x7);

                if immediate_flag {
                    Instruction::ANDIMM(immediate_value, r0, r1)
                } else {
                    Instruction::AND(r0, r1, r2)
                }
            }

            0x06 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);
                let offset = (instruction) & 0x3f;

                Instruction::LDR(r0, r1, offset)
            }

            0x07 => {
                let sr = Register::from((instruction >> 9) & 0x7);
                let base_r = Register::from((instruction >> 6) & 0x7);
                let offset = instruction & 0x3f;

                Instruction::STR(sr, base_r, offset)
            }

            0x08 => Instruction::UNUSED,

            0x09 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);

                Instruction::NOT(r0, r1)
            }

            0x0a => {
                let dr = Register::from((instruction >> 9) & 0x7);
                let pc_offset = (instruction & 0x1ff).sign_extend(9);

                Instruction::LDI(dr, pc_offset)
            }

            0x0b => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let pc_offset = instruction & 0x1ff;

                Instruction::STI(r0, pc_offset)
            }

            0x0c => {
                let r0 = Register::from((instruction >> 6) & 0x7);

                Instruction::JMP(r0)
            }

            0x0d => Instruction::RESERVED,

            0x0e => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let pc_offset = instruction & 0x1ff;

                Instruction::LEA(r0, pc_offset)
            }

            0x0f => {
                let trap_vector = TrapVector::decode(instruction);

                Instruction::TRAP(trap_vector)
            }

            _ => unreachable!("bad instruction: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Instruction::{self, *};
    use super::Register::*;
    use super::{Condition, TrapVector};

    fn assert_decode(instruction: u16, expected: Instruction) {
        assert_eq!(Instruction::decode(instruction), expected);
    }

    #[test]
    fn process_add_immediate() {
        assert_decode(0b0001_010_001_1_00001, ADDIMM(R2, R1, 1));
        //              ^    ^   `r1 ^ ^
        //              `add |       | ` literal 1
        //                   ` r2    `immediate
    }

    #[test]
    fn process_add_register() {
        assert_decode(0b0001_010_001_0_00_000, ADD(R2, R1, R0));
        //              ^    ^   `r1 ^ ^  `r0
        //              `add |       | ` unused
        //                   |       `register
        //                   ` r2 (destination)
    }

    #[test]
    fn process_ldi() {
        assert_decode(0b1010_000_000000001, LDI(R0, 1));
        //              ^    `r0 ^
        //              `LDI     `pc_offset
    }

    #[test]
    fn process_jmp() {
        assert_decode(0b1100_000_010_000000, JMP(R2));
        //              ^        `register
        //              `JMP
    }

    #[test]
    fn process_jmp_ret() {
        assert_decode(0b1100_000_111_000000, JMP(R7));
        //              ^        `register
        //              `JMP
    }

    #[test]
    fn process_br_n() {
        let condition = Condition {
            n: true,
            z: false,
            p: false,
        };
        assert_decode(0b0000_1_0_0_000000101, BR(condition, 5));
        //              ^    `n    `pc_offset (5)
        //              `BR
    }

    #[test]
    fn process_br_z() {
        let condition = Condition {
            n: false,
            z: true,
            p: false,
        };
        assert_decode(0b0000_0_1_0_000000101, BR(condition, 5));
        //              ^      `z  `pc_offset (5)
        //              `BR
    }

    #[test]
    fn process_br_p() {
        let condition = Condition {
            n: false,
            z: false,
            p: true,
        };
        assert_decode(0b0000_0_0_1_000000101, BR(condition, 5));
        //              ^        ^ `pc_offset (5)
        //              `BR      `p
    }

    #[test]
    fn process_br_nz() {
        let condition = Condition {
            n: true,
            z: true,
            p: false,
        };
        assert_decode(0b0000_1_1_0_000000101, BR(condition, 5));
        //              ^    ^ `z  `pc_offset (5)
        //              `BR  `n
    }

    #[test]
    fn process_ld() {
        assert_decode(0b0010_011_000000101, LD(R3, 5));
        //              ^    `r3 ^
        //              `LD      `pc_offset (5)
    }

    #[test]
    fn process_st() {
        assert_decode(0b0011_011_000000101, ST(R3, 5));
        //              ^    `r3 ^
        //              `LD      `pc_offset (5)
    }

    #[test]
    fn process_jsr() {
        assert_decode(0b0100_0_00_011_000000, JSRR(R3));
        //              ^    ^ ^  `r3 `unused
        //              `JSR | `unused
        //                   `use pc_offset
    }

    #[test]
    fn process_jsrr() {
        assert_decode(0b0100_1_10000000011, JSR(1027));
        //              ^    ^ `pc_offset (1027)
        //              `JSR |
        //                   `use pc_offset
    }

    #[test]
    fn process_and() {
        assert_decode(0b0101_001_010_0_00_011, AND(R1, R2, R3));
        //              ^    `r0 `r1 ^    `r2
        //              `AND         `immediate_flag
    }

    #[test]
    fn process_andimm() {
        assert_decode(0b0101_001_010_1_00101, ANDIMM(5, R1, R2));
        //              ^    `r0 `r1 ^ `immediate_value (5)
        //              `AND         `immediate_flag
    }

    #[test]
    fn process_ldr() {
        assert_decode(0b0110_001_010_000011, LDR(R1, R2, 3));
        //              ^    `r0 `r1 `offset (3)
        //              `AND
    }

    #[test]
    fn process_str() {
        assert_decode(0b0111_001_010_000011, STR(R1, R2, 3));
        //              ^    `r0 `r1 `offset (3)
        //              `AND
    }

    #[test]
    fn process_not() {
        assert_decode(0b1001_001_010_1_11111, NOT(R1, R2));
        //              ^    `r0 `r1
        //              `NOT
    }

    #[test]
    fn process_sti() {
        assert_decode(0b1011_001_000000010, STI(R1, 2));
        //              ^    `r0 `pc_offset (2)
        //              `STI
    }

    #[test]
    fn process_lea() {
        assert_decode(0b1110_001_000000010, LEA(R1, 2));
        //              ^    `r0 `pc_offset (2)
        //              `LEA
    }

    #[test]
    fn process_trap_halt() {
        assert_decode(0b1111_0000_00100101, TRAP(TrapVector::HALT));
        //              ^         `HALT (0x25)
        //              `TRAP
    }
}

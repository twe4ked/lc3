use crate::trap_vector::TrapVector;
use crate::SignExtend;

#[derive(Debug)]
pub enum Instruction {
    BR(bool, bool, bool, u16),
    ADD(Register, Register, Register, bool, u16),
    LD(Register, u16),
    ST(Register, u16),
    JSR(bool, u16, Register),
    AND(bool, u16, Register, Register, Register),
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

#[derive(Debug, Clone, Copy)]
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

impl Instruction {
    pub fn decode(instruction: u16) -> Self {
        let value = instruction >> 12;

        match value {
            0x00 => {
                let n = ((instruction >> 11) & 0x1) == 1;
                let z = ((instruction >> 10) & 0x1) == 1;
                let p = ((instruction >> 9) & 0x1) == 1;
                let pc_offset = instruction & 0x1ff;

                Instruction::BR(n, z, p, pc_offset)
            }

            0x01 => {
                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);
                let r2 = Register::from(instruction & 0x7);
                let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                Instruction::ADD(r0, r1, r2, immediate_flag, immediate_value)
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

                Instruction::JSR(use_pc_offset, pc_offset, r0)
            }

            0x05 => {
                let immediate_flag = ((instruction >> 5) & 1) == 1;
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                let r0 = Register::from((instruction >> 9) & 0x7);
                let r1 = Register::from((instruction >> 6) & 0x7);
                let r2 = Register::from((instruction) & 0x7);

                Instruction::AND(immediate_flag, immediate_value, r0, r1, r2)
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

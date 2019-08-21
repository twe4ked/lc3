use crate::trap_vector::TrapVector;
use crate::SignExtend;

#[derive(Debug)]
pub enum Opcode {
    BR(bool, bool, bool),
    ADD(u16, u16, bool),
    LD(u16, u16),
    ST(u16, u16),
    JSR(bool, u16, u16),
    AND(bool, u16, u16, u16, u16),
    LDR(u16, u16, u16),
    STR(u16, u16, u16),
    UNUSED,
    NOT(u16, u16),
    LDI(u16, u16),
    STI(u16, u16),
    JMP(u16),
    RESERVED,
    LEA(u16, u16),
    TRAP(TrapVector),
}

impl Opcode {
    pub fn from_instruction(instruction: u16) -> Opcode {
        let value = instruction >> 12;

        match value {
            0x00 => {
                let n = ((instruction >> 11) & 0x1) == 1;
                let z = ((instruction >> 10) & 0x1) == 1;
                let p = ((instruction >> 9) & 0x1) == 1;

                Opcode::BR(n, z, p)
            }

            0x01 => {
                let r0 = (instruction >> 9) & 0x7;
                let r1 = (instruction >> 6) & 0x7;
                let immediate_flag = ((instruction >> 5) & 0x1) == 0x1;

                Opcode::ADD(r0, r1, immediate_flag)
            }

            0x02 => {
                let r0 = (instruction >> 9) & 0x7;
                let pc_offset = instruction & 0x1ff;

                Opcode::LD(r0, pc_offset)
            }

            0x03 => {
                let r0 = (instruction >> 9) & 0x7;
                let pc_offset = instruction & 0x1ff;

                Opcode::ST(r0, pc_offset)
            }

            0x04 => {
                let use_pc_offset = ((instruction >> 11) & 1) == 1;
                let r0 = (instruction >> 6) & 7;
                let pc_offset = instruction & 0x7ff;

                Opcode::JSR(use_pc_offset, pc_offset, r0)
            }

            0x05 => {
                let immediate_flag = ((instruction >> 5) & 1) == 1;
                let immediate_value = (instruction & 0x1f).sign_extend(5);

                let r0 = (instruction >> 9) & 0x7;
                let r1 = (instruction >> 6) & 0x7;
                let r2 = (instruction) & 0x7;

                Opcode::AND(immediate_flag, immediate_value, r0, r1, r2)
            }

            0x06 => {
                let r0 = (instruction >> 9) & 0x7;
                let r1 = (instruction >> 6) & 0x7;
                let offset = (instruction) & 0x3f;

                Opcode::LDR(r0, r1, offset)
            }

            0x07 => {
                let sr = (instruction >> 9) & 0x7;
                let base_r = (instruction >> 6) & 0x7;
                let offset = instruction & 0x3f;

                Opcode::STR(sr, base_r, offset)
            }

            0x08 => Opcode::UNUSED,

            0x09 => {
                let r0 = (instruction >> 9) & 0x7;
                let r1 = (instruction >> 6) & 0x7;

                Opcode::NOT(r0, r1)
            }

            0x0a => {
                let dr = (instruction >> 9) & 0x7;
                let pc_offset = (instruction & 0x1ff).sign_extend(9);

                Opcode::LDI(dr, pc_offset)
            }

            0x0b => {
                let r0 = (instruction >> 9) & 0x7;
                let pc_offset = instruction & 0x1ff;

                Opcode::STI(r0, pc_offset)
            }

            0x0c => {
                let r0 = (instruction >> 6) & 0x7;

                Opcode::JMP(r0)
            }

            0x0d => Opcode::RESERVED,

            0x0e => {
                let r0 = (instruction >> 9) & 0x7;
                let pc_offset = instruction & 0x1ff;

                Opcode::LEA(r0, pc_offset)
            }

            0x0f => {
                let trap_vector = TrapVector::from_instruction(instruction);

                Opcode::TRAP(trap_vector)
            }

            _ => unreachable!("bad opcode: {}", value),
        }
    }
}

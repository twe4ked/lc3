#[derive(Debug)]
pub enum Opcode {
    BR,
    ADD,
    LD,
    ST,
    JSR,
    AND,
    LDR,
    STR,
    UNUSED,
    NOT,
    LDI,
    STI,
    JMP,
    RESERVED,
    LEA,
    TRAP,
}

impl Opcode {
    pub fn from_instruction(instruction: u16) -> Opcode {
        let value = instruction >> 12;

        match value {
            0x00 => Opcode::BR,
            0x01 => Opcode::ADD,
            0x02 => Opcode::LD,
            0x03 => Opcode::ST,
            0x04 => Opcode::JSR,
            0x05 => Opcode::AND,
            0x06 => Opcode::LDR,
            0x07 => Opcode::STR,
            0x08 => Opcode::UNUSED,
            0x09 => Opcode::NOT,
            0x0a => Opcode::LDI,
            0x0b => Opcode::STI,
            0x0c => Opcode::JMP,
            0x0d => Opcode::RESERVED,
            0x0e => Opcode::LEA,
            0x0f => Opcode::TRAP,
            _ => unreachable!("bad opcode: {}", value),
        }
    }
}

#[derive(Debug)]
pub(crate) enum TrapVector {
    GETC,
    OUT,
    PUTS,
    IN,
    PUTSP,
    HALT,
}

impl TrapVector {
    pub(crate) fn from_instruction(instruction: u16) -> TrapVector {
        let value = instruction & 0xFF;

        match value {
            0x20 => TrapVector::GETC,
            0x21 => TrapVector::OUT,
            0x22 => TrapVector::PUTS,
            0x23 => TrapVector::IN,
            0x24 => TrapVector::PUTSP,
            0x25 => TrapVector::HALT,
            _ => unreachable!("bad TRAP vector: {:x}", value),
        }
    }
}

#[derive(Debug)]
pub(crate) enum TrapVector {
    GETC, OUT, PUTS, IN, PUTSP, HALT,
}

impl TrapVector {
    pub(crate) fn from_instruction(instruction: u16) -> Result<TrapVector, String> {
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

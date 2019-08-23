use crate::instruction::Register;

pub struct Registers {
    registers: [u16; 8],
}

impl Registers {
    pub fn new() -> Self {
        Self { registers: [0; 8] }
    }

    pub fn read(&self, register: Register) -> u16 {
        self.registers[register as usize]
    }

    pub fn write(&mut self, register: Register, value: u16) {
        self.registers[register as usize] = value
    }

    pub fn registers(&self) -> [u16; 8] {
        self.registers
    }
}

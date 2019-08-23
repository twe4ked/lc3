pub mod memory;

use crate::cpu::execute;
use crate::instruction::{Instruction, Register};
use memory::Memory;

pub struct State {
    pub memory: Memory,
    registers: [u16; 8],
    pub pc: u16,
    pub condition: Condition,
    pub running: bool,
    pub debug_continue: bool,
    pub break_address: Option<u16>,
}

impl State {
    pub fn new() -> Self {
        Self {
            memory: Memory::new(),
            registers: [0; 8],
            pc: 0x3000,
            condition: Condition::P,
            running: true,
            debug_continue: false,
            break_address: None,
        }
    }

    pub fn update_flags(&mut self, r: Register) -> &Self {
        if self.read_register(r) == 0 {
            self.condition = Condition::Z;
        } else if (self.read_register(r) >> 15) == 1 {
            // NOTE: A 1 in the left-most bit indicates negative
            self.condition = Condition::N;
        } else {
            self.condition = Condition::P;
        }

        self
    }

    pub fn read_register(&self, register: Register) -> u16 {
        self.registers[register as usize]
    }

    pub fn write_register(&mut self, register: Register, value: u16) {
        self.registers[register as usize] = value
    }

    pub fn step(mut self) -> Self {
        let instruction = self.memory.read(self.pc);
        let instruction = Instruction::decode(instruction);
        execute(self, instruction)
    }

    pub fn registers(&self) -> [u16; 8] {
        self.registers
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    P,
    Z,
    N,
}

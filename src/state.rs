pub mod memory;
pub mod registers;

use crate::cpu::execute;
use crate::instruction::{Instruction, Register};
use memory::Memory;
use registers::Registers;

pub struct State {
    pub memory: Memory,
    pub registers: Registers,
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
            registers: Registers::new(),
            pc: 0x0000,
            condition: Condition::P,
            running: true,
            debug_continue: false,
            break_address: None,
        }
    }

    pub fn update_flags(&mut self, r: Register) -> &Self {
        if self.registers.read(r) == 0 {
            self.condition = Condition::Z;
        } else if (self.registers.read(r) >> 15) == 1 {
            // NOTE: A 1 in the left-most bit indicates negative
            self.condition = Condition::N;
        } else {
            self.condition = Condition::P;
        }

        self
    }

    pub fn step(mut self) -> Self {
        let instruction = self.memory.read(self.pc);
        let instruction = Instruction::decode(instruction);
        execute(self, instruction)
    }

    pub fn registers(&self) -> [u16; 8] {
        self.registers.registers()
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    P,
    Z,
    N,
}

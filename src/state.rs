pub mod instructions;

use instructions::{execute, Instruction};
use libc;
use nix::sys::{
    select::{select, FdSet},
    time::{TimeVal, TimeValLike},
};
use std::io::{self, Read};

pub type Memory = [u16; u16::max_value() as usize];

enum MemoryMappedRegister {
    KBSR = 0xfe00, // keyboard status register
    KBDR = 0xfe02, // keyboard data register
}

pub struct State {
    pub memory: Memory,
    pub registers: [u16; 8],
    pub pc: u16,
    pub condition: Condition,
    pub running: bool,
    pub debug_continue: bool,
    pub break_address: Option<u16>,
}

impl State {
    pub fn new() -> State {
        State {
            memory: [0; u16::max_value() as usize],
            registers: [0; 8],
            pc: 0x3000,
            condition: Condition::P,
            running: true,
            debug_continue: false,
            break_address: None,
        }
    }

    pub fn read_memory(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegister::KBSR as u16 {
            if check_key() {
                self.memory[MemoryMappedRegister::KBSR as usize] = 1 << 15;
                self.memory[MemoryMappedRegister::KBDR as usize] = get_char();
            } else {
                self.memory[MemoryMappedRegister::KBSR as usize] = 0;
            }
        }

        if address < u16::max_value() {
            self.memory[address as usize]
        } else {
            0
        }
    }

    pub fn update_flags(&mut self, r: u16) -> &State {
        if self.registers[r as usize] == 0 {
            self.condition = Condition::Z;
        } else if (self.registers[r as usize] >> 15) == 1 {
            // NOTE: A 1 in the left-most bit indicates negative
            self.condition = Condition::N;
        } else {
            self.condition = Condition::P;
        }

        self
    }

    pub fn step(mut self) -> State {
        let instruction = self.read_memory(self.pc);
        let instruction = Instruction::decode(instruction);
        execute(self, instruction)
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    P = 1,
    Z = 1 << 1,
    N = 1 << 2,
}

fn check_key() -> bool {
    let mut readfds = FdSet::new();
    readfds.insert(libc::STDIN_FILENO);

    match select(None, &mut readfds, None, None, &mut TimeVal::zero()) {
        Ok(value) => value == 1,
        Err(_) => false,
    }
}

fn get_char() -> u16 {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer).unwrap();

    u16::from(buffer[0])
}

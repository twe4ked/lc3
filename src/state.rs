use std::fmt;

pub(crate) type Memory = [u16; std::u16::MAX as usize];

enum MemoryMappedRegister {
    KBSR = 0xfe00, // keyboard status register
    KBDR = 0xfe02, // keyboard data register
}

pub(crate) struct State {
    pub(crate) memory: Memory,
    pub(crate) registers: [u16; 8],
    pub(crate) pc: u16,
    pub(crate) condition: Condition,
    pub(crate) running: bool,
    pub(crate) debug_continue: bool,
    pub(crate) debug: bool,
    pub(crate) break_address: Option<u16>,
}

impl State {
    pub(crate) fn new(debug: bool) -> State {
        State {
            memory: [0; std::u16::MAX as usize],
            registers: [0; 8],
            pc: 0x3000,
            condition: Condition::P,
            running: true,
            debug_continue: false,
            debug: debug,
            break_address: None,
        }
    }

    pub(crate) fn read_memory(&self, address: u16) -> u16 {
        if address == MemoryMappedRegister::KBSR as u16 {
            panic!("KBSR");
        } else if address == MemoryMappedRegister::KBDR  as u16 {
            panic!("KBDR");
        }

        if address < std::u16::MAX {
            self.memory[address as usize]
        } else {
            0
        }
    }
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "registers: {:?}, condition: {:?}", self.registers, self.condition)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Condition {
    P = 1 << 0,
    Z = 1 << 1,
    N = 1 << 2,
}

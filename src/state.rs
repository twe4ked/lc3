use std::fmt;

pub type Memory = [u16; std::u16::MAX as usize];

pub struct State {
    pub memory: Memory,
    pub registers: [u16; 8],
    pub pc: u16,
    pub condition: Condition,
    pub running: bool,
    pub debug_continue: bool,
    pub debug: bool,
    pub break_address: Option<u16>,
}

impl State {
    pub fn new(debug: bool) -> State {
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
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "registers: {:?}, condition: {:?}", self.registers, self.condition)
    }
}

#[derive(Debug, PartialEq)]
pub enum Condition {
    P = 1 << 0,
    Z = 1 << 1,
    N = 1 << 2,
}

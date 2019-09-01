use libc;
use nix::sys::{
    select::{select, FdSet},
    time::{TimeVal, TimeValLike},
};
use std::io::{self, Read};

pub struct Memory {
    memory: [u16; u16::max_value() as usize],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0; u16::max_value() as usize],
        }
    }

    pub fn read(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegister::KBSR as u16 {
            if check_key() {
                self.memory[MemoryMappedRegister::KBSR as usize] = 1 << 15;
                self.memory[MemoryMappedRegister::KBDR as usize] = get_char();
            } else {
                self.memory[MemoryMappedRegister::KBSR as usize] = 0;
            }
        }

        self.memory[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = value;
    }
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
    io::stdin()
        .read_exact(&mut buffer)
        .expect("unable to read from STDIN");

    u16::from(buffer[0])
}

enum MemoryMappedRegister {
    // Keyboard status register. The ready bit (bit [15]) indicates if the keyboard has received a
    // new character.
    KBSR = 0xfe00,

    // Keyboard data register. Bits [7:0] contain the last character typed on the keyboard.
    KBDR = 0xfe02,

    // Display status register. The ready bit (bit [15]) indicates if the display device is ready
    // to receive another character to print on the screen.
    #[allow(unused)]
    DSR = 0xfe04,

    // Display data register. A character written in the low byte of this register will be
    // displayed on the screen.
    #[allow(unused)]
    DDR = 0xfe06,

    // Machine control register. Bit [15] is the clock enable bit. When cleared, instruction
    // processing stops.
    #[allow(unused)]
    MCR = 0xfffe,
}

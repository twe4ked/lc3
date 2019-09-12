use nix::sys::select::{select, FdSet};
use nix::sys::time::{TimeVal, TimeValLike};
use std::io::{self, Read};

// Keyboard status register. The ready bit (bit [15]) indicates if the keyboard has received a new
// character.
const KBSR: u16 = 0xfe00;

// Keyboard data register. Bits [7:0] contain the last character typed on the keyboard.
const KBDR: u16 = 0xfe02;

// Display status register. The ready bit (bit [15]) indicates if the display device is ready to
// receive another character to print on the screen.
const DSR: u16 = 0xfe04;

// Display data register. A character written in the low byte of this register will be displayed on
// the screen.
const DDR: u16 = 0xfe06;

// Machine control register. Bit [15] is the clock enable bit. When cleared, instruction processing
// stops.
const MCR: u16 = 0xfffe;

pub struct Memory {
    memory: [u16; u16::max_value() as usize],
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = [0; u16::max_value() as usize];
        memory[DSR as usize] = 1 << 15;
        memory[MCR as usize] = 1 << 15;

        Self { memory }
    }

    pub fn read(&mut self, address: u16) -> u16 {
        if KBSR == address {
            let value = if check_key() { 1 << 15 } else { 0 };
            self.memory[KBSR as usize] = value;
            value
        } else if KBDR == address {
            let kbsr = self.memory[KBSR as usize];
            if ((kbsr >> 15) & 0x1) == 1 {
                get_char()
            } else {
                0
            }
        } else if DSR == address {
            unimplemented!("DSR")
        } else if DDR == address {
            unimplemented!("DDR")
        } else if MCR == address {
            unimplemented!("MCR")
        } else {
            self.memory[address as usize]
        }
    }

    pub fn write(&mut self, address: u16, value: u16) {
        self.memory[address as usize] = value;
    }
}

fn check_key() -> bool {
    const STDIN_FILENO: i32 = 0;

    let mut readfds = FdSet::new();
    readfds.insert(STDIN_FILENO);

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

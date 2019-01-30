use libc;
use std::io::Read;
use std::{fmt, io};

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
    pub(crate) break_address: Option<u16>,
}

impl State {
    pub(crate) fn new() -> State {
        State {
            memory: [0; std::u16::MAX as usize],
            registers: [0; 8],
            pc: 0x3000,
            condition: Condition::P,
            running: true,
            debug_continue: false,
            break_address: None,
        }
    }

    pub(crate) fn read_memory(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegister::KBSR as u16 {
            if check_key() {
                self.memory[MemoryMappedRegister::KBSR as usize] = 1 << 15;
                self.memory[MemoryMappedRegister::KBDR as usize] = get_char();
            } else {
                self.memory[MemoryMappedRegister::KBSR as usize] = 0;
            }
        }

        if address < std::u16::MAX {
            self.memory[address as usize]
        } else {
            0
        }
    }

    pub(crate) fn update_flags(&mut self, r: u16) -> &State {
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
}

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "registers: {:?}, condition: {:?}",
            self.registers, self.condition
        )
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Condition {
    P = 1,
    Z = 1 << 1,
    N = 1 << 2,
}

fn check_key() -> bool {
    unsafe {
        let mut readfds = std::mem::uninitialized::<libc::fd_set>();
        libc::FD_ZERO(&mut readfds);
        libc::FD_SET(libc::STDIN_FILENO, &mut readfds);

        let mut writefds = std::mem::uninitialized::<libc::fd_set>();
        libc::FD_ZERO(&mut writefds);

        let mut errorfds = std::mem::uninitialized::<libc::fd_set>();
        libc::FD_ZERO(&mut errorfds);

        libc::select(
            libc::STDOUT_FILENO,
            &mut readfds,
            &mut writefds,
            &mut errorfds,
            &mut libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        ) != 0
    }
}

fn get_char() -> u16 {
    let mut buffer = [0; 1];
    io::stdin().read_exact(&mut buffer).unwrap();

    u16::from(buffer[0])
}

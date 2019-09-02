use crate::instruction::Instruction;
use crate::state::State;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpListener;

pub struct Debugger {
    debug_continue: bool,
    break_address: Option<u16>,
}

#[derive(PartialEq, Debug)]
enum Command {
    Continue,
    Registers,
    Flags,
    Disassemble,
    Read(u16),
    BreakAddress(u16),
    Help,
    Exit,
    Unknown(String),
    Error(String),
}

impl Debugger {
    pub fn new() -> Self {
        Debugger {
            debug_continue: false,
            break_address: None,
        }
    }

    pub fn step(&mut self, mut state: State) {
        let listener = TcpListener::bind("127.0.0.1:6379").expect("unable to bind to port 6379");

        eprintln!("Waiting for connection...");

        match listener.accept() {
            Ok((stream, address)) => {
                eprintln!("Debug client connected: {:?}", address);

                while state.running {
                    while state.running && !self.debug_continue && self.should_break(state.pc) {
                        self.debug_continue = false;

                        let mut line = String::new();
                        let command = match BufReader::new(&stream).read_line(&mut line) {
                            Ok(_) => parse(line.trim().as_ref()),
                            Err(_) => Command::Error("Unable to read line".to_string()),
                        };

                        let response = self.handle_command(&mut state, command);

                        BufWriter::new(&stream)
                            .write_all(format!("{}\n", response).as_bytes())
                            .expect("unable to write to socket");
                    }

                    self.debug_continue = false;

                    state = state.step();
                }
            }
            Err(e) => eprintln!("Couldn't get client: {:?}", e),
        }
    }

    fn should_break(&mut self, pc: u16) -> bool {
        match self.break_address {
            Some(break_address) => {
                if break_address == pc {
                    self.break_address = None;
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }

    fn handle_command(&mut self, state: &mut State, command: Command) -> String {
        match command {
            Command::Continue => {
                self.debug_continue = true;
                format!("PC {:#04x}", state.pc)
            }

            Command::Flags => format!("{:?}", state.condition),

            Command::Registers => state
                .registers()
                .iter()
                .enumerate()
                .map(|(i, register)| format!("R{}: {:#04x}", i, register))
                .collect::<Vec<String>>()
                .join("\n"),

            Command::Disassemble => {
                let instruction = state.memory.read(state.pc);

                format!(
                    "{:?}, {:08b}_{:08b}",
                    Instruction::decode(instruction),
                    (instruction >> 8) & 0xff,
                    instruction & 0xff
                )
            }

            Command::Read(address) => {
                let value = state.memory.read(address);
                format!("{:#04x}, {:#016b}", value, value)
            }

            Command::BreakAddress(address) => {
                self.break_address = Some(address);
                format!("Break address set to {:#04x}", address)
            }

            Command::Help => [
                "c, continue               Continue execution.",
                "r, registers              Print registers.",
                "f, flags                  Print flags.",
                "d, disassemble            Disassemble current instruction.",
                "   read <addr>            Read and display memory address. e.g. read 0x3000",
                "   break-address <addr>   Break at address. e.g. break-address 0x3000",
            ]
            .join("\n"),

            Command::Exit => {
                state.running = false;
                "Exiting...".to_string()
            }

            Command::Unknown(line) => format!("Unknown command {:?}", line),

            Command::Error(message) => message,
        }
    }
}

fn parse(line: &str) -> Command {
    match line {
        "c" | "continue" => Command::Continue,
        "f" | "flags" => Command::Flags,
        "r" | "registers" => Command::Registers,
        "d" | "disassemble" => Command::Disassemble,
        "h" | "help" => Command::Help,
        "exit" => Command::Exit,
        line => {
            match parse_hex_after_pattern("read 0x", line) {
                Some(address) => return Command::Read(address),
                None => (),
            }
            match parse_hex_after_pattern("break-address 0x", line) {
                Some(address) => return Command::BreakAddress(address),
                None => (),
            }

            Command::Unknown(line.trim().to_string())
        }
    }
}

fn parse_hex_after_pattern(pattern: &str, line: &str) -> Option<u16> {
    if line.starts_with(pattern) {
        match line.find(pattern) {
            Some(_) => {
                let (_, address) = line.split_at(pattern.len());
                if address.len() > 0
                    && address.len() <= 4
                    && address.bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Some(
                        u16::from_str_radix(address, 16).expect("unable to parse address"),
                    );
                }
            }
            None => (),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_after_pattern() {
        for command in vec!["read", "read 0x", "read 0x12345", "read 0x1z", "a read 0x1"] {
            assert_eq!(parse_hex_after_pattern("read 0x", command), None);
        }

        assert_eq!(parse_hex_after_pattern("read 0x", "read 0x1"), Some(1));
        assert_eq!(
            parse_hex_after_pattern("read 0x", "read 0x1234"),
            Some(4660)
        );
    }
}

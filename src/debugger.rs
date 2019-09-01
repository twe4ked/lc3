use crate::instruction::Instruction;
use crate::state::State;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::TcpListener;

pub fn debug(mut state: State) {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    println!("Waiting for connection...");

    match listener.accept() {
        Ok((stream, address)) => {
            println!("Debug client connected: {:?}", address);
            while state.running {
                let mut should_break = true;

                if let Some(break_address) = state.break_address {
                    if break_address == state.pc {
                        state.break_address = None;
                        should_break = true;
                    } else {
                        should_break = false;
                    }
                }

                while state.running && !state.debug_continue && should_break {
                    lazy_static! {
                        static ref READ_REGEX: Regex =
                            Regex::new(r"^read 0x([a-f0-9]{1,4})$").unwrap();
                        static ref BREAK_ADDRESS_REGEX: Regex =
                            Regex::new(r"^break-address 0x([a-f0-9]{1,4})$").unwrap();
                    }

                    state.debug_continue = false;

                    let mut stream_reader = BufReader::new(&stream);
                    let mut line = String::new();

                    let response = match stream_reader.read_line(&mut line) {
                        Ok(_) => match line.trim().as_ref() {
                            "c" | "continue" => {
                                state.debug_continue = true;
                                format!("PC {:#04x}", state.pc)
                            }

                            "f" | "flags" => {
                                format!("{:?}", state.condition)
                            }

                            "r" | "registers" => {
                                let mut s = vec![];
                                for (i, register) in
                                    state.registers().iter().enumerate()
                                {
                                    s.push(format!("r{}: {:#04x}", i, register));
                                }
                                 s.join("\n")
                            }

                            "d" | "disassemble" => {
                                let instruction: u16 = state.memory.read(state.pc);

                                format!(
                                    "{:?}, {:08b}_{:08b}",
                                    Instruction::decode(instruction),
                                    (instruction >> 8) & 0xff,
                                    instruction & 0xff
                                )
                            }

                            line if READ_REGEX.is_match(line) => {
                                if let Some(address) =
                                    READ_REGEX.captures(line).unwrap().get(1)
                                {
                                    let address =
                                        u16::from_str_radix(address.as_str(), 16)
                                            .unwrap();
                                    let value = state.memory.read(address);
                                        format!("{:#04x}, {:#016b}", value, value)
                                } else {
                                    "Error".to_string()
                                }
                            }

                            line if BREAK_ADDRESS_REGEX.is_match(line) => {
                                if let Some(address) =
                                    BREAK_ADDRESS_REGEX.captures(line).unwrap().get(1)
                                {
                                    let address =
                                        u16::from_str_radix(address.as_str(), 16)
                                            .unwrap();
                                    state.break_address = Some(address);
                                    format!(
                                        "Break address set to {:#04x}",
                                        address
                                    )
                                } else {
                                    "Error".to_string()
                                }
                            }

                            "h" | "help" => {
                                 [
                                    "c, continue               Continue execution.",
                                    "r, registers              Print registers.",
                                    "f, flags                  Print flags.",
                                    "d, disassemble            Disassemble current instruction.",
                                    "   read <addr>            Read and display memory address. e.g. read 0x3000",
                                    "   break-address <addr>   Break at address. e.g. break-address 0x3000",
                                ].join("\n")
                            }

                            "exit" => {
                                state.running = false;
                                "Exiting...".to_string()
                            }

                            _ => {
                                format!("Unknown command {:?}", line.trim())
                            }
                        }
                        Err(_) => String::from("Error reading line"),
                    };

                    BufWriter::new(&stream)
                        .write_all(format!("{}\n", response).as_bytes())
                        .unwrap();
                }

                state.debug_continue = false;

                state = state.step();
            }
        }
        Err(e) => println!("Couldn't get client: {:?}", e),
    }
}

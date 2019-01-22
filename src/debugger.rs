mod disassemble;

use crate::debugger::disassemble::disassemble;
use crate::opcode::Opcode;
use crate::state::*;
use lazy_static::lazy_static;
use regex::Regex;
use rustyline;

pub(crate) fn debug(mut state: State) -> State {
    let mut rl = rustyline::Editor::<()>::new();
    let readline = rl.readline(&format!("{:#04x}> ", state.pc));

    let instruction : u16 = state.read_memory(state.pc);
    let opcode = Opcode::from_instruction(instruction);

    lazy_static! {
        static ref READ_REGEX: Regex = Regex::new(r"^read 0x([a-f0-9]{1,4})$").unwrap();
        static ref BREAK_ADDRESS_REGEX: Regex = Regex::new(r"^break-address 0x([a-f0-9]{1,4})$").unwrap();
    }

    state.debug_continue = false;

    match readline {
        Ok(line) => {
            rl.add_history_entry(line.as_ref());

            match line.as_ref() {
                "c" | "continue" => {
                    state.debug_continue = true;
                }

                "i" | "inspect" => {
                    println!("{:?}, op_code: {:?}, instruction: {:#4x}, {:#016b}", state, opcode, instruction, instruction);
                }

                "d" | "disassemble" => {
                    disassemble(instruction, opcode);
                }

                line if READ_REGEX.is_match(line) => {
                    if let Some(address) = READ_REGEX.captures(line).unwrap().get(1) {
                        let address = u16::from_str_radix(address.as_str(), 16).unwrap();
                        let value = state.read_memory(address);
                        println!("{:#04x}, {:#016b}", value, value);
                    }
                }

                line if BREAK_ADDRESS_REGEX.is_match(line) => {
                    if let Some(address) = BREAK_ADDRESS_REGEX.captures(line).unwrap().get(1) {
                        let address = u16::from_str_radix(address.as_str(), 16).unwrap();
                        state.break_address = Some(address);
                        println!("Break address set to {:#04x}", address);
                    }
                }

                "h" | "help" => {
                    println!("c, continue               Continue execution.");
                    println!("i, inspect                Inspect state.");
                    println!("d, disassemble            Disassemble current instruction.");
                    println!("   read <addr>            Read and display memory address. e.g. read 0x3000");
                    println!("   break-address <addr>   Break at address. e.g. read 0x3000");
                }

                "exit" => {
                    state.running = false;
                }

                "" => {
                    // Allow hitting enter
                }

                _ => {
                    println!("Unknown command {:?}", line);
                }
            }
        },
        Err(rustyline::error::ReadlineError::Interrupted) => {
            state.running = false;
        },
        Err(rustyline::error::ReadlineError::Eof) => {
            state.running = false;
        },
        Err(err) => {
            println!("Error: {:?}", err);
            state.running = false;
        }
    }

    state
}

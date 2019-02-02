mod config;
mod debugger;
mod opcode;
mod process;
mod state;
mod trap_vector;
mod utilities;

pub use crate::config::Config;
use crate::debugger::run as run_debugger;
use crate::process::run as run_processor;
use crate::state::*;
use byteorder::{BigEndian, ReadBytesExt};
use std::error::Error;
use std::fs;
use std::io::BufReader;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut state = State::new();
    state = load_file(config.filename, state)?;

    if config.debug {
        run_debugger(state)
    } else {
        run_processor(state)
    }

    Ok(())
}

fn load_file(filename: String, mut state: State) -> Result<State, std::io::Error> {
    let mut reader = BufReader::new(fs::File::open(filename)?);
    let mut address = usize::from(reader.read_u16::<BigEndian>()?);

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(instruction) => {
                state.memory[address] = instruction;
                address += 1;
            }
            Err(e) => {
                return if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(state)
                } else {
                    Err(e)
                };
            }
        }
    }
}

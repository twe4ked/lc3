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

    let buffer = read_file(config.filename);
    let starting_address = buffer[0];

    let mut i = 0;
    while i < (buffer.len() - 1) {
        state.memory[(starting_address as usize) + i] = buffer[i + 1];
        i += 1
    }

    if config.debug {
        run_debugger(state)
    } else {
        run_processor(state)
    }

    Ok(())
}

fn read_file(filename: String) -> Vec<u16> {
    let file_size = fs::metadata(filename.clone())
        .expect("could not get file meta data")
        .len() as usize;
    if &file_size % 2 != 0 {
        panic!("file was not even number of bytes long");
    }
    let array_size = file_size / 2;

    let file = fs::File::open(filename).expect("failed to open file");
    let mut buf_reader = BufReader::new(file);

    let mut buffer: Vec<u16> = Vec::with_capacity(array_size);
    unsafe {
        buffer.set_len(array_size);
    }
    buf_reader
        .read_u16_into::<BigEndian>(&mut buffer[..])
        .expect("failed to read");

    buffer
}

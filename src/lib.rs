use std::error::Error;
use std::fs;
use std::io::BufReader;
use byteorder::{BigEndian, ReadBytesExt};

mod process;
use crate::process::{process, debug, state::State};

mod config;
pub use crate::config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut state = State::new(config.debug);

    let buffer = read_file(config.filename);
    let starting_address = buffer[0];

    let mut i = 0;
    while i < (buffer.len() - 1) {
        state.memory[(starting_address as usize) + i] = buffer[i + 1];
        i += 1
    }

    while state.running {
        if state.debug {
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
                state = debug(state);
            }

            state.debug_continue = false;
        }

        state = process(state)
    }

    Ok(())
}

fn read_file(filename: String) -> Vec<u16> {
    let file_size = fs::metadata(filename.clone()).expect("could not get file meta data").len() as usize;
    if &file_size % 2 != 0 {
        panic!("file was not even number of bytes long");
    }
    let array_size = file_size/2;

    let file = fs::File::open(filename).expect("failed to open file");
    let mut buf_reader = BufReader::new(file);

    let mut buffer: Vec<u16> = Vec::with_capacity(array_size);
    unsafe { buffer.set_len(array_size); }
    buf_reader.read_u16_into::<BigEndian>(&mut buffer[..]).expect("failed to read");

    buffer
}

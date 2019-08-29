mod cpu;
mod debugger;
mod file_loader;
mod instruction;
mod sign_extend;
mod state;

use crate::{file_loader::read_rom, sign_extend::SignExtend, state::State};
use std::error::Error;

pub fn run(filename: String, debug: bool) -> Result<(), Box<dyn Error>> {
    let mut rom = read_rom(filename)?;
    let mut state = State::new();
    state.load_rom(&mut rom);

    if debug {
        debugger::debug(state)
    } else {
        while state.running {
            state = state.step()
        }
    }

    Ok(())
}

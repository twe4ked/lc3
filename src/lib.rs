mod cpu;
mod debugger;
mod file_loader;
mod instruction;
mod sign_extend;
mod state;

use crate::{file_loader::load_file, sign_extend::SignExtend, state::State};
use std::error::Error;

pub fn run(filename: String, debug: bool) -> Result<(), Box<dyn Error>> {
    let mut state = load_file(filename, State::new())?;

    if debug {
        debugger::debug(state)
    } else {
        while state.running {
            state = state.step()
        }
    }

    Ok(())
}

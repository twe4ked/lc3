mod config;
mod debugger;
mod file_loader;
mod opcode;
mod sign_extend;
mod state;
mod trap_vector;

pub use crate::config::Config;
use crate::debugger::run as run_debugger;
use crate::{file_loader::load_file, sign_extend::SignExtend, state::State};
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut state = load_file(config.filename, State::new())?;

    if config.debug {
        run_debugger(state)
    } else {
        while state.running {
            state = state.process()
        }
    }

    Ok(())
}

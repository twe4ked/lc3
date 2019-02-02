mod config;
mod debugger;
mod file_loader;
mod opcode;
mod process;
mod sign_extend;
mod state;
mod trap_vector;

pub use crate::config::Config;
use crate::debugger::run as run_debugger;
use crate::file_loader::load_file;
use crate::process::run as run_processor;
use crate::sign_extend::SignExtend;
use crate::state::State;
use std::error::Error;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let state = load_file(config.filename, State::new())?;

    if config.debug {
        run_debugger(state)
    } else {
        run_processor(state)
    }

    Ok(())
}

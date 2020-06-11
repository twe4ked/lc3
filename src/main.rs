use clap::{App, Arg};
use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg};
use std::boxed::Box;
use std::error::Error;
use std::process;

fn main() {
    if let Err(e) = run() {
        println!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let matches = App::new("LC-3 VM")
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Runs in debug mode"),
        )
        .arg(
            Arg::with_name("PROGRAM")
                .help("The program to run.")
                .required(true)
                .index(1),
        )
        .get_matches();

    disable_input_buffering()?;

    lc3::run(
        matches.value_of("PROGRAM").unwrap().to_string(),
        matches.is_present("debug"),
    )?;

    Ok(())
}

fn disable_input_buffering() -> Result<(), nix::Error> {
    const STDIN_FILENO: i32 = 0;

    let mut termios = tcgetattr(STDIN_FILENO)?;
    termios.local_flags &= !(LocalFlags::ICANON | LocalFlags::ECHO);

    tcsetattr(0, SetArg::TCSANOW, &termios)?;

    Ok(())
}

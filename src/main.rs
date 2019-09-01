use clap::{App, Arg};
use lc3;
use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg};
use std::process;

fn main() {
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

    disable_input_buffering();

    if let Err(e) = lc3::run(
        matches.value_of("PROGRAM").unwrap().to_string(),
        matches.is_present("debug"),
    ) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

fn disable_input_buffering() {
    const STDIN_FILENO: i32 = 0;

    let mut termios = tcgetattr(STDIN_FILENO).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
    termios.local_flags &= !(LocalFlags::ICANON | LocalFlags::ECHO);

    tcsetattr(0, SetArg::TCSANOW, &termios).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
}

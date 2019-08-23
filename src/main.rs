use clap::{App, Arg};
use lc3;
use std::process;
use termios::*;

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
    let mut termios = Termios::from_fd(libc::STDIN_FILENO).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
    termios.c_lflag &= !(ICANON | ECHO);

    tcsetattr(0, TCSANOW, &termios).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
}

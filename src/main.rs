use lc3::{self, Config};
use std::{env, process};
use termios::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::with(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    disable_input_buffering();

    if let Err(e) = lc3::run(config) {
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

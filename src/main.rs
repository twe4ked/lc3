use lc3;
use lc3::Config;
use std::env;
use std::process;
use termios::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    let termios = Termios::from_fd(libc::STDIN_FILENO).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });

    disable_input_buffering(termios);

    if let Err(e) = lc3::run(config) {
        enable_input_buffering(termios);

        println!("Application error: {}", e);
        process::exit(1);
    }

    enable_input_buffering(termios);
}

fn disable_input_buffering(mut termios: Termios) {
    termios.c_lflag &= !(ICANON | ECHO);

    tcsetattr(0, TCSANOW, &termios).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
}

fn enable_input_buffering(termios: Termios) {
    tcsetattr(0, TCSANOW, &termios).unwrap_or_else(|err| {
        println!("An error occured: {}", err);
        process::exit(1);
    });
}

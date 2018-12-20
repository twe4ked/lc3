use std::env;
use std::process;
use lc3;
use lc3::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    if let Err(e) = lc3::run(config) {
        println!("Application error: {}", e);

        process::exit(1);
    }
}

use std::process;
use clap::Parser;
use minigrep::Config;

fn main() {
    if let Err(e) = minigrep::run(Config::parse()) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

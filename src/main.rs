use alian::benchmark::benchmark;
use alian::repl;
use std::env;

fn main() {
    let option_mode = env::args().nth(1);
    if let Some(mode) = option_mode {
        if mode == "benchmark" {
            benchmark();
            return;
        }
    }
    println!("Welcome to the 👽 programming language!\n");
    repl::start();
}

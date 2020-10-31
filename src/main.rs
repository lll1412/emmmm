use alian::benchmark::benchmark;
use alian::{current_mode, repl, Mode, eval_or_compile};

fn main() {
    let engine = eval_or_compile();
    match current_mode() {
        Mode::Benchmark => benchmark(engine),
        Mode::Run => repl::start(engine),
    }
}

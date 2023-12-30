use std::io;
use std::io::BufRead;

use engine::engine_uci::{EngineUCI, Printer};
use engine::standard_searcher::StandardSearcher;

struct StandardPrinter;

impl Printer for StandardPrinter {
    fn print(s: &str) {
        println!("{s}");
    }
}

fn main() {
    let stdin = io::stdin();
    let mut message = String::new();

    let mut engine = EngineUCI::<StandardSearcher, StandardPrinter>::new();

    while message.trim() != "quit" {
        message.clear();
        stdin
            .lock()
            .read_line(&mut message)
            .expect("failed to read line");

        engine.receive_command(message.trim());
    }
}

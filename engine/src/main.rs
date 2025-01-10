use std::io;
use std::io::BufRead;

use engine::{
    engine_uci::{EngineUCI, StandardPrinter},
    standard_searcher::StandardSearchWorkerPool,
};

fn main() {
    let stdin = io::stdin();
    let mut message = String::new();

    let mut engine = EngineUCI::new(StandardSearchWorkerPool::new(), StandardPrinter);

    while message.trim() != "quit" {
        message.clear();
        stdin
            .lock()
            .read_line(&mut message)
            .expect("failed to read line");

        if !message.is_empty() {
            engine.receive_command(message.trim());
        }
    }
}

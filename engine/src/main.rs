use std::io;
use std::io::BufRead;

use engine::uci::EngineUCI;
use standard_printer::StandardPrinter;
use standard_searcherpool::StandardSearchWorkerPool;

pub mod standard_printer;
pub mod standard_searcherpool;

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

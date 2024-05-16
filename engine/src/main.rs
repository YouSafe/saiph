use std::io;
use std::io::BufRead;

use engine::engine_uci::EngineUCI;
use engine::searcher::StandardPrinter;
use engine::standard_searcher::StandardSearcher;

fn main() {
    let stdin = io::stdin();
    let mut message = String::new();

    let mut engine = EngineUCI::new(StandardSearcher::new(), StandardPrinter);

    while message.trim() != "quit" {
        message.clear();
        stdin
            .lock()
            .read_line(&mut message)
            .expect("failed to read line");

        engine.receive_command(message.trim());
    }
}

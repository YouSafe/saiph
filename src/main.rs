use std::io;
use std::io::BufRead;
use chess_engine::EngineUCI;

fn main() {
    let stdin = io::stdin();
    let mut message = String::new();

    let engine = EngineUCI::new();

    while message.trim() != "quit" {
        message.clear();
        stdin.lock().read_line(&mut message).expect("failed to read line");

        engine.receive_command(&message.trim());
    }
}

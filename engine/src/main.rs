use std::{
    io::{stdin, BufRead},
    sync::mpsc::channel,
    thread::{self, spawn},
};

use engine::{
    uci::{EngineMessage, EngineUCI},
    Printer, ThreadSpawner,
};

struct DefaultSpawner;
impl ThreadSpawner for DefaultSpawner {
    fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(f);
    }
}

struct StdoutPrinter;
impl Printer for StdoutPrinter {
    fn println(s: &str) {
        println!("{s}");
    }
}

fn main() {
    let (engine_tx, engine_rx) = channel();

    spawn({
        let engine_tx = engine_tx.clone();

        move || {
            let mut input = String::new();

            while input.trim() != "quit" {
                input.clear();

                stdin().lock().read_line(&mut input).unwrap();

                while input.ends_with('\n') || input.ends_with('\r') {
                    input.pop();
                }

                engine_tx
                    .send(EngineMessage::Command(input.clone()))
                    .unwrap();
            }
        }
    });

    let engine: EngineUCI<DefaultSpawner, StdoutPrinter> = EngineUCI::new(engine_tx);
    engine.run(engine_rx);
}

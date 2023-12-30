use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use chess_core::board::Board;
use engine::{
    engine_uci::{EngineUCI, Printer},
    search::Search,
    search_limits::SearchLimits,
    searcher::Searcher,
    transposition_table::TranspositionTable,
};
use wasm_bindgen::prelude::*;

use crate::output;

#[wasm_bindgen]
pub struct Engine {
    engine_uci: EngineUCI<WasmSearcher, WasmPrinter>,
}

pub struct WasmPrinter;

impl Printer for WasmPrinter {
    fn print(s: &str) {
        output(s);
    }
}

pub struct WasmSearcher {
    table: Arc<Mutex<TranspositionTable>>,
    stop: Arc<AtomicBool>,
}

impl Searcher for WasmSearcher {
    fn new() -> Self {
        let table = Arc::new(Mutex::new(TranspositionTable::new()));
        let stop = Arc::new(AtomicBool::new(false));

        Self {
            table: table.clone(),
            stop: stop.clone(),
        }
    }

    fn clear_tables(&mut self) {
        self.table.lock().unwrap().clear();
    }

    fn initiate_search(&self, board: Board, limits: SearchLimits) {
        self.stop.store(false, Ordering::SeqCst);
        let stop_ref = self.stop.as_ref();
        let table_ref = &mut self.table.lock().unwrap();

        let mut search = Search::<WasmPrinter>::new(board, table_ref, stop_ref);

        let pick = search.find_best_move(limits);
        output(format!("bestmove {}", pick.chess_move.unwrap()).as_str());
    }

    fn stop_search(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        unimplemented!();
    }
}

#[wasm_bindgen]
impl Engine {
    pub fn new() -> Engine {
        Engine {
            engine_uci: EngineUCI::new(),
        }
    }

    pub fn receive_command(&mut self, message: &str) {
        self.engine_uci.receive_command(message);
    }
}

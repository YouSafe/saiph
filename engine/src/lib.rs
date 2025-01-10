use board::Board;
use search_limits::SearchLimits;

pub mod clock;
pub mod engine_uci;
pub mod evaluation;
pub mod move_ordering;
pub mod nnue;
pub mod piece_square_table;
pub mod pv_table;
pub mod search;
pub mod search_limits;
pub mod transposition_table;

pub mod board;
pub mod move_generation;
pub mod tables;
pub mod types;
pub mod uci_move;
mod zobrist;

#[cfg(not(target_arch = "wasm32"))]
pub mod standard_searcher;

pub trait Printer {
    fn print(&self, s: &str);
}

pub trait SearchWorkerPool {
    fn clear_tables(&mut self);
    fn initiate_search(&self, board: Board, limits: SearchLimits);
    fn stop_search(&mut self);
}

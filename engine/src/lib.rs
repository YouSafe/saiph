use board::Board;
use search_limits::SearchLimits;

pub mod clock;
pub mod evaluation;
pub mod move_ordering;
pub mod nnue;
pub mod piece_square_table;
pub mod pv_table;
pub mod search;
pub mod search_limits;
pub mod transposition_table;
pub mod uci;

pub mod attacks;
pub mod board;
pub mod move_generation;
pub mod types;
pub mod uci_move;
mod zobrist;

pub trait Printer {
    fn print(&self, s: &str);
}

pub trait SearcherPool {
    fn clear_tables(&mut self);
    fn initiate_search(&self, board: Board, limits: SearchLimits);
    fn stop_search(&mut self);
}

use crate::search_limits::SearchLimits;

use chess_core::board::Board;

pub trait Searcher {
    fn new() -> Self;
    fn clear_tables(&mut self);
    fn initiate_search(&self, board: Board, limits: SearchLimits);
    fn stop_search(&mut self);
}

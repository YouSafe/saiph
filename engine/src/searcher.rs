use crate::{engine_uci::Printer, search_limits::SearchLimits};

use chess_core::board::Board;

pub trait Searcher {
    fn clear_tables(&mut self);
    fn initiate_search(&self, board: Board, limits: SearchLimits);
    fn stop_search(&mut self);
}

pub struct StandardPrinter;

impl Printer for StandardPrinter {
    fn print(&self, s: &str) {
        println!("{s}");
    }
}

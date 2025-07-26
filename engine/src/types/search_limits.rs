use crate::types::color::PerColor;

use super::uci_move::UCIMove;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchLimits {
    pub time: TimeLimit,
    pub depth: Option<u8>,
    pub mate: Option<u8>,
    pub nodes: Option<u64>,
    pub search_moves: Vec<UCIMove>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Millis(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TimeLimit {
    #[default]
    Infinite,
    Fixed {
        move_time: Millis,
    },
    Dynamic {
        time_left: PerColor<Millis>,
        increment: PerColor<Millis>,
        moves_to_go: Option<u8>,
    },
    External,
}

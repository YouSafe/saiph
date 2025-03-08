use crate::types::color::NUM_COLORS;
use std::time::Duration;

use super::uci_move::UCIMove;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SearchLimits {
    pub time: TimeLimit,
    pub depth: Option<u8>,
    pub mate: Option<u8>,
    pub nodes: Option<u64>,
    pub search_moves: Vec<UCIMove>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TimeLimit {
    #[default]
    Infinite,
    Fixed {
        move_time: Duration,
    },
    Dynamic {
        time_left: [Duration; NUM_COLORS],
        increment: [Duration; NUM_COLORS],
        moves_to_go: Option<u8>,
    },
    External,
}

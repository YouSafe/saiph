use std::time::Duration;
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TimeLimit {
    #[default]
    Infinite,
    Fixed {
        move_time: Duration,
    },
    Dynamic {
        time_left: PerColor<Duration>,
        increment: PerColor<Duration>,
        moves_to_go: Option<u8>,
    },
    External,
}

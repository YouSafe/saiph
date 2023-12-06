use chess_core::color::NUM_COLORS;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLimits {
    pub infinite: bool,
    pub time_left: [Duration; NUM_COLORS],
    pub increment: [Duration; NUM_COLORS],
    pub move_time: Duration,
    pub depth: u8,
    pub mate: u8,
}

impl SearchLimits {}

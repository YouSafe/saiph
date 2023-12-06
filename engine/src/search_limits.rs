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

impl SearchLimits {
    pub fn new_depth_limit(depth: u8) -> SearchLimits {
        SearchLimits {
            infinite: false,
            time_left: [Duration::default(); 2],
            increment: [Duration::default(); 2],
            move_time: Default::default(),
            depth,
            mate: 0,
        }
    }
}

use crate::types::color::Color;
use crate::types::search_limits::TimeLimit;
use web_time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub start: Instant,
    pub maximum: Option<Instant>,
    pub optimum: Option<Instant>,
}

impl Clock {
    pub fn new(start: Instant, limits: &TimeLimit, game_ply: u16, color: Color) -> Self {
        match limits {
            TimeLimit::Infinite | TimeLimit::External => Self {
                start,
                maximum: None,
                optimum: None,
            },
            TimeLimit::Fixed { move_time } => Self {
                start,
                maximum: Some(start + *move_time),
                optimum: Some(start + *move_time),
            },
            TimeLimit::Dynamic {
                time_left,
                increment,
                moves_to_go,
            } => {
                // Inspired by: https://github.com/official-stockfish/Stockfish/blob/65ece7d985291cc787d6c804a33f1dd82b75736d/src/timeman.cpp#L37

                const MOVE_OVERHEAD: u64 = 50;
                const MIN_TIME: u64 = 1;
                const MIN_MTG: u64 = 50;

                let mtg: u64 = moves_to_go.map_or(MIN_MTG, |v| (v as u64).min(MIN_MTG));

                let time_left = time_left[color].as_millis() as u64;
                let inc = increment[color].as_millis() as u64;

                let allocated_time = time_left / mtg + 3 * inc / 4;

                let optimum_time = allocated_time.saturating_sub(MOVE_OVERHEAD).max(MIN_TIME);

                let max_scale = (4.0 + game_ply as f64 / 12.0).min(7.0);

                let maximum_time = ((optimum_time as f64 * max_scale) as u64).clamp(
                    MIN_TIME,
                    time_left.saturating_sub(MOVE_OVERHEAD).max(MIN_TIME),
                );

                Self {
                    start,
                    maximum: Some(start + Duration::from_millis(maximum_time)),
                    optimum: Some(start + Duration::from_millis(optimum_time)),
                }
            }
        }

        // everything is in milliseconds
    }
}

#[cfg(test)]
mod test {
    use crate::clock::Clock;
    use crate::types::color::{Color, PerColor};
    use crate::types::search_limits::TimeLimit;
    use std::time::{Duration, Instant};

    #[test]
    fn test_first_move() {
        let clock = Clock::new(
            Instant::now(),
            &TimeLimit::Dynamic {
                time_left: PerColor::new([Duration::from_secs(120); 2]),
                increment: PerColor::new([Duration::from_secs(1); 2]),
                moves_to_go: None,
            },
            0,
            Color::White,
        );

        println!("{clock:#?}");
        println!(
            "Optimum duration: {:?}",
            (clock.optimum.unwrap() - clock.start)
        );
        println!(
            "Maximum duration: {:?}",
            (clock.maximum.unwrap() - clock.start)
        );
    }
}

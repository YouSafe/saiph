use crate::types::search_limits::TimeLimit;
use crate::types::{color::Color, search_limits::Millis};
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
            TimeLimit::Fixed {
                move_time: Millis(move_time),
            } => Self {
                start,
                maximum: Some(start + Duration::from_millis(*move_time)),
                optimum: Some(start + Duration::from_millis(*move_time)),
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

                let Millis(time_left) = time_left[color];
                let Millis(inc) = increment[color];

                let allocated_time = time_left / mtg + 3 * inc / 4;

                let optimum_time = (allocated_time - MOVE_OVERHEAD).max(MIN_TIME);

                let max_scale = (4.0 + game_ply as f64 / 12.0).min(7.0);

                let maximum_time = ((optimum_time as f64 * max_scale) as u64)
                    .clamp(MIN_TIME, (time_left - MOVE_OVERHEAD).max(MIN_TIME));

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
    use crate::types::search_limits::{Millis, TimeLimit};
    use std::time::Instant;

    impl Millis {
        pub const fn from_secs(secs: u64) -> Self {
            const MILLIS_PER_SEC: u64 = 1_000;
            Self(secs * MILLIS_PER_SEC)
        }
    }

    #[test]
    fn test_first_move() {
        let clock = Clock::new(
            Instant::now(),
            &TimeLimit::Dynamic {
                time_left: PerColor::new([Millis::from_secs(120); 2]),
                increment: PerColor::new([Millis::from_secs(1); 2]),
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

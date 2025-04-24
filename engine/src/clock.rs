use crate::types::search_limits::TimeLimit;
use instant::{Duration, Instant};
use types::color::Color;

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

                let move_overhead: u64 = 300;
                let moves_to_go_horizon: u64 = moves_to_go.map_or(50, |v| (v as u64).min(50));

                let time_left_mills = time_left[color].as_millis() as u64;
                let increment_mills = increment[color].as_millis() as u64;

                let time_left = (time_left_mills
                    + increment_mills
                        * (moves_to_go_horizon - 1)
                            .saturating_sub(move_overhead * (2 + moves_to_go_horizon)))
                .max(1);

                let opt_extra =
                    (1.0 + 12.0 * increment_mills as f64 / time_left_mills as f64).clamp(1.0, 1.12);

                let opt_scale = (0.0120 + (game_ply as f64 + 3.0).powf(0.45) * 0.0039)
                    .min(0.2 * time_left_mills as f64 / time_left as f64)
                    * opt_extra;

                let max_scale = (4.0 + game_ply as f64 / 12.0).min(7.0);

                let optimum_time = opt_scale * time_left as f64;
                let maximum_time = (max_scale * optimum_time)
                    .min(0.8 * time_left_mills as f64 - move_overhead as f64)
                    - 10.0;

                Self {
                    start,
                    maximum: Some(start + Duration::from_millis(maximum_time as u64)),
                    optimum: Some(start + Duration::from_millis(optimum_time as u64)),
                }
            }
        }

        // everything is in milliseconds
    }
}

#[cfg(test)]
mod test {
    use crate::clock::Clock;
    use crate::types::search_limits::TimeLimit;
    use std::time::{Duration, Instant};
    use types::color::{Color, PerColor};

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

        println!("{:#?}", clock);
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

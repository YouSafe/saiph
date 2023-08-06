use crate::search_limits::SearchLimits;
use chess_core::color::Color;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub start: Instant,
    pub maximum: Option<Instant>,
    pub optimum: Option<Instant>,
}

impl Clock {
    pub fn new(limits: &SearchLimits, game_ply: u16, color: Color) -> Self {
        // everything is in milliseconds

        // Inspired by: https://github.com/official-stockfish/Stockfish/blob/65ece7d985291cc787d6c804a33f1dd82b75736d/src/timeman.cpp#L37

        let move_overhead: u64 = 300;
        let moves_to_go_horizon: u64 = 50;

        let time_left_mills = limits.time_left[color as usize].as_millis() as u64;
        let increment_mills = limits.increment[color as usize].as_millis() as u64;

        let limited_by_time = !limits.move_time.is_zero();
        let limited_by_depth = limits.depth != 0;

        let limited_by_self = !limited_by_depth && !limited_by_time && !limits.infinite;

        let start = Instant::now();
        let optimum_end;
        let maximum_end;

        if limited_by_self {
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

            maximum_end = Some(start + Duration::from_millis(maximum_time as u64));
            optimum_end = Some(start + Duration::from_millis(optimum_time as u64));
        } else if limited_by_time {
            optimum_end = Some(start + limits.move_time);
            maximum_end = Some(start + limits.move_time);
        } else {
            optimum_end = None;
            maximum_end = None;
        }

        Clock {
            start,
            maximum: maximum_end,
            optimum: optimum_end,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::clock::Clock;
    use crate::search_limits::SearchLimits;
    use chess_core::color::Color;
    use std::time::Duration;

    #[test]
    fn test_first_move() {
        let clock = Clock::new(
            &SearchLimits {
                infinite: false,
                time_left: [Duration::from_secs(120), Duration::from_secs(120)],
                increment: [Duration::from_secs(1); 2],
                move_time: Default::default(),
                depth: 0,
                mate: 0,
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

    #[test]
    fn test_infinite_time() {
        let clock = Clock::new(
            &SearchLimits {
                infinite: true,
                time_left: [Duration::default(); 2],
                increment: [Duration::default(); 2],
                move_time: Default::default(),
                depth: 0,
                mate: 0,
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

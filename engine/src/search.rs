use crate::board::Board;
use crate::clock::Clock;
use crate::evaluation::{board_value, Evaluation};
use crate::move_ordering::mmv_lva;
use crate::pv_table::PrincipleVariationTable;
use crate::search_limits::SearchLimits;
use crate::transposition_table::{Entry, TranspositionTable, ValueType};
use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::Printer;
use instant::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SearchStatistics {
    nodes: u64,
}

pub struct Search<'a, P: Printer> {
    board: Board,
    limits: SearchLimits,
    pv_table: PrincipleVariationTable,
    table: &'a mut TranspositionTable,
    local_stop: bool,
    stop: &'a AtomicBool,
    printer: &'a P,
}

impl<'a, P: Printer> Search<'a, P> {
    pub fn new(
        board: Board,
        table: &'a mut TranspositionTable,
        stop: &'a AtomicBool,
        printer: &'a P,
        search_limits: SearchLimits,
    ) -> Self {
        Search {
            board,
            limits: search_limits,
            table,
            stop,
            printer,
            pv_table: PrincipleVariationTable::new(),
            local_stop: false,
        }
    }

    /// Fail soft variant of negamax search
    fn negamax_search<const PV: bool, const ROOT: bool>(
        &mut self,
        clock: &Clock,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        depth: u8,
        ply: u8,
        stats: &mut SearchStatistics,
    ) -> Evaluation {
        self.pv_table.clear(ply as usize);

        if self.should_interrupt(clock, stats.nodes) {
            return Evaluation::INVALID;
        }

        if !ROOT {
            if self.board.is_repetition() || self.board.is_draw_by_fifty_move_rule() {
                return Evaluation::EQUALITY;
            }

            alpha = alpha.max(Evaluation::mated_in(ply));
            beta = beta.min(Evaluation::mate_in(ply + 1));

            if alpha >= beta {
                return alpha;
            }
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, ply, stats);
        }

        stats.nodes += 1;

        let entry = self.table.probe(&self.board, ply);
        if let Some(entry) = &entry {
            if !PV && entry.depth >= depth && tt_cutoff(entry, alpha, beta) {
                return entry.value;
            }
        }

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            if !self.board.checkers().is_empty() {
                return Evaluation::mated_in(ply);
            } else {
                return Evaluation::EQUALITY;
            }
        }

        let original_alpha = alpha;
        let mut best_score = Evaluation::MIN;
        let mut best_move = None;

        moves.sort_by_key(|mov| {
            if let Some(entry) = &entry {
                if let Some(tt_move) = &entry.best_move {
                    if tt_move == mov {
                        return -200000;
                    }
                }
            }

            let src_piece = mov.piece;
            let dst_piece = self.board.piece_at(mov.destination());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece, dst_piece);
            }
            0
        });

        for chess_move in moves {
            self.board.apply_move(chess_move);
            let score =
                -self.negamax_search::<PV, false>(clock, -beta, -alpha, depth - 1, ply + 1, stats);
            self.board.undo_move();

            if self.local_stop {
                return Evaluation::INVALID;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(chess_move);

                if score > alpha {
                    alpha = score;

                    self.pv_table.update(ply as usize, chess_move);
                }
            }

            if alpha >= beta {
                break;
            }
        }

        let value_type = get_value_type(best_score, original_alpha, beta);

        self.table
            .store(&self.board, best_move, depth, best_score, value_type, ply);

        best_score
    }

    fn quiescence(
        &mut self,
        mut alpha: Evaluation,
        beta: Evaluation,
        ply: u8,
        stats: &mut SearchStatistics,
    ) -> Evaluation {
        stats.nodes += 1;

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            if !self.board.checkers().is_empty() {
                return Evaluation::mated_in(ply);
            } else {
                return Evaluation::EQUALITY;
            }
        }

        let evaluation = match self.board.side_to_move() {
            Color::White => board_value(&self.board),
            Color::Black => -board_value(&self.board),
        };

        alpha = alpha.max(evaluation);

        if alpha >= beta {
            return evaluation;
        }

        moves.retain(|m| m.is_capture());

        moves.sort_by_key(|mov| {
            let src_piece = mov.piece;
            let dst_piece = self.board.piece_at(mov.destination());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece, dst_piece);
            }
            0
        });

        let mut best_score = evaluation;
        for chess_move in moves {
            self.board.apply_move(chess_move);
            let score = -self.quiescence(-beta, -alpha, ply + 1, stats);
            self.board.undo_move();

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }

            if alpha >= beta {
                break;
            }
        }
        best_score
    }

    fn should_interrupt(&mut self, clock: &Clock, nodes: u64) -> bool {
        if let Some(max_nodes) = self.limits.nodes {
            if nodes >= max_nodes {
                self.local_stop = true;
                return true;
            }
        }

        if nodes & 4095 == 0 {
            return self.local_stop;
        }

        if self.stop.load(Ordering::Relaxed) {
            self.local_stop = true;
        } else if let Some(maximum) = clock.maximum {
            if maximum < Instant::now() {
                self.local_stop = true;
            }
        }

        self.local_stop
    }

    pub fn find_best_move(mut self) -> Option<Move> {
        let mut evaluation;
        let clock = Clock::new(
            &self.limits,
            self.board.game_ply(),
            self.board.side_to_move(),
        );

        let mut stats = SearchStatistics { nodes: 0 };

        for depth in 1..u8::MAX {
            evaluation = self.negamax_search::<true, true>(
                &clock,
                Evaluation::MIN,
                Evaluation::MAX,
                depth,
                0,
                &mut stats,
            );

            if self.local_stop {
                break;
            }

            let line = self.pv_table.variation();

            let output = format!(
                "info depth {} score {} time {} nodes {} pv {}",
                depth,
                if evaluation.is_mate() {
                    format!("mate {}", evaluation.mate_full_moves())
                } else {
                    format!("cp {}", evaluation)
                },
                clock.start.elapsed().as_millis(),
                stats.nodes,
                line.iter()
                    .filter_map(|mov| mov.map(|mov| mov.to_string()))
                    .collect::<Vec<_>>()
                    .join(" ")
            );

            self.printer.print(output.as_str());

            if depth >= self.limits.depth.unwrap_or(u8::MAX) {
                break;
            }

            if let Some(optimum) = clock.optimum {
                if optimum < Instant::now() {
                    break;
                }
            }
        }

        self.pv_table.best_move()
    }
}

fn get_value_type(score: Evaluation, alpha: Evaluation, beta: Evaluation) -> ValueType {
    if score <= alpha {
        ValueType::Upperbound
    } else if score >= beta {
        ValueType::Lowerbound
    } else {
        ValueType::Exact
    }
}

fn tt_cutoff(entry: &Entry, alpha: Evaluation, beta: Evaluation) -> bool {
    match entry.value_type {
        ValueType::Exact => true,
        ValueType::Lowerbound => entry.value >= beta,
        ValueType::Upperbound => entry.value <= alpha,
    }
}

// #[cfg(test)]
// mod test {
//     use crate::evaluation::Evaluation;
//     use crate::search::{ScoringMove, Search};
//     use crate::search_limits::{SearchLimits, TimeLimits};
//     use crate::searcher::StandardPrinter;
//     use crate::transposition_table::TranspositionTable;
//     use chess_core::board::{Board, BoardStatus};
//     use chess_core::chess_move::{Move, MoveFlag};
//     use chess_core::color::Color;
//     use chess_core::piece::Piece;
//     use chess_core::square::Square;
//     use std::str::FromStr;
//     use std::sync::atomic::AtomicBool;
//     use std::time::Duration;
//
//     struct TestingSetup {
//         table: TranspositionTable,
//         stop: AtomicBool,
//     }
//
//     impl TestingSetup {
//         fn new() -> Self {
//             let table = TranspositionTable::new();
//             let stop = AtomicBool::new(false);
//
//             Self { table, stop }
//         }
//
//         fn search(&mut self, fen: &str, limits: SearchLimits, expected_best_move: Move) {
//             let board = Board::from_str(&fen).unwrap();
//             let mut search = Search::new(board, &mut self.table, &self.stop, &StandardPrinter);
//             self.stop.store(false, std::sync::atomic::Ordering::SeqCst);
//
//             let best_move = search.find_best_move(limits).chess_move.unwrap();
//             assert_eq!(best_move, expected_best_move);
//         }
//     }
//
//     #[test]
//     fn take_white_queen() {
//         TestingSetup::new().search(
//             "8/1kQ5/8/8/8/8/8/7K b - - 0 1",
//             SearchLimits {
//                 time: TimeLimits::Fixed {
//                     move_time: Duration::from_secs(1),
//                 },
//                 depth: Some(2),
//                 ..Default::default()
//             },
//             Move {
//                 from: Square::B7,
//                 to: Square::C7,
//                 promotion: None,
//                 piece: Piece::King,
//                 flags: MoveFlag::Capture,
//             },
//         );
//     }
//
//     #[test]
//     fn mate_in_one() {
//         TestingSetup::new().search(
//             "8/8/8/8/8/6q1/7r/K6k b - - 6 4",
//             SearchLimits {
//                 mate: Some(1),
//                 ..Default::default()
//             },
//             Move {
//                 from: Square::G3,
//                 to: Square::E1,
//                 promotion: None,
//                 piece: Piece::Queen,
//                 flags: MoveFlag::Normal,
//             },
//         );
//     }
//
//     //     #[test]
//     //     fn back_rank_mate_white() {
//     //         let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         assert_eq!(
//     //             best_move.chess_move,
//     //             Some(Move {
//     //                 from: Square::D8,
//     //                 to: Square::D1,
//     //                 promotion: None,
//     //                 piece: Piece::Rook,
//     //                 flags: MoveFlag::Normal,
//     //             })
//     //         );
//     //         println!("{}", best_move.evaluation);
//     //     }
//
//     //     #[test]
//     //     fn back_rank_mate_black() {
//     //         let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         assert_eq!(
//     //             best_move.chess_move,
//     //             Some(Move {
//     //                 from: Square::D1,
//     //                 to: Square::D8,
//     //                 promotion: None,
//     //                 piece: Piece::Rook,
//     //                 flags: MoveFlag::Normal,
//     //             })
//     //         );
//     //         println!("{}", best_move.evaluation);
//     //     }
//
//     //     #[test]
//     //     fn mate_in_one_queen_rook() {
//     //         let board = Board::from_str("1Q3q1k/p5pp/8/2p2P2/P1B2P1P/6K1/R7/8 w - - 4 41").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         assert_eq!(
//     //             best_move.chess_move,
//     //             Some(Move {
//     //                 from: Square::B8,
//     //                 to: Square::F8,
//     //                 promotion: None,
//     //                 piece: Piece::Queen,
//     //                 flags: MoveFlag::Capture,
//     //             })
//     //         )
//     //     }
//
//     //     #[test]
//     //     fn mate_in_one_corner() {
//     //         let board = Board::from_str("5rk1/7p/3R2p1/3p4/7P/5pP1/5P1K/5q2 b - - 4 39").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         assert_eq!(
//     //             best_move.chess_move,
//     //             Some(Move {
//     //                 from: Square::F1,
//     //                 to: Square::G2,
//     //                 promotion: None,
//     //                 piece: Piece::Queen,
//     //                 flags: MoveFlag::Normal,
//     //             })
//     //         );
//     //     }
//
//     //     #[test]
//     //     fn mate_in_one_two_queens() {
//     //         let mut board = Board::from_str("8/1K6/6k1/r7/8/2q5/8/3q4 b - - 1 56").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         board.apply_move(best_move.chess_move.unwrap());
//     //         assert_eq!(board.status(), BoardStatus::Checkmate);
//     //     }
//
//     //     #[test]
//     //     fn mate_in_two_ply() {
//     //         let mut board = Board::from_str("1r6/8/8/8/8/8/2k5/K7 w - - 0 1").unwrap();
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let whites_move = search.find_best_move(SearchLimits::new_depth_limit(3));
//     //         assert_eq!(
//     //             whites_move,
//     //             ScoringMove {
//     //                 evaluation: Evaluation::new_mate_eval(Color::Black, 2),
//     //                 chess_move: Some(Move {
//     //                     from: Square::A1,
//     //                     to: Square::A2,
//     //                     promotion: None,
//     //                     piece: Piece::King,
//     //                     flags: MoveFlag::Normal,
//     //                 })
//     //             }
//     //         );
//     //         board.apply_move(whites_move.chess_move.unwrap());
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//     //         let black_move = search.find_best_move(SearchLimits::new_depth_limit(3));
//     //         assert_eq!(
//     //             black_move,
//     //             ScoringMove {
//     //                 evaluation: Evaluation::new_mate_eval(Color::White, 1),
//     //                 chess_move: Some(Move {
//     //                     from: Square::B8,
//     //                     to: Square::A8,
//     //                     promotion: None,
//     //                     piece: Piece::Rook,
//     //                     flags: MoveFlag::Normal,
//     //                 })
//     //             }
//     //         );
//     //         board.apply_move(black_move.chess_move.unwrap());
//     //         assert_eq!(board.status(), BoardStatus::Checkmate);
//     //     }
//
//     //     #[test]
//     //     fn test_deep_mate() {
//     //         // let mut board = Board::from_str("8/8/p7/K7/8/8/2k5/1R6 w - - 10 67").unwrap();
//     //         let board = Board::from_str("6r1/5K2/8/8/7k/7P/8/8 b - - 10 67").unwrap();
//     //         // let board = Board::from_str("8/8/5K2/8/8/4r2k/8/8 w - - 0 71").unwrap();
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(40));
//     //         eprintln!("eval: {:?}", best_move.evaluation);
//     //         assert!(best_move.evaluation.is_mate());
//     //     }
//
//     //     #[test]
//     //     fn test_rook_vs_king() {
//     //         let mut board = Board::from_str("8/6K1/8/8/8/r6k/8/8 w - - 6 74").unwrap();
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(22));
//     //         assert!(best_move.evaluation.is_mate());
//     //         eprintln!("eval: {:?}", best_move.evaluation);
//
//     //         let line = table.pv_line(&mut board, 21);
//
//     //         for mov in line {
//     //             board.apply_move(mov);
//     //         }
//
//     //         println!("{}", board);
//     //     }
//
//     //     #[test]
//     //     fn test_pawn_vs_king() {
//     //         let board = Board::from_str("8/8/8/1k6/8/1K6/1P6/8 b - - 0 1").unwrap();
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(44));
//     //         assert!(best_move.evaluation.is_mate());
//     //         eprintln!("eval: {:?}", best_move.evaluation);
//     //     }
//
//     //     #[test]
//     //     fn test_mate_in_three() {
//     //         let mut board = Board::from_str("8/8/3k4/7R/6Q1/8/8/7K w - - 0 1").unwrap();
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search;
//
//     //         for _ in 0..5 {
//     //             search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//     //             let best_move = search.find_best_move(SearchLimits::new_depth_limit(7));
//     //             let chess_move = best_move.chess_move;
//     //             board.apply_move(chess_move.unwrap());
//     //         }
//     //         assert_eq!(board.status(), BoardStatus::Checkmate);
//     //     }
//
//     //     #[test]
//     //     fn test_mate_in_two() {
//     //         let mut board = Board::from_str("8/3k4/7R/6Q1/8/8/8/7K w - - 0 1").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//
//     //         let mut search;
//
//     //         for _ in 0..3 {
//     //             search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//     //             let best_move = search.find_best_move(SearchLimits::new_depth_limit(4));
//     //             let chess_move = best_move.chess_move;
//     //             board.apply_move(chess_move.unwrap());
//     //         }
//     //         assert_eq!(board.status(), BoardStatus::Checkmate);
//     //     }
//
//     //     #[test]
//     //     fn test_mate_in_seven() {
//     //         let mut board = Board::from_str("7k/8/1K2PPPP/8/B7/8/4pppp/8 w - - 0 1").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//
//     //         let mut search;
//
//     //         for _ in 0..13 {
//     //             search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//     //             let best_move = search.find_best_move(SearchLimits::new_depth_limit(14));
//     //             let chess_move = best_move.chess_move;
//     //             board.apply_move(chess_move.unwrap());
//     //         }
//
//     //         assert_eq!(board.status(), BoardStatus::Checkmate);
//     //     }
//
//     //     #[test]
//     //     fn test_tricky_mate_in_one() {
//     //         // Position from: https://www.stmintz.com/ccc/index.php?id=123825
//     //         let board =
//     //             Board::from_str("8/8/pppppppK/NBBR1NRp/nbbrqnrP/PPPPPPPk/8/Q7 w - - 0 1").unwrap();
//
//     //         let mut table = TranspositionTable::new();
//     //         let stop = AtomicBool::new(false);
//     //         let mut search = Search::new(board.clone(), &mut table, &stop, &StandardPrinter);
//
//     //         let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
//     //         assert_eq!(
//     //             best_move.chess_move,
//     //             Some(Move {
//     //                 from: Square::A1,
//     //                 to: Square::H1,
//     //                 promotion: None,
//     //                 piece: Piece::Queen,
//     //                 flags: MoveFlag::Normal,
//     //             })
//     //         );
//     //     }
// }

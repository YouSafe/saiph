use crate::clock::Clock;
use crate::evaluation::{board_value, Evaluation};
use crate::move_ordering::mmv_lva;
use crate::search_limits::SearchLimits;
use crate::transposition_table::{TranspositionTable, ValueType};
use chess_core::bitboard::BitBoard;
use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::color::Color;
use instant::Instant;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SearchStatistics {
    nodes: u64,
}

pub struct Search<'a> {
    board: Board,
    stop: &'a AtomicBool,
    table: &'a mut TranspositionTable,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScoringMove {
    pub evaluation: Evaluation,
    pub chess_move: Option<Move>,
}

impl<'a> Search<'a> {
    pub fn new(board: Board, table: &'a mut TranspositionTable, stop: &'a AtomicBool) -> Self {
        Search { board, table, stop }
    }

    /// Fail-hard variation of negamax search
    fn negamax_search(
        &mut self,
        clock: &Clock,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        depth: u8,
        ply: u8,
        stats: &mut SearchStatistics,
    ) -> Evaluation {
        assert!(alpha < beta);
        let old_alpha = alpha;

        if let Some(optimum) = clock.optimum {
            if optimum < Instant::now() && (stats.nodes & 4095) == 0 {
                self.stop.store(true, Ordering::SeqCst);
                return Evaluation(0);
            }
        }

        if ply > 0 {
            if self.board.is_repetition() {
                return Evaluation(0);
            }

            alpha = alpha.max(Evaluation::mated_in(ply));
            beta = beta.min(Evaluation::mate_in(ply + 1));
            if alpha >= beta {
                return alpha;
            }
        }

        if ply > 0 {
            if let Some(entry) = self.table.probe(&self.board) {
                if entry.depth >= depth {
                    let corrected_value = if entry.value.is_mate() {
                        entry.value.tt_to_score(ply)
                    } else {
                        entry.value
                    };

                    match entry.value_type {
                        ValueType::Exact => return corrected_value,
                        ValueType::Alpha => {
                            beta = beta.min(corrected_value);
                        }
                        ValueType::Beta => {
                            alpha = alpha.max(corrected_value);
                        }
                    }

                    if alpha >= beta {
                        return corrected_value;
                    }
                }
            }
        }

        if depth == 0 {
            return self.quiescence(alpha, beta, ply, stats);
        }

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            return if self.board.checkers() != BitBoard::EMPTY {
                Evaluation::mated_in(ply)
            } else {
                Evaluation(0)
            };
        }

        let pv_move = self.table.probe_pv(&self.board);

        // move ordering
        moves.sort_by_key(|mov| {
            let src_piece = mov.piece;
            let dst_piece = self.board.piece_on_square(mov.destination());

            if let Some(pv_move) = pv_move {
                if mov == &pv_move {
                    return -20000;
                }
            }

            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece, dst_piece);
            }
            0
        });

        let mut best_score = Evaluation::MIN;
        let mut best_move = None;

        for chess_move in moves {
            self.board.apply_move(chess_move);
            stats.nodes += 1;

            let score = -self.negamax_search(clock, -beta, -alpha, depth - 1, ply + 1, stats);
            self.board.undo_move();

            if self.stop.load(Ordering::SeqCst) {
                return Evaluation(0);
            }

            if score > best_score {
                best_score = score;

                if score > alpha {
                    best_move = Some(chess_move);

                    // update pv later

                    if score >= beta {
                        break;
                    } else {
                        alpha = score;
                    }
                }
            }
        }

        let value_type = if best_score >= beta {
            ValueType::Beta
        } else if best_score > old_alpha {
            ValueType::Exact
        } else {
            ValueType::Alpha
        };

        self.table
            .store(&self.board, best_move, depth, best_score, value_type, ply);

        // node fails low
        best_score
    }

    fn quiescence(
        &mut self,
        mut alpha: Evaluation,
        beta: Evaluation,
        ply: u8,
        stats: &mut SearchStatistics,
    ) -> Evaluation {
        let evaluation = match self.board.side_to_move() {
            Color::White => board_value(&self.board),
            Color::Black => -board_value(&self.board),
        };

        let mut moves = self.board.generate_moves();
        if moves.is_empty() {
            return if self.board.checkers() != BitBoard::EMPTY {
                Evaluation::mated_in(ply)
            } else {
                Evaluation(0)
            };
        }

        if evaluation >= beta {
            return evaluation;
        }

        if evaluation > alpha {
            alpha = evaluation;
        }

        moves.retain(|m| m.is_capture());
        moves.sort_by_key(|mov| {
            let src_piece = mov.piece;
            let dst_piece = self.board.piece_on_square(mov.destination());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece, dst_piece);
            }
            0
        });

        let mut best_value = evaluation;
        for chess_move in moves {
            self.board.apply_move(chess_move);
            stats.nodes += 1;
            let score = -self.quiescence(-beta, -alpha, ply + 1, stats);
            self.board.undo_move();

            if score > best_value {
                best_value = score;

                if score > alpha {
                    if score >= beta {
                        break;
                    } else {
                        alpha = score;
                    }
                }
            }
        }

        best_value
    }

    pub fn find_best_move(&mut self, limits: SearchLimits) -> ScoringMove {
        self.table.age();

        let mut evaluation = Evaluation(0);
        let mut line = vec![];
        let mut stats = SearchStatistics { nodes: 0 };

        let clock = Clock::new(&limits, self.board.game_ply(), self.board.side_to_move());

        for max_depth in 1.. {
            if limits.depth != 0 && max_depth > limits.depth {
                break;
            }

            evaluation = self.negamax_search(
                &clock,
                Evaluation::MIN,
                Evaluation::MAX,
                max_depth,
                0,
                &mut stats,
            );
            //
            // // the eval negamax returns is from the point of view of the current side to play
            // // however, for me it's so much more intuitive when the eval is from the point of view
            // // of the white player
            // if self.board.side_to_move() == Color::Black {
            //     evaluation = -evaluation;
            // }

            if self.stop.load(Ordering::SeqCst) {
                eprintln!("stop search");
                break;
            }

            line = self.table.pv_line(&mut self.board, max_depth);

            print!("info depth {} ", max_depth);
            if evaluation.is_mate() {
                print!("score mate {} ", evaluation.mate_full_moves());
            } else {
                print!("score cp {} ", evaluation);
            }
            print!("time {} ", clock.start.elapsed().as_millis());
            print!("nodes {} pv ", stats.nodes);
            for mov in line.iter() {
                print!("{} ", mov);
            }
            println!();

            // This leads to unwanted threefold repetitions
            // if evaluation.is_mate() && evaluation.mate_num_ply() >= limits.mate as i8 {
            //     break;
            // }
        }

        ScoringMove {
            evaluation,
            chess_move: line.get(0).cloned(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::evaluation::Evaluation;
    use crate::search::{ScoringMove, Search};
    use crate::search_limits::SearchLimits;
    use crate::transposition_table::TranspositionTable;
    use chess_core::board::{Board, BoardStatus};
    use chess_core::chess_move::{Move, MoveFlag};
    use chess_core::color::Color;
    use chess_core::piece::Piece;
    use chess_core::square::Square;
    use std::str::FromStr;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    #[test]
    fn take_white_queen() {
        let board = Board::from_str("8/1kQ5/8/8/8/8/8/7K b - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);

        let mut search = Search::new(board, &mut table, &stop);
        let limits = SearchLimits {
            infinite: false,
            time_left: [Duration::from_secs(1); 2],
            increment: [Duration::from_millis(0); 2],
            move_time: Default::default(),
            depth: 0,
            mate: 0,
        };

        let best_move = search.find_best_move(limits);
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::B7,
                to: Square::C7,
                promotion: None,
                piece: Piece::King,
                flags: MoveFlag::Capture,
            })
        )
    }

    #[test]
    fn mate_in_one() {
        let board = Board::from_str("8/8/8/8/8/6q1/7r/K6k b - - 6 4").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);

        let mut search = Search::new(board, &mut table, &stop);
        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::G3,
                to: Square::E1,
                promotion: None,
                piece: Piece::Queen,
                flags: MoveFlag::Normal,
            })
        );

        println!("{}", best_move.evaluation);
    }

    #[test]
    fn back_rank_mate_white() {
        let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);

        let mut search = Search::new(board, &mut table, &stop);
        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::D8,
                to: Square::D1,
                promotion: None,
                piece: Piece::Rook,
                flags: MoveFlag::Normal,
            })
        );
        println!("{}", best_move.evaluation);
    }

    #[test]
    fn back_rank_mate_black() {
        let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board, &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::D1,
                to: Square::D8,
                promotion: None,
                piece: Piece::Rook,
                flags: MoveFlag::Normal,
            })
        );
        println!("{}", best_move.evaluation);
    }

    #[test]
    fn mate_in_one_queen_rook() {
        let board = Board::from_str("1Q3q1k/p5pp/8/2p2P2/P1B2P1P/6K1/R7/8 w - - 4 41").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board, &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::B8,
                to: Square::F8,
                promotion: None,
                piece: Piece::Queen,
                flags: MoveFlag::Capture,
            })
        )
    }

    #[test]
    fn mate_in_one_corner() {
        let board = Board::from_str("5rk1/7p/3R2p1/3p4/7P/5pP1/5P1K/5q2 b - - 4 39").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board, &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        assert_eq!(
            best_move.chess_move,
            Some(Move {
                from: Square::F1,
                to: Square::G2,
                promotion: None,
                piece: Piece::Queen,
                flags: MoveFlag::Normal,
            })
        );
    }

    #[test]
    fn mate_in_one_two_queens() {
        let mut board = Board::from_str("8/1K6/6k1/r7/8/2q5/8/3q4 b - - 1 56").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board.clone(), &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(2));
        board.apply_move(best_move.chess_move.unwrap());
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn mate_in_two_ply() {
        let mut board = Board::from_str("1r6/8/8/8/8/8/2k5/K7 w - - 0 1").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board.clone(), &mut table, &stop);

        let whites_move = search.find_best_move(SearchLimits::new_depth_limit(3));
        assert_eq!(
            whites_move,
            ScoringMove {
                evaluation: Evaluation::new_mate_eval(Color::Black, 2),
                chess_move: Some(Move {
                    from: Square::A1,
                    to: Square::A2,
                    promotion: None,
                    piece: Piece::King,
                    flags: MoveFlag::Normal,
                })
            }
        );
        board.apply_move(whites_move.chess_move.unwrap());
        let mut search = Search::new(board.clone(), &mut table, &stop);
        let black_move = search.find_best_move(SearchLimits::new_depth_limit(3));
        assert_eq!(
            black_move,
            ScoringMove {
                evaluation: Evaluation::new_mate_eval(Color::White, 1),
                chess_move: Some(Move {
                    from: Square::B8,
                    to: Square::A8,
                    promotion: None,
                    piece: Piece::Rook,
                    flags: MoveFlag::Normal,
                })
            }
        );
        board.apply_move(black_move.chess_move.unwrap());
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_deep_mate() {
        // let mut board = Board::from_str("8/8/p7/K7/8/8/2k5/1R6 w - - 10 67").unwrap();
        let board = Board::from_str("6r1/5K2/8/8/7k/7P/8/8 b - - 10 67").unwrap();
        // let board = Board::from_str("8/8/5K2/8/8/4r2k/8/8 w - - 0 71").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board, &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(40));
        eprintln!("eval: {:?}", best_move.evaluation);
        assert!(best_move.evaluation.is_mate());
    }

    #[test]
    fn test_rook_vs_king() {
        let mut board = Board::from_str("8/6K1/8/8/8/r6k/8/8 w - - 6 74").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board.clone(), &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(22));
        assert!(best_move.evaluation.is_mate());
        eprintln!("eval: {:?}", best_move.evaluation);

        let line = table.pv_line(&mut board, 21);

        for mov in line {
            board.apply_move(mov);
        }

        println!("{}", board);
    }

    #[test]
    fn test_pawn_vs_king() {
        let board = Board::from_str("8/8/8/1k6/8/1K6/1P6/8 b - - 0 1").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(board, &mut table, &stop);

        let best_move = search.find_best_move(SearchLimits::new_depth_limit(44));
        assert!(best_move.evaluation.is_mate());
        eprintln!("eval: {:?}", best_move.evaluation);
    }

    #[test]
    fn test_mate_in_three() {
        let mut board = Board::from_str("8/8/3k4/7R/6Q1/8/8/7K w - - 0 1").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search;

        for _ in 0..5 {
            search = Search::new(board.clone(), &mut table, &stop);
            let best_move = search.find_best_move(SearchLimits::new_depth_limit(7));
            let chess_move = best_move.chess_move;
            board.apply_move(chess_move.unwrap());
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_two() {
        let mut board = Board::from_str("8/3k4/7R/6Q1/8/8/8/7K w - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);

        let mut search;

        for _ in 0..3 {
            search = Search::new(board.clone(), &mut table, &stop);
            let best_move = search.find_best_move(SearchLimits::new_depth_limit(4));
            let chess_move = best_move.chess_move;
            board.apply_move(chess_move.unwrap());
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_seven() {
        let mut board = Board::from_str("7k/8/1K2PPPP/8/B7/8/4pppp/8 w - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);

        let mut search;

        for _ in 0..13 {
            search = Search::new(board.clone(), &mut table, &stop);
            let best_move = search.find_best_move(SearchLimits::new_depth_limit(14));
            let chess_move = best_move.chess_move;
            board.apply_move(chess_move.unwrap());
        }

        assert_eq!(board.status(), BoardStatus::Checkmate);
    }
}

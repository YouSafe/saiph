use crate::evaluation::{board_value, Evaluation};
use crate::move_ordering::mmv_lva;
use crate::timer::Timer;
use crate::transposition_table::{TranspositionTable, ValueType};
use chess_core::bitboard::BitBoard;
use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::color::Color;
use chess_core::movgen::generate_moves;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SearchStatistics {
    nodes: u64,
}

pub struct Search<'a> {
    stop: &'a AtomicBool,
    table: &'a mut TranspositionTable,
    visited_positions: HashSet<Board>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScoringMove {
    pub evaluation: Evaluation,
    pub chess_move: Option<Move>,
}

impl<'a> Search<'a> {
    pub fn new(table: &'a mut TranspositionTable, stop: &'a AtomicBool) -> Self {
        Search {
            table,
            stop,
            visited_positions: HashSet::with_capacity(120),
        }
    }

    fn is_repetition(&self, board: &Board) -> bool {
        self.visited_positions.contains(board)
    }

    /// Fail-hard variation of negamax search
    fn negamax_search(
        &mut self,
        board: &Board,
        timer: &Timer,
        mut alpha: Evaluation,
        beta: Evaluation,
        depth: u8,
        ply: u8,
        stats: &mut SearchStatistics,
    ) -> Evaluation {
        let mut value_type = ValueType::Alpha;

        if depth == 0 {
            return self.quiescence(board, alpha, beta);
        }

        if timer.is_already_up() {
            self.stop.store(true, Ordering::Relaxed);
        }

        stats.nodes += 1;

        if self.is_repetition(board) {
            return Evaluation(0);
        }

        if let Some(scoring_move) = self.table.probe(board, alpha, beta, depth, ply) {
            return scoring_move.evaluation;
        }

        let mut moves = generate_moves(board);
        if moves.is_empty() {
            return if board.checkers() != BitBoard::EMPTY {
                match board.side_to_move() {
                    Color::White => Evaluation::new_mate_eval(!board.side_to_move(), ply),
                    Color::Black => -Evaluation::new_mate_eval(!board.side_to_move(), ply),
                }
            } else {
                Evaluation(0)
            };
        }

        let pv_move = self.table.probe_pv(board);

        // move ordering
        moves.sort_by_key(|mov| {
            let src_piece = mov.piece;
            let dst_piece = board.piece_on_square(mov.destination());

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

        let mut local_best = ScoringMove {
            evaluation: Evaluation::MIN,
            chess_move: None,
        };

        for chess_move in moves {
            let result = board.make_move(chess_move);

            let score =
                -self.negamax_search(&result, timer, -beta, -alpha, depth - 1, ply + 1, stats);

            if self.stop.load(Ordering::Relaxed) {
                return Evaluation(0);
            }

            if score > local_best.evaluation {
                local_best = ScoringMove {
                    evaluation: score,
                    chess_move: Some(chess_move),
                };
                if score > alpha {
                    if score >= beta {
                        self.table.store(
                            board,
                            local_best.chess_move,
                            depth,
                            beta,
                            ValueType::Beta,
                            ply,
                        );

                        // node fails high
                        return beta;
                    }

                    // PV node
                    alpha = score;
                    value_type = ValueType::Exact;
                }
            }
        }

        self.table
            .store(board, local_best.chess_move, depth, alpha, value_type, ply);

        // node fails low
        alpha
    }

    fn quiescence(&mut self, board: &Board, mut alpha: Evaluation, beta: Evaluation) -> Evaluation {
        let evaluation = match board.side_to_move() {
            Color::White => board_value(board),
            Color::Black => -board_value(board),
        };

        if evaluation > alpha {
            alpha = evaluation;

            if evaluation >= beta {
                return beta;
            }
        }

        let mut moves = generate_moves(board);
        moves.retain(|m| m.is_capture());
        moves.sort_by_key(|mov| {
            let src_piece = mov.piece;
            let dst_piece = board.piece_on_square(mov.destination());
            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece, dst_piece);
            }
            0
        });

        for chess_move in moves {
            let result = board.make_move(chess_move);

            let score = self.quiescence(&result, alpha, beta);

            if score > alpha {
                // PV node
                alpha = score;

                if score >= beta {
                    // node fails high
                    return beta;
                }
            }
        }

        // node fails low
        alpha
    }

    pub fn find_best_move(&mut self, board: &Board, timer: &Timer) -> ScoringMove {
        self.table.age();

        let mut evaluation = Evaluation(0);
        let mut line = vec![];
        for max_depth in 1..40 {
            let mut stats = SearchStatistics { nodes: 0 };

            evaluation = self.negamax_search(
                board,
                timer,
                Evaluation::MIN,
                Evaluation::MAX,
                max_depth,
                0,
                &mut stats,
            );

            // the eval negamax returns is from the point of view of the current side to play
            // however, for me it's so much more intuitive when the eval is from the point of view
            // of the white player
            if board.side_to_move() == Color::Black {
                evaluation = -evaluation;
            }

            if self.stop.load(Ordering::Relaxed) {
                eprintln!("stop search");
                break;
            }

            line = self.table.pv_line(board, max_depth);

            print!(
                "depth: {} score {} nodes {} ",
                max_depth, evaluation, stats.nodes
            );
            for mov in line.iter() {
                print!("{} ", mov);
            }
            println!();

            if evaluation.is_mate() {
                break;
            }
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
    use crate::timer::Timer;
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

        let mut search = Search::new(&mut table, &stop);
        let mut timer = Timer::new();
        timer.set_timer(Duration::from_secs(1));
        let best_move = search.find_best_move(&board, &timer);
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

        let mut search = Search::new(&mut table, &stop);
        let best_move = search.find_best_move(&board, &Timer::new());
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

        let mut search = Search::new(&mut table, &stop);
        let best_move = search.find_best_move(&board, &Timer::new());
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
        let mut search = Search::new(&mut table, &stop);

        let best_move = search.find_best_move(&board, &Timer::new());
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
        let mut search = Search::new(&mut table, &stop);

        let best_move = search.find_best_move(&board, &Timer::new());
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
        let mut search = Search::new(&mut table, &stop);

        let best_move = search.find_best_move(&board, &Timer::new());
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
        let board = Board::from_str("8/1K6/6k1/r7/8/2q5/8/3q4 b - - 1 56").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        let best_move = search.find_best_move(&board, &Timer::new());
        let result = board.make_move(best_move.chess_move.unwrap());
        assert_eq!(result.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn mate_in_two_ply() {
        let board = Board::from_str("1r6/8/8/8/8/8/2k5/K7 w - - 0 1").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        let whites_move = search.find_best_move(&board, &Timer::new());
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
        let result = board.make_move(whites_move.chess_move.unwrap());
        let black_move = search.find_best_move(&result, &Timer::new());
        assert_eq!(
            black_move,
            ScoringMove {
                evaluation: Evaluation::new_mate_eval(Color::Black, 1),
                chess_move: Some(Move {
                    from: Square::B8,
                    to: Square::A8,
                    promotion: None,
                    piece: Piece::Rook,
                    flags: MoveFlag::Normal,
                })
            }
        );
        let result = result.make_move(black_move.chess_move.unwrap());
        assert_eq!(result.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_deep_mate() {
        // let mut board = Board::from_str("8/8/p7/K7/8/8/2k5/1R6 w - - 10 67").unwrap();
        let board = Board::from_str("6r1/5K2/8/8/7k/7P/8/8 b - - 10 67").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        let best_move = search.find_best_move(&board, &Timer::new());
        eprintln!("eval: {:?}", best_move.evaluation);
        assert_eq!(best_move.evaluation.mate_num_ply(), -16 * 2 + 1)
    }

    #[test]
    fn test_mate_in_three() {
        let mut board = Board::from_str("8/8/3k4/7R/6Q1/8/8/7K w - - 0 1").unwrap();
        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        for _ in 0..5 {
            let best_move = search.find_best_move(&board, &Timer::new());
            let chess_move = best_move.chess_move;
            board = board.make_move(chess_move.unwrap());
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_two() {
        let mut board = Board::from_str("8/3k4/7R/6Q1/8/8/8/7K w - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        for _ in 0..3 {
            let best_move = search.find_best_move(&board, &Timer::new());
            let chess_move = best_move.chess_move;
            board = board.make_move(chess_move.unwrap());
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_seven() {
        let mut board = Board::from_str("7k/8/1K2PPPP/8/B7/8/4pppp/8 w - - 0 1").unwrap();

        let mut table = TranspositionTable::new();
        let stop = AtomicBool::new(false);
        let mut search = Search::new(&mut table, &stop);

        let timer = Timer::new();

        for _ in 0..13 {
            let best_move = search.find_best_move(&board, &timer);
            let chess_move = best_move.chess_move;
            board = board.make_move(chess_move.unwrap());
        }

        assert_eq!(board.status(), BoardStatus::Checkmate);
    }
}

use crate::evaluation::{board_value, Evaluation};
use crate::move_ordering::mmv_lva;
use crate::transposition_table::{TranspositionTable, ValueType};
use chess_core::bitboard::BitBoard;
use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::color::Color;
use chess_core::movgen::generate_moves;
use std::fmt;

pub struct Search {
    transposition_table: TranspositionTable,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ScoringMove {
    pub evaluation: Evaluation,
    pub chess_move: Option<Move>,
}

impl fmt::Display for ScoringMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.chess_move {
            None => {
                write!(f, "({:?}, None)", self.evaluation)
            }
            Some(chess_move) => {
                write!(f, "({:?}, {})", self.evaluation, chess_move)
            }
        }
    }
}

impl Search {
    pub fn new() -> Self {
        Search {
            transposition_table: TranspositionTable::new(),
        }
    }

    fn minimax_search(
        &mut self,
        board: &Board,
        is_maximizing: bool,
        ply: u8,
        depth: u8,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        visited_nodes: &mut i64,
    ) -> ScoringMove {
        // try reading entry from transposition table
        let entry = self
            .transposition_table
            .read_entry(board, depth, ply, alpha, beta);

        if let Some(entry) = entry {
            return entry;
        }

        let mut moves = generate_moves(&board);
        if moves.is_empty() {
            return ScoringMove {
                evaluation: match board.checkers() != BitBoard::EMPTY {
                    true => Evaluation::new_mate_eval(!board.side_to_move(), ply),
                    false => Evaluation(0),
                },
                chess_move: None,
            };
        }

        if depth == 0 {
            let value =
                self.quiescence_search(board, ply, is_maximizing, alpha, beta, visited_nodes);
            // let value = board_value(board);
            *visited_nodes += 1;

            return ScoringMove {
                evaluation: value,
                chess_move: None,
            };
        }

        moves.sort_unstable_by_key(|mov| {
            let src_piece = board.piece_on_square(mov.source());
            let dst_piece = board.piece_on_square(mov.destination());

            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece.unwrap(), dst_piece);
            }

            0
        });

        let mut best_scoring_move = ScoringMove {
            evaluation: match is_maximizing {
                true => Evaluation::MIN,
                false => Evaluation::MAX,
            },
            chess_move: None,
        };

        let mut value_type = None;

        for chess_move in moves {
            let result = board.make_move(chess_move);

            let scoring_move = self.minimax_search(
                &result,
                !is_maximizing,
                ply + 1,
                depth - 1,
                alpha,
                beta,
                visited_nodes,
            );

            if is_maximizing {
                if scoring_move.evaluation > best_scoring_move.evaluation {
                    best_scoring_move.evaluation = scoring_move.evaluation;
                    best_scoring_move.chess_move = Some(chess_move);
                    alpha = alpha.max(best_scoring_move.evaluation);

                    value_type = Some(ValueType::Exact);
                }

                if alpha >= beta {
                    best_scoring_move.evaluation = beta;
                    best_scoring_move.chess_move = Some(chess_move);
                    value_type = Some(ValueType::Alpha);
                    break;
                }
            } else {
                if scoring_move.evaluation < best_scoring_move.evaluation {
                    best_scoring_move.evaluation = scoring_move.evaluation;
                    best_scoring_move.chess_move = Some(chess_move);
                    beta = beta.min(best_scoring_move.evaluation);

                    value_type = Some(ValueType::Exact);
                }

                if alpha >= beta {
                    best_scoring_move.evaluation = alpha;
                    best_scoring_move.chess_move = Some(chess_move);
                    value_type = Some(ValueType::Beta);
                    break;
                }
            }
        }

        assert!(best_scoring_move.chess_move.is_some());

        self.transposition_table.add_entry(
            *board,
            best_scoring_move.evaluation,
            value_type.unwrap(),
            best_scoring_move.chess_move,
            depth,
            ply,
        );

        best_scoring_move
    }

    pub fn quiescence_search(
        &mut self,
        board: &Board,
        ply_from_root: u8,
        is_maximizing: bool,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        visited_nodes: &mut i64,
    ) -> Evaluation {
        let mut eval = board_value(board);

        if is_maximizing && eval >= beta {
            return beta;
        } else if !is_maximizing && eval <= alpha {
            return alpha;
        }

        if is_maximizing && alpha < eval {
            alpha = eval;
        } else if !is_maximizing && beta > eval {
            beta = eval;
        }

        let mut capture_moves = generate_moves(board);
        capture_moves.retain(|m| m.is_capture());

        capture_moves.sort_unstable_by_key(|mov| {
            let src_piece = board.piece_on_square(mov.source());
            let dst_piece = board.piece_on_square(mov.destination());

            if let Some(dst_piece) = dst_piece {
                return -mmv_lva(src_piece.unwrap(), dst_piece);
            }
            0
        });

        for chess_move in capture_moves {
            let result = board.make_move(chess_move);

            eval = self.quiescence_search(
                &result,
                ply_from_root + 1,
                !is_maximizing,
                alpha,
                beta,
                visited_nodes,
            );

            if is_maximizing {
                alpha = alpha.max(eval);
            } else if !is_maximizing {
                beta = beta.min(eval);
            }

            if alpha >= beta {
                break;
            }
        }

        if is_maximizing {
            alpha
        } else {
            beta
        }
    }

    pub fn find_best_move(&mut self, board: &Board, max_depth: u8) -> Option<ScoringMove> {
        let color_to_play = board.side_to_move();

        let mut visited_nodes = 0;

        let mut best_move = None;
        for max_depth in 0..=max_depth {
            best_move = Some(self.minimax_search(
                &board,
                color_to_play == Color::White,
                0,
                max_depth,
                Evaluation::MIN,
                Evaluation::MAX,
                &mut visited_nodes,
            ));

            let is_mate = best_move
                .map(|scoring_move| scoring_move.evaluation.is_mate())
                .unwrap_or(false);

            if is_mate {
                break;
            }
        }

        if let Some(best_move) = best_move.clone() {
            eprintln!("move: {}", best_move);
        }
        best_move
    }
}

#[cfg(test)]
mod test {
    use crate::evaluation::Evaluation;
    use crate::search::{ScoringMove, Search};
    use chess_core::board::{Board, BoardStatus};
    use chess_core::chess_move::{Move, MoveFlag};
    use chess_core::color::Color;
    use chess_core::piece::Piece;
    use chess_core::square::Square;
    use std::str::FromStr;

    #[test]
    fn first_move() {
        let board = Board::default();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert!(best_move.is_some());
    }

    #[test]
    fn take_white_queen() {
        let board = Board::from_str("8/1kQ5/8/8/8/8/8/7K b - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
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

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(Move {
                from: Square::G3,
                to: Square::E1,
                promotion: None,
                piece: Piece::Queen,
                flags: MoveFlag::Normal,
            })
        )
    }

    #[test]
    fn take_black_queen() {
        let board = Board::from_str("8/1Kq5/8/8/8/8/8/7k w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
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
    fn back_rank_mate_black() {
        let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(Move {
                from: Square::D1,
                to: Square::D8,
                promotion: None,
                piece: Piece::Rook,
                flags: MoveFlag::Normal,
            })
        );
    }

    #[test]
    fn back_rank_mate_white() {
        let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(Move {
                from: Square::D1,
                to: Square::D8,
                promotion: None,
                piece: Piece::Rook,
                flags: MoveFlag::Normal,
            })
        );
    }

    #[test]
    fn mate_in_one_queen_rook() {
        let board = Board::from_str("1Q3q1k/p5pp/8/2p2P2/P1B2P1P/6K1/R7/8 w - - 4 41").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert_eq!(
            best_move.unwrap().chess_move,
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

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert_eq!(
            best_move.unwrap().chess_move,
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
    fn test_quiet_search() {
        let board = Board::from_str("8/8/8/5k2/4p3/3P4/8/K7 w - - 0 1").unwrap();
        let mut search = Search::new();
        let mut visited_nodes = 0;
        let eval = search.quiescence_search(
            &board,
            0,
            true,
            Evaluation::MIN,
            Evaluation::MAX,
            &mut visited_nodes,
        );
        assert_eq!(eval, Evaluation(0));
    }

    #[test]
    fn test_tries_to_move_opposite_color_piece() {
        let board =
            Board::from_str("2k3r1/p1pr1pp1/3b4/5Q2/P2Bn3/1P2P2P/2qP1PP1/RN3RK1 w - - 3 21")
                .unwrap();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        // TODO: remove test as it's not very useful
        assert_ne!(
            best_move.unwrap().chess_move,
            Some(Move {
                from: Square::D8,
                to: Square::D7,
                promotion: None,
                piece: Piece::Rook,
                flags: MoveFlag::Normal,
            })
        );
    }

    #[test]
    fn mate_in_one_two_queens() {
        let board = Board::from_str("8/1K6/6k1/r7/8/2q5/8/3q4 b - - 1 56").unwrap();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        let result = board.make_move(best_move.unwrap().chess_move.unwrap());
        assert_eq!(result.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn mate_in_one_two_ply() {
        let board = Board::from_str("1r6/8/8/8/8/8/2k5/K7 w - - 0 1").unwrap();
        let mut search = Search::new();
        let whites_move = search.find_best_move(&board, 3);
        assert_eq!(
            whites_move,
            Some(ScoringMove {
                evaluation: Evaluation::new_mate_eval(Color::Black, 2),
                chess_move: Some(Move {
                    from: Square::A1,
                    to: Square::A2,
                    promotion: None,
                    piece: Piece::King,
                    flags: MoveFlag::Normal,
                })
            })
        );
        let result = board.make_move(whites_move.unwrap().chess_move.unwrap());
        let black_move = search.find_best_move(&result, 3);
        assert_eq!(
            black_move,
            Some(ScoringMove {
                evaluation: Evaluation::new_mate_eval(Color::Black, 1),
                chess_move: Some(Move {
                    from: Square::B8,
                    to: Square::A8,
                    promotion: None,
                    piece: Piece::Rook,
                    flags: MoveFlag::Normal,
                })
            })
        );
        let result = result.make_move(black_move.unwrap().chess_move.unwrap());
        assert_eq!(result.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_hippo_defense() {
        let board = Board::from_str(
            "r2qk2r/1bpnnpb1/p2pp1pp/1p6/3PP3/PBN2NB1/1PP2PPP/R2QK2R w KQkq - 1 11",
        )
        .unwrap();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert!(best_move.is_some());
    }

    #[test]
    fn test_deep_mate() {
        // let mut board = Board::from_str("8/8/p7/K7/8/8/2k5/1R6 w - - 10 67").unwrap();
        let board = Board::from_str("6r1/5K2/8/8/7k/7P/8/8 b - - 10 67").unwrap();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 18);
        let scoring_move = best_move.expect("expected move");
        eprintln!("eval: {:?}", scoring_move.evaluation);
        assert_eq!(scoring_move.evaluation.mate_num_ply(), -16 * 2 + 1)
    }

    #[test]
    fn test_mate_in_three() {
        let mut board = Board::from_str("8/8/3k4/7R/6Q1/8/8/7K w - - 0 1").unwrap();
        let mut search = Search::new();
        for _ in 0..5 {
            let best_move = search.find_best_move(&board, 7);
            let chess_move = best_move.unwrap().chess_move.unwrap();
            board = board.make_move(chess_move);
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_two() {
        let mut board = Board::from_str("8/3k4/7R/6Q1/8/8/8/7K w - - 0 1").unwrap();
        let mut search = Search::new();
        for _ in 0..3 {
            let best_move = search.find_best_move(&board, 5);
            let chess_move = best_move.unwrap().chess_move.unwrap();
            board = board.make_move(chess_move);
        }
        assert_eq!(board.status(), BoardStatus::Checkmate);
    }

    #[test]
    fn test_mate_in_seven() {
        let mut board = Board::from_str("7k/8/1K2PPPP/8/B7/8/4pppp/8 w - - 0 1").unwrap();
        let mut search = Search::new();

        for _ in 0..13 {
            // println!("{}", board.to_string());
            let best_move = search.find_best_move(&board, 14);
            let chess_move = best_move.unwrap().chess_move.unwrap();
            board = board.make_move(chess_move);
        }
        // println!("{}", board.to_string());

        assert_eq!(board.status(), BoardStatus::Checkmate);
    }
}

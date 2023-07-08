use crate::evaluation::{board_value, raw_piece_value, Evaluation};
use crate::transposition_table::{TranspositionTable, ValueType};
use chess::{Board, ChessMove, Color, MoveGen, EMPTY};
use std::ops::Not;

pub struct Search {
    transposition_table: TranspositionTable,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ScoringMove {
    pub evaluation: Evaluation,
    pub chess_move: Option<ChessMove>,
}

impl Search {
    pub fn new() -> Self {
        Search {
            transposition_table: TranspositionTable::new(),
        }
    }

    fn alpha_beta(
        &mut self,
        board: &Board,
        ply: u8,
        depth: u8,
        color_to_move: Color,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        visited_nodes: &mut i64,
    ) -> ScoringMove {
        let entry = self
            .transposition_table
            .read_entry(board, depth, alpha, beta);

        if let Some(entry) = entry {
            return ScoringMove {
                evaluation: entry.value,
                chess_move: entry.best_move,
            };
        }

        if depth == 0 {
            let value = self.quiescence_search(
                board,
                color_to_move == Color::White,
                alpha,
                beta,
                visited_nodes,
            );
            *visited_nodes += 1;

            return ScoringMove {
                evaluation: value,
                chess_move: None,
            };
        }

        let mut best_scoring_move = ScoringMove {
            evaluation: match color_to_move {
                Color::White => Evaluation::MIN,
                Color::Black => Evaluation::MAX,
            },
            chess_move: None,
        };

        let mut moves = MoveGen::new_legal(&board).collect::<Vec<_>>();

        if moves.is_empty() {
            let value = if board.checkers() != &EMPTY {
                let color_to_move = board.side_to_move();
                Evaluation::new_mate_eval(color_to_move.not(), ply)
            } else {
                Evaluation(0)
            };

            return ScoringMove {
                evaluation: value,
                chess_move: None,
            };
        }

        moves.sort_unstable_by_key(|mov| {
            let src_piece = board.piece_on(mov.get_source());
            let dst_piece = board.piece_on(mov.get_dest());

            if let Some(dst_piece) = dst_piece {
                return -(100 * raw_piece_value(dst_piece)
                    - src_piece.map(|piece| raw_piece_value(piece)).unwrap_or(0));
            }
            0
        });

        for chess_move in moves {
            let result = board.make_move_new(chess_move);

            let scoring_move = self.alpha_beta(
                &result,
                ply + 1,
                depth - 1,
                !color_to_move,
                alpha,
                beta,
                visited_nodes,
            );

            if color_to_move == Color::White
                && best_scoring_move.evaluation < scoring_move.evaluation
            {
                best_scoring_move.evaluation = scoring_move.evaluation;
                best_scoring_move.chess_move = Some(chess_move);
                self.transposition_table.add_entry(
                    *board,
                    scoring_move.evaluation,
                    ValueType::Exact,
                    best_scoring_move.chess_move,
                    depth,
                );
                alpha = alpha.max(best_scoring_move.evaluation);
            } else if color_to_move == Color::Black
                && best_scoring_move.evaluation > scoring_move.evaluation
            {
                best_scoring_move.evaluation = scoring_move.evaluation;
                best_scoring_move.chess_move = Some(chess_move);
                self.transposition_table.add_entry(
                    *board,
                    scoring_move.evaluation,
                    ValueType::Exact,
                    best_scoring_move.chess_move,
                    depth,
                );
                beta = beta.min(best_scoring_move.evaluation);
            }

            // alpha -> lower bound
            // beta -> upper bound
            if color_to_move == Color::White && beta <= scoring_move.evaluation {
                // move too good
                self.transposition_table.add_entry(
                    *board,
                    beta,
                    ValueType::Beta,
                    Some(chess_move),
                    depth,
                );
                break;
            } else if color_to_move == Color::Black && alpha >= scoring_move.evaluation {
                self.transposition_table.add_entry(
                    *board,
                    alpha,
                    ValueType::Alpha,
                    Some(chess_move),
                    depth,
                );
                break;
            }
        }

        best_scoring_move
    }

    pub fn quiescence_search(
        &mut self,
        board: &Board,
        is_maximizing: bool,
        mut alpha: Evaluation,
        mut beta: Evaluation,
        visited_nodes: &mut i64,
    ) -> Evaluation {
        let mut eval = board_value(board);

        if is_maximizing && beta <= eval {
            return beta;
        } else if !is_maximizing && alpha >= eval {
            return alpha;
        }

        if is_maximizing && alpha < eval {
            alpha = eval;
        } else if !is_maximizing && beta > eval {
            beta = eval;
        }

        let targets = board.color_combined(!board.side_to_move());
        let mut moves = MoveGen::new_legal(&board);

        moves.set_iterator_mask(*targets);
        for chess_move in &mut moves {
            let result = board.make_move_new(chess_move);

            eval = self.quiescence_search(&result, !is_maximizing, alpha, beta, visited_nodes);

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
        for max_depth in (1..=max_depth).rev() {
            best_move = Some(self.alpha_beta(
                &board,
                0,
                max_depth,
                color_to_play,
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

        eprintln!("move: {:?}", best_move);
        best_move
    }
}

#[cfg(test)]
mod test {
    use crate::evaluation::Evaluation;
    use crate::search::{ScoringMove, Search};
    use chess::{Board, BoardStatus, ChessMove, Color};
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
            Some(ChessMove::from_str("b7c7").unwrap())
        )
    }

    #[test]
    fn take_black_queen() {
        let board = Board::from_str("8/1Kq5/8/8/8/8/8/7k w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("b7c7").unwrap())
        )
    }

    #[test]
    fn back_rank_mate_black() {
        let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("d1d8").unwrap())
        );
    }

    #[test]
    fn back_rank_mate_white() {
        let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("d8d1").unwrap())
        );
    }

    #[test]
    fn mate_in_one() {
        let board = Board::from_str("1Q3q1k/p5pp/8/2p2P2/P1B2P1P/6K1/R7/8 w - - 4 41").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("b8f8").unwrap())
        )
    }

    #[test]
    fn mate_in_one_corner() {
        let board = Board::from_str("5rk1/7p/3R2p1/3p4/7P/5pP1/5P1K/5q2 b - - 4 39").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert_eq!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("f1g2").unwrap())
        );
    }

    #[test]
    fn test_quiet_search() {
        let board = Board::from_str("8/8/8/5k2/4p3/3P4/8/K7 w - - 0 1").unwrap();
        let mut search = Search::new();
        let mut visited_nodes = 0;
        let eval = search.quiescence_search(
            &board,
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
        assert_ne!(
            best_move.unwrap().chess_move,
            Some(ChessMove::from_str("d8d7").unwrap())
        );
    }

    #[test]
    fn mate_in_one_two_queens() {
        let board = Board::from_str("8/1K6/6k1/r7/8/2q5/8/3q4 b - - 1 56").unwrap();
        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        let result = board.make_move_new(best_move.unwrap().chess_move.unwrap());
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
                chess_move: Some(ChessMove::from_str("a1a2").unwrap())
            })
        );
        let result = board.make_move_new(whites_move.unwrap().chess_move.unwrap());
        let black_move = search.find_best_move(&result, 3);
        let result = result.make_move_new(black_move.unwrap().chess_move.unwrap());
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
}

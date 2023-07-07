use crate::evaluation::piece_square_table;
use crate::transposition_table::{TranspositionTable, ValueType};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square, ALL_PIECES, EMPTY};

pub struct Search {
    transposition_table: TranspositionTable,
}

pub struct ScoringMove {
    pub evaluation: i64,
    pub chess_move: Option<ChessMove>,
}

impl Search {
    pub fn new() -> Self {
        Search {
            transposition_table: TranspositionTable::new(),
        }
    }

    fn piece_value(piece: Piece, square: Square, piece_color: Color) -> i64 {
        let sign = match piece_color {
            Color::White => 1,
            Color::Black => -1,
        };

        let piece_value = match piece {
            Piece::Pawn => 100,
            Piece::Knight => 320,
            Piece::Bishop => 330,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 20000,
        };

        let bonus = piece_square_table(piece, square, piece_color);

        sign * (piece_value + bonus)
    }

    fn board_value(board: &Board) -> i64 {
        let board_status = board.status();
        if let BoardStatus::Checkmate = board_status {
            return if let Color::White = board.side_to_move() {
                -100000
            } else {
                100000
            };
        } else if let BoardStatus::Stalemate = board_status {
            return 0;
        }

        let mut result = 0;
        for piece in ALL_PIECES {
            let bitboard = *board.pieces(piece);
            for square in bitboard {
                result += Self::piece_value(piece, square, board.color_on(square).unwrap());
            }
        }
        result
    }

    fn alpha_beta(
        &mut self,
        board: &Board,
        depth: u8,
        is_maximizing: bool,
        alpha: i64,
        beta: i64,
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

        if depth == 0 || board.status() != BoardStatus::Ongoing {
            let value = Self::board_value(board);
            *visited_nodes += 1;

            return ScoringMove {
                evaluation: value,
                chess_move: None,
            };
        }

        let mut alpha = alpha;
        let mut beta = beta;

        let targets = board.color_combined(!board.side_to_move());
        let masks = [*targets, !EMPTY];

        let mut best_scoring_move = ScoringMove {
            evaluation: if is_maximizing { i64::MIN } else { i64::MAX },
            chess_move: None,
        };

        let mut moves = MoveGen::new_legal(&board);

        for mask in masks {
            moves.set_iterator_mask(mask);
            for chess_move in &mut moves {
                let result = board.make_move_new(chess_move);

                if result.status() == BoardStatus::Checkmate {
                    let value = Self::board_value(&result);
                    return ScoringMove {
                        evaluation: value,
                        chess_move: Some(chess_move),
                    };
                }

                let scoring_move = self.alpha_beta(
                    &result,
                    depth - 1,
                    !is_maximizing,
                    alpha,
                    beta,
                    visited_nodes,
                );

                if is_maximizing && best_scoring_move.evaluation < scoring_move.evaluation {
                    best_scoring_move.evaluation = scoring_move.evaluation;
                    best_scoring_move.chess_move = Some(chess_move);
                    self.transposition_table.add_entry(
                        result,
                        scoring_move.evaluation,
                        ValueType::Exact,
                        best_scoring_move.chess_move,
                        depth,
                    );
                    alpha = alpha.max(best_scoring_move.evaluation);
                } else if !is_maximizing && best_scoring_move.evaluation > scoring_move.evaluation {
                    best_scoring_move.evaluation = scoring_move.evaluation;
                    best_scoring_move.chess_move = Some(chess_move);
                    self.transposition_table.add_entry(
                        result,
                        scoring_move.evaluation,
                        ValueType::Exact,
                        best_scoring_move.chess_move,
                        depth,
                    );
                    beta = beta.min(best_scoring_move.evaluation);
                }

                // alpha -> lower bound
                // beta -> upper bound
                if is_maximizing && beta <= scoring_move.evaluation {
                    // move too good
                    self.transposition_table.add_entry(
                        result,
                        beta,
                        ValueType::Beta,
                        Some(chess_move),
                        depth,
                    );
                    break;
                } else if !is_maximizing && alpha >= scoring_move.evaluation {
                    self.transposition_table.add_entry(
                        result,
                        alpha,
                        ValueType::Alpha,
                        Some(chess_move),
                        depth,
                    );
                    break;
                }
            }
        }

        best_scoring_move
    }

    pub fn find_best_move(&mut self, board: &Board, max_depth: u8) -> Option<ChessMove> {
        let color_to_play = board.side_to_move();
        eprintln!("Current val: {}", Self::board_value(board));

        let mut visited_nodes = 0;
        let best_move = self.alpha_beta(
            &board,
            max_depth,
            color_to_play == Color::White,
            i64::MIN,
            i64::MAX,
            &mut visited_nodes,
        );
        eprintln!(
            "move {:?} visited nodes: {}",
            best_move.chess_move, visited_nodes
        );

        best_move.chess_move
    }
}

#[cfg(test)]
mod test {
    use crate::search::Search;
    use chess::{Board, ChessMove};
    use std::str::FromStr;

    #[test]
    fn take_white_queen() {
        let board = Board::from_str("8/1kQ5/8/8/8/8/8/7K b - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("b7c7").unwrap()))
    }

    #[test]
    fn take_black_queen() {
        let board = Board::from_str("8/1Kq5/8/8/8/8/8/7k w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("b7c7").unwrap()))
    }

    #[test]
    fn back_rank_mate_black() {
        let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("d1d8").unwrap()));
    }

    #[test]
    fn back_rank_mate_white() {
        let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("d8d1").unwrap()));
    }

    #[test]
    fn mate_in_one() {
        let board = Board::from_str("1Q3q1k/p5pp/8/2p2P2/P1B2P1P/6K1/R7/8 w - - 4 41").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        assert_eq!(best_move, Some(ChessMove::from_str("b8f8").unwrap()))
    }

    #[test]
    fn mate_in_one_corner() {
        let board = Board::from_str("5rk1/7p/3R2p1/3p4/7P/5pP1/5P1K/5q2 b - - 4 39").unwrap();

        let mut search = Search::new();
        let best_move = search.find_best_move(&board, 7);
        search.find_best_move(&board, 14);
        assert_eq!(best_move, Some(ChessMove::from_str("f1g2").unwrap()));
    }
}

use crate::transposition_table::{TranspositionTable, ValueType};
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square, ALL_PIECES};

pub struct Search {
    transposition_table: TranspositionTable,
}

impl Search {
    pub fn new() -> Self {
        Search {
            transposition_table: TranspositionTable::new(),
        }
    }

    fn piece_value(piece: Piece, square: Square, piece_color: Option<Color>) -> i64 {
        // TODO: make value depend on the position
        let _square_index = square.to_index();

        let sign = match piece_color {
            Some(Color::White) => 1,
            Some(Color::Black) => -1,
            None => 0,
        };

        let piece_value = match piece {
            Piece::Pawn => 100,
            Piece::Knight => 300,
            Piece::Bishop => 300,
            Piece::Rook => 500,
            Piece::Queen => 900,
            Piece::King => 10000,
        };

        sign * piece_value
    }

    fn board_value(board: &Board) -> i64 {
        let board_status = board.status();
        if let BoardStatus::Checkmate = board_status {
            return if let Color::White = board.side_to_move() {
                -1000000
            } else {
                1000000
            };
        } else if let BoardStatus::Stalemate = board_status {
            return 0;
        }

        let mut result = 0;
        for piece in ALL_PIECES {
            let bitboard = *board.pieces(piece);
            for square in bitboard {
                result += Self::piece_value(piece, square, board.color_on(square));
            }
        }
        result
    }

    fn alpha_beta(
        &self,
        board: &Board,
        depth: u8,
        is_maximizing: bool,
        alpha: i64,
        beta: i64,
    ) -> i64 {
        if depth == 0 || board.status() != BoardStatus::Ongoing {
            let value = Self::board_value(board);
            return value;
        }

        let mut alpha = alpha;
        let mut beta = beta;

        let moves = MoveGen::new_legal(&board);
        let mut result = Board::default();

        if is_maximizing {
            let mut max_eval = i64::MIN;
            for chess_move in moves {
                board.make_move(chess_move, &mut result);
                let eval = self.alpha_beta(&result, depth - 1, false, alpha, beta);

                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }

            max_eval
        } else {
            let mut min_eval = i64::MAX;
            for chess_move in moves {
                board.make_move(chess_move, &mut result);
                let eval = self.alpha_beta(&result, depth - 1, true, alpha, beta);

                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }

            min_eval
        }
    }

    pub fn find_best_move(&self, board: &Board, max_depth: u8) -> Option<ChessMove> {
        let color_to_play = board.side_to_move();
        eprintln!("Current val: {}", Self::board_value(board));
        let moves = MoveGen::new_legal(&board);

        let comparator = match color_to_play {
            Color::White => i64::gt,
            Color::Black => i64::lt,
        };

        let mut best_value = match color_to_play {
            Color::White => i64::MIN,
            Color::Black => i64::MAX,
        };

        let mut best_move: Option<ChessMove> = None;

        for chess_move in moves {
            let mut result = Board::default();
            board.make_move(chess_move, &mut result);
            let value = self.alpha_beta(
                &result,
                max_depth,
                color_to_play == Color::Black,
                i64::MIN,
                i64::MAX,
            );
            if comparator(&value, &best_value) {
                best_value = value;
                best_move = Some(chess_move);
            }
        }

        best_move
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

        let search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("b7c7").unwrap()))
    }

    #[test]
    fn take_black_queen() {
        let board = Board::from_str("8/1Kq5/8/8/8/8/8/7k w - - 0 1").unwrap();

        let search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("b7c7").unwrap()))
    }

    #[test]
    fn back_rank_mate_black() {
        let board = Board::from_str("6k1/5ppp/8/8/8/8/8/K2R4 w - - 0 1").unwrap();

        let search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("d1d8").unwrap()));
    }

    #[test]
    fn back_rank_mate_white() {
        let board = Board::from_str("3r3k/8/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();

        let search = Search::new();
        let best_move = search.find_best_move(&board, 6);
        assert_eq!(best_move, Some(ChessMove::from_str("d8d1").unwrap()));
    }
}

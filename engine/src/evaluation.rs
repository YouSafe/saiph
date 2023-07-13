use crate::piece_square_table::piece_square_table;
use chess::{Board, Color, Piece, Square, ALL_PIECES};
use std::ops::Neg;

#[derive(PartialEq, Clone, Copy, Debug, PartialOrd, Ord, Eq)]
pub struct Evaluation(pub i32);

impl Evaluation {
    pub const MIN: Evaluation = Evaluation(i32::MIN + 1);
    pub const MAX: Evaluation = Evaluation(i32::MAX);

    // [MIN, ..., -IMMEDIATE_MATE_SCORE, ..., IMMEDIATE_MATE_SCORE + MAX_MATE_DEPTH,
    // ..., SCORE, ...,
    // IMMEDIATE_MATE_SCORE - MAX_MATE_DEPTH, ..., IMMEDIATE_MATE_SCORE, ..., MAX]

    const IMMEDIATE_MATE_SCORE: i32 = 100_000;
    const MAX_MATE_DEPTH: i32 = 1000;

    pub const fn is_mate(&self) -> bool {
        self.0.abs() > (Evaluation::IMMEDIATE_MATE_SCORE - Evaluation::MAX_MATE_DEPTH)
    }

    pub const fn mate_num_ply(&self) -> i8 {
        assert!(self.is_mate());
        (self.0.signum() * (Evaluation::IMMEDIATE_MATE_SCORE - self.0.abs())) as i8
    }

    pub fn new_mate_eval(mating_color: Color, ply_from_root: u8) -> Evaluation {
        let sign = match mating_color {
            Color::White => 1,
            Color::Black => -1,
        };

        Evaluation(sign * (Evaluation::IMMEDIATE_MATE_SCORE - ply_from_root as i32))
    }

    pub const fn score_to_tt(&self, ply: u8) -> Evaluation {
        assert!(self.is_mate());
        Evaluation((self.0.abs() + ply as i32) * self.0.signum())
    }

    pub const fn tt_to_score(&self, ply: u8) -> Evaluation {
        assert!(self.is_mate());
        Evaluation((self.0.abs() - ply as i32) * self.0.signum())
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Evaluation(self.0.neg())
    }
}

pub const fn raw_piece_value(piece: Piece) -> i32 {
    let piece_value = match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20000,
    };
    piece_value
}

fn piece_value(piece: Piece, square: Square, piece_color: Color) -> i32 {
    let sign = match piece_color {
        Color::White => 1,
        Color::Black => -1,
    };

    let piece_value = raw_piece_value(piece);

    let bonus = piece_square_table(piece, square, piece_color);

    sign * (piece_value + bonus)
}

pub fn board_value(board: &Board) -> Evaluation {
    let mut result: i32 = 0;
    for piece in ALL_PIECES {
        let bitboard = *board.pieces(piece);
        for square in bitboard {
            result += piece_value(piece, square, board.color_on(square).unwrap());
        }
    }
    Evaluation(result)
}

#[cfg(test)]
mod test {
    use crate::evaluation::Evaluation;
    use chess::Color;

    #[test]
    fn test_adjust_mate_ply() {
        // store current position as mate in 10 ply
        let store_ply = 10;

        // retrieve stored position later at 5 and 8 ply depth
        // at ply 5 -> mate in 5 ply
        // at ply 2 -> mate in 2 ply
        let retrieval_ply = 5;
        let retrieval_ply2 = 2;

        // eval for current position
        let white_mate = Evaluation::new_mate_eval(Color::White, 10);
        let stored_white_mate = white_mate.score_to_tt(store_ply);

        // write
        // white -> increase score
        assert_eq!(
            stored_white_mate,
            Evaluation::new_mate_eval(Color::White, 0)
        );

        // read
        // white -> decrease score
        let retrieved_white_mate = stored_white_mate.tt_to_score(retrieval_ply);
        let retrieved_white_mate2 = stored_white_mate.tt_to_score(retrieval_ply2);
        assert_eq!(
            retrieved_white_mate,
            Evaluation::new_mate_eval(Color::White, 5)
        );
        assert_eq!(
            retrieved_white_mate2,
            Evaluation::new_mate_eval(Color::White, 2)
        );

        let black_mate = Evaluation::new_mate_eval(Color::Black, 10);
        let stored_black_mate = black_mate.score_to_tt(store_ply);
        // black -> decrease score
        assert_eq!(
            stored_black_mate,
            Evaluation::new_mate_eval(Color::Black, 0)
        );
        // read
        // black -> increase score
        let retrieved_black_mate = stored_black_mate.tt_to_score(retrieval_ply);
        let retrieved_black_mate2 = stored_black_mate.tt_to_score(retrieval_ply2);
        assert_eq!(
            retrieved_black_mate,
            Evaluation::new_mate_eval(Color::Black, 5)
        );
        assert_eq!(
            retrieved_black_mate2,
            Evaluation::new_mate_eval(Color::Black, 2)
        );
    }

    #[test]
    fn test_checkmate_eval() {
        let mate = Evaluation::new_mate_eval(Color::Black, 0);
        assert!(mate.is_mate());
    }
}

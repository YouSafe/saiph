use crate::piece_square_table::piece_square_table;
use chess::{Board, Color, Piece, Square, ALL_PIECES};

#[derive(PartialEq, Clone, Copy, Eq, Debug, PartialOrd, Ord)]
pub struct Evaluation(pub i32);

impl Evaluation {
    pub const MIN: Evaluation = Evaluation(i32::MIN + 1);
    pub const MAX: Evaluation = Evaluation(i32::MAX);

    const IMMEDIATE_MATE_SCORE: i32 = 100_000;
    const MAX_MATE_DEPTH: i32 = 1000;

    pub const fn is_mate(&self) -> bool {
        self.0.abs() > Evaluation::IMMEDIATE_MATE_SCORE - Evaluation::MAX_MATE_DEPTH
    }

    pub const fn mate_num_ply(&self) -> i8 {
        assert!(self.is_mate());
        (Evaluation::IMMEDIATE_MATE_SCORE - self.0) as i8
    }

    pub const fn new_mate_eval(mating_color: Color, ply_from_root: u8) -> Evaluation {
        let sign = match mating_color {
            Color::White => 1,
            Color::Black => -1,
        };

        Evaluation(sign * (Evaluation::IMMEDIATE_MATE_SCORE - ply_from_root as i32))
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
mod test {}

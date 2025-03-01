use crate::board::Board;
use crate::types::color::Color;
use crate::types::piece::{Piece, ALL_PIECES};
use crate::types::square::Square;

use super::Evaluation;

pub const fn raw_piece_value(piece: Piece) -> i16 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        // The value of king is defined to be 0 to avoid arithmetic overflow. This is valid because
        // both sides always have exactly one king. In classic chess the king can not be captured
        // but other chess variants might allow capturing the king. However, other variants are not
        // supported yet by this chess engine.
        Piece::King => 0,
    }
}

fn piece_value(piece: Piece, square: Square, piece_color: Color) -> i16 {
    let sign = match piece_color {
        Color::White => 1,
        Color::Black => -1,
    };

    let piece_value = raw_piece_value(piece);

    let bonus = piece_square_table(piece, square, piece_color);

    sign * (piece_value + bonus)
}

pub fn board_value(board: &Board) -> Evaluation {
    let mut result: i16 = 0;
    for piece in ALL_PIECES {
        for square in board.pieces(piece).iter() {
            result += piece_value(piece, square, board.color_at(square).unwrap());
        }
    }
    Evaluation(result)
}

// See: https://www.chessprogramming.org/Simplified_Evaluation_Function

pub fn piece_square_table(piece: Piece, square: Square, piece_color: Color) -> i16 {
    let square_index = square as usize;

    let rank = square.to_rank() as usize;
    let file = square.to_file() as usize;

    let lookup_index = match piece_color {
        Color::White => (7 - rank) * 8 + file,
        Color::Black => square_index,
    };

    (match piece {
        Piece::Pawn => PAWNS_TABLE[lookup_index],
        Piece::Knight => KNIGHTS_TABLE[lookup_index],
        Piece::Bishop => BISHOP_TABLE[lookup_index],
        Piece::Rook => ROOK_TABLE[lookup_index],
        Piece::Queen => QUEEN_TABLE[lookup_index],
        Piece::King => 0,
    }) as i16
}

#[rustfmt::skip]
const PAWNS_TABLE: [i8; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
const KNIGHTS_TABLE: [i8; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i8; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i8; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
const QUEEN_TABLE: [i8; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];
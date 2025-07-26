use crate::board::Board;
use crate::types::color::Color;
use crate::types::piece::{ALL_PIECES, PieceType};
use crate::types::square::Square;

use super::Evaluation;

pub const fn raw_piece_value(piece: PieceType) -> i16 {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        // The value of king is defined to be 0 to avoid arithmetic overflow. This is valid because
        // both sides always have exactly one king. In classic chess the king can not be captured
        // but other chess variants might allow capturing the king. However, other variants are not
        // supported yet by this chess engine.
        PieceType::King => 0,
    }
}

fn piece_value(piece: PieceType, square: Square, piece_color: Color) -> i16 {
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
            result += piece_value(piece, square, board.piece_at(square).unwrap().color());
        }
    }
    Evaluation(result)
}

// See: https://www.chessprogramming.org/Simplified_Evaluation_Function

pub fn piece_square_table(piece: PieceType, square: Square, piece_color: Color) -> i16 {
    let square_index = square as usize;

    let rank = square.to_rank() as usize;
    let file = square.to_file() as usize;

    let lookup_index = match piece_color {
        Color::White => (7 - rank) * 8 + file,
        Color::Black => square_index,
    };

    (match piece {
        PieceType::Pawn => PAWNS_TABLE[lookup_index],
        PieceType::Knight => KNIGHTS_TABLE[lookup_index],
        PieceType::Bishop => BISHOP_TABLE[lookup_index],
        PieceType::Rook => ROOK_TABLE[lookup_index],
        PieceType::Queen => QUEEN_TABLE[lookup_index],
        PieceType::King => 0,
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

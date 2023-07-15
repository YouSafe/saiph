use crate::color::Color;
use crate::piece::Piece::{Bishop, King, Knight, Pawn, Queen, Rook};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

pub const NUM_PIECES: usize = 6;
pub const ALL_PIECES: [Piece; 6] = [Pawn, Knight, Bishop, Rook, Queen, King];

impl Piece {
    pub fn to_unicode(&self, color: Color) -> char {
        match (color, *self) {
            (Color::White, Piece::Pawn) => '♙',
            (Color::White, Piece::Knight) => '♘',
            (Color::White, Piece::Bishop) => '♗',
            (Color::White, Piece::Rook) => '♖',
            (Color::White, Piece::Queen) => '♕',
            (Color::White, Piece::King) => '♔',

            (Color::Black, Piece::Pawn) => '♟',
            (Color::Black, Piece::Knight) => '♞',
            (Color::Black, Piece::Bishop) => '♝',
            (Color::Black, Piece::Rook) => '♜',
            (Color::Black, Piece::Queen) => '♛',
            (Color::Black, Piece::King) => '♚',
        }
    }

    pub fn to_ascii(&self, color: Color) -> char {
        match (color, *self) {
            (Color::White, Piece::Pawn) => 'P',
            (Color::White, Piece::Knight) => 'N',
            (Color::White, Piece::Bishop) => 'B',
            (Color::White, Piece::Rook) => 'R',
            (Color::White, Piece::Queen) => 'Q',
            (Color::White, Piece::King) => 'K',

            (Color::Black, Piece::Pawn) => 'p',
            (Color::Black, Piece::Knight) => 'n',
            (Color::Black, Piece::Bishop) => 'b',
            (Color::Black, Piece::Rook) => 'r',
            (Color::Black, Piece::Queen) => 'q',
            (Color::Black, Piece::King) => 'k',
        }
    }
}

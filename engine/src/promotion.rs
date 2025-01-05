use crate::piece::Piece;
use crate::promotion::Promotion::{Bishop, Knight, Queen, Rook};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Knight,
    Bishop,
}

impl Promotion {
    pub const fn as_piece(&self) -> Piece {
        match self {
            Queen => Piece::Queen,
            Rook => Piece::Rook,
            Knight => Piece::Knight,
            Bishop => Piece::Bishop,
        }
    }
}

pub const ALL_PROMOTIONS: [Promotion; 4] = [Queen, Rook, Knight, Bishop];

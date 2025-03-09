use crate::types::piece::PieceType;
use crate::types::promotion::Promotion::{Bishop, Knight, Queen, Rook};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Knight,
    Bishop,
}

impl Promotion {
    pub const fn as_piece_type(&self) -> PieceType {
        match self {
            Queen => PieceType::Queen,
            Rook => PieceType::Rook,
            Knight => PieceType::Knight,
            Bishop => PieceType::Bishop,
        }
    }
}

pub const ALL_PROMOTIONS: [Promotion; 4] = [Queen, Rook, Knight, Bishop];

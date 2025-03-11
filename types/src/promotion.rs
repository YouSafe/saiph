use crate::piece::PieceType;

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
            Promotion::Queen => PieceType::Queen,
            Promotion::Rook => PieceType::Rook,
            Promotion::Knight => PieceType::Knight,
            Promotion::Bishop => PieceType::Bishop,
        }
    }
}

pub const ALL_PROMOTIONS: [Promotion; 4] = [
    Promotion::Queen,
    Promotion::Rook,
    Promotion::Knight,
    Promotion::Bishop,
];

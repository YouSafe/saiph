use crate::promotion::Promotion::{Bishop, Knight, Queen, Rook};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Knight,
    Bishop,
}

pub const ALL_PROMOTIONS: [Promotion; 4] = [Queen, Rook, Knight, Bishop];

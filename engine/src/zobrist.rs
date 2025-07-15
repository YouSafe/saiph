use crate::types::{
    castling_rights::CastlingRights,
    color::Color,
    piece::PieceType,
    square::{File, Square},
};

mod internal {
    include!(concat!(env!("OUT_DIR"), "/zobrist.rs"));
}

#[inline(always)]
pub fn piece_keys(color: Color, piece_type: PieceType, square: Square) -> u64 {
    internal::PIECE_KEYS[color as usize][piece_type as usize][square as usize]
}

#[inline(always)]
pub fn en_passant_keys(file: File) -> u64 {
    internal::EN_PASSANT_KEYS[file as usize]
}

#[inline(always)]
pub fn castle_keys(castling_rights: CastlingRights) -> u64 {
    internal::CASTLE_KEYS[usize::from(castling_rights)]
}

#[inline(always)]
pub fn side_key() -> u64 {
    internal::SIDE_KEY
}

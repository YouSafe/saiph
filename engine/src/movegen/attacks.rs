use crate::types::bitboard::BitBoard;
use crate::types::color::Color;
use crate::types::square::Square;

mod internal {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
    include!(concat!(env!("OUT_DIR"), "/magics.rs"));
}

pub fn bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::BISHOP_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
    BitBoard(internal::SLIDER_ATTACKS[magic_index as usize])
}

pub fn rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::ROOK_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
    BitBoard(internal::SLIDER_ATTACKS[magic_index as usize])
}

pub fn pawn_attacks(square: Square, color: Color) -> BitBoard {
    BitBoard(internal::PAWN_ATTACKS[color as usize][square as usize])
}

pub fn knight_attacks(square: Square) -> BitBoard {
    BitBoard(internal::KNIGHT_ATTACKS[square as usize])
}

pub fn king_attacks(square: Square) -> BitBoard {
    BitBoard(internal::KING_ATTACKS[square as usize])
}

pub fn queen_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    rook_attacks(square, blockers) | bishop_attacks(square, blockers)
}

pub fn between(from: Square, to: Square) -> BitBoard {
    BitBoard(internal::SQUARES_BETWEEN[from as usize][to as usize])
}

pub fn line(from: Square, target: Square) -> BitBoard {
    BitBoard(internal::SQUARES_LINE[from as usize][target as usize])
}

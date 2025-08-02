use crate::types::bitboard::BitBoard;
use crate::types::color::Color;
use crate::types::square::{Rank, Square};

mod internal {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
    include!(concat!(env!("OUT_DIR"), "/magics.rs"));
}

pub fn bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::BISHOP_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
    BitBoard(unsafe { *internal::SLIDER_ATTACKS.get_unchecked(magic_index as usize) })
}

pub fn rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::ROOK_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
    BitBoard(unsafe { *internal::SLIDER_ATTACKS.get_unchecked(magic_index as usize) })
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

pub fn slider_horizontal(square: Square, blockers: BitBoard) -> BitBoard {
    let rankx8 = square.rank() as i32 * 8;
    let file = square.file();

    // mask for rank
    let mask = BitBoard(0xFF << rankx8);

    // mask and map to first rank
    let mapped = Square::from(Rank::R1, file) as usize;
    let occupancy = (blockers & mask) >> rankx8;
    let exclude_edges = (occupancy.0 >> 1) & 0x3F;

    let attacks = internal::FIRST_RANK_ATTACKS[mapped as usize][exclude_edges as usize] as u64;

    // unmap from first rank
    BitBoard(attacks << rankx8)
}

use crate::types::bitboard::BitBoard;
use crate::types::color::Color;
use crate::types::direction::{Direction, RelativeDir};
use crate::types::square::{Rank, Square};

mod internal {
    include!(concat!(env!("OUT_DIR"), "/tables.rs"));
    include!(concat!(env!("OUT_DIR"), "/magics.rs"));
}

pub fn bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::BISHOP_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
    unsafe { std::hint::assert_unchecked((magic_index as usize) < internal::SLIDER_ATTACKS.len()) };
    BitBoard(internal::SLIDER_ATTACKS[magic_index as usize])
}

pub fn rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &internal::ROOK_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
    unsafe { std::hint::assert_unchecked((magic_index as usize) < internal::SLIDER_ATTACKS.len()) };
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

pub const fn between(from: Square, to: Square) -> BitBoard {
    BitBoard(internal::SQUARES_BETWEEN[from as usize][to as usize])
}

pub fn pawn_attacks_all(bb: BitBoard, color: Color) -> BitBoard {
    bb.masked_shift_oriented(RelativeDir::ForwardRight, color)
        | bb.masked_shift_oriented(RelativeDir::ForwardLeft, color)
}

pub fn knight_attacks_all(knights: BitBoard) -> BitBoard {
    let l1 = knights.masked_shift::<1>(Direction::W);
    let l2 = knights.masked_shift::<2>(Direction::W);
    let r1 = knights.masked_shift::<1>(Direction::E);
    let r2 = knights.masked_shift::<2>(Direction::E);

    let h1 = l1 | r1;
    let h2 = l2 | r2;

    h1.masked_shift::<2>(Direction::N)
        | h1.masked_shift::<2>(Direction::S)
        | h2.masked_shift::<1>(Direction::N)
        | h2.masked_shift::<1>(Direction::S)
}

pub fn king_attacks_all(kings: BitBoard) -> BitBoard {
    let l = kings.masked_shift::<1>(Direction::W);
    let r = kings.masked_shift::<1>(Direction::E);
    let h = l | r;
    let attacks = h | kings;

    attacks.masked_shift::<1>(Direction::N) | attacks.masked_shift::<1>(Direction::S) | h
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

    let attacks = internal::FIRST_RANK_ATTACKS[mapped][exclude_edges as usize] as u64;

    // unmap from first rank
    BitBoard(attacks << rankx8)
}

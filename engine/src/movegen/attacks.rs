use types::bitboard::BitBoard;
use types::color::Color;
use types::color::PerColor;
use types::square::PerSquare;
use types::square::Square;

include!(concat!(env!("OUT_DIR"), "/tables.rs"));
            
include!(concat!(env!("OUT_DIR"), "/magics.rs"));

#[repr(C)]
pub struct Magic {
    pub magic: u64,
    pub mask: u64,
    pub offset: u64,
}

pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &BISHOP_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
    SLIDER_ATTACKS[magic_index as usize]
}

pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &ROOK_MAGICS[square as usize];
    let magic_index =
        ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
    SLIDER_ATTACKS[magic_index as usize]
}

pub fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
    PAWN_ATTACKS[color][square]
}

pub fn get_knight_attacks(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square]
}

pub fn get_king_attacks(square: Square) -> BitBoard {
    KING_ATTACKS[square]
}

pub fn get_queen_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_attacks(square, blockers) | get_bishop_attacks(square, blockers)
}

pub fn between(from: Square, to: Square) -> BitBoard {
    SQUARES_BETWEEN[from][to]
}

pub fn line(from: Square, target: Square) -> BitBoard {
    SQUARES_LINE[from][target]
}

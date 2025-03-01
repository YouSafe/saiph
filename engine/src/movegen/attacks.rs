use crate::types::bitboard::BitBoard;
use crate::types::color::Color;
use crate::types::square::Square;

static PAWN_ATTACKS: [[BitBoard; 64]; 2] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/pawn_attacks"))) };

static KNIGHT_ATTACKS: [BitBoard; 64] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/knight_attacks"))) };

static KING_ATTACKS: [BitBoard; 64] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/king_attacks"))) };

static SLIDER_ATTACKS: [BitBoard; 88772] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/slider_attacks"))) };

static SQUARES_BETWEEN: [[BitBoard; 64]; 64] = unsafe {
    std::mem::transmute(*include_bytes!(concat!(
        env!("OUT_DIR"),
        "/squares_between"
    )))
};

static SQUARES_LINE: [[BitBoard; 64]; 64] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/squares_line"))) };

static ROOK_MAGICS: [Magic; 64] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/rook_magics"))) };

static BISHOP_MAGICS: [Magic; 64] =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/bishop_magics"))) };

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
    PAWN_ATTACKS[color as usize][square as usize]
}

pub fn get_knight_attacks(square: Square) -> BitBoard {
    KNIGHT_ATTACKS[square as usize]
}

pub fn get_king_attacks(square: Square) -> BitBoard {
    KING_ATTACKS[square as usize]
}

pub fn get_queen_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_attacks(square, blockers) | get_bishop_attacks(square, blockers)
}

pub fn between(from: Square, to: Square) -> BitBoard {
    SQUARES_BETWEEN[from as usize][to as usize]
}

pub fn line(from: Square, target: Square) -> BitBoard {
    SQUARES_LINE[from as usize][target as usize]
}

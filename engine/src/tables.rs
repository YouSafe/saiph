use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::square::Square;

#[repr(C)]
pub struct Tables {
    pawn_attacks: [[BitBoard; 64]; 2],
    king_attacks: [BitBoard; 64],
    squares_between: [[BitBoard; 64]; 64],
    squares_line: [[BitBoard; 64]; 64],
    knight_attacks: [BitBoard; 64],
    slider_attacks: [BitBoard; 88772],
    bishop_magics: [Magic; 64],
    rook_magics: [Magic; 64],
}

pub struct Magic {
    pub magic: u64,
    pub mask: u64,
    pub offset: u64,
}

static TABLES: Tables =
    unsafe { std::mem::transmute(*include_bytes!("../../tables.bin")) };

pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &TABLES.bishop_magics[square as usize];
    let magic_index = ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
    TABLES.slider_attacks[magic_index as usize]
}

pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic = &TABLES.rook_magics[square as usize];
    let magic_index = ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
    TABLES.slider_attacks[magic_index as usize]
}

pub fn get_pawn_attacks(square: Square, color: Color) -> BitBoard {
    TABLES.pawn_attacks[color as usize][square as usize]
}

pub fn get_knight_attacks(square: Square) -> BitBoard {
    TABLES.knight_attacks[square as usize]
}

pub fn get_king_attacks(square: Square) -> BitBoard {
    TABLES.king_attacks[square as usize]
}

pub fn get_queen_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    get_rook_attacks(square, blockers) | get_bishop_attacks(square, blockers)
}

pub fn between(from: Square, to: Square) -> BitBoard {
    TABLES.squares_between[from as usize][to as usize]
}

pub fn line(from: Square, target: Square) -> BitBoard {
    TABLES.squares_line[from as usize][target as usize]
}
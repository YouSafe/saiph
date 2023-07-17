use chess_core::bitboard::BitBoard;
use chess_core::color::Color;
use chess_core::square::Square;

#[derive(Debug, Default, Clone, Copy)]
pub struct Magic {
    pub magic_number: u64,
    pub shift: u8,
    pub mask: BitBoard,
}

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic_number = BISHOP_MAGIC_NUMBERS[square as usize];
    let magic_index =
        ((blockers & magic_number.mask) * magic_number.magic_number).0 >> magic_number.shift;
    BISHOP_ATTACKS[magic_index as usize][square as usize]
}

pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let magic_number = ROOK_MAGIC_NUMBERS[square as usize];
    let magic_index =
        ((blockers & magic_number.mask) * magic_number.magic_number).0 >> magic_number.shift;
    ROOK_ATTACKS[magic_index as usize][square as usize]
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

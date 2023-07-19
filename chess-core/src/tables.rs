use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::square::Square;
use crate::tables::bishop_move::{generate_bishop_attacks, BISHOP_MAGIC_NUMBERS};
use crate::tables::king_move::generate_king_attacks;
use crate::tables::knight_move::generate_knight_attacks;
use crate::tables::pawn_move::generate_pawn_attacks;
use crate::tables::rays_between::generate_rays_between;
use crate::tables::rook_move::{generate_rook_attacks, ROOK_MAGIC_NUMBERS};
use crate::tables::xray_line::generate_xray_lines;
use lazy_static::lazy_static;

pub mod bishop_move;
pub mod king_move;
pub mod knight_move;
pub mod magic_number;
pub mod pawn_move;

pub mod rays_between;
pub mod rook_move;
pub mod xray_line;

static PAWN_ATTACKS: [[BitBoard; 64]; 2] = generate_pawn_attacks();
static KING_ATTACKS: [BitBoard; 64] = generate_king_attacks();
static SQUARES_BETWEEN: [[BitBoard; 64]; 64] = generate_rays_between();
static SQUARES_LINE: [[BitBoard; 64]; 64] = generate_xray_lines();
static KNIGHT_ATTACKS: [BitBoard; 64] = generate_knight_attacks();

lazy_static! {
    static ref ROOK_ATTACKS: Vec<[BitBoard; 64]> = generate_rook_attacks();
    static ref BISHOP_ATTACKS: Vec<[BitBoard; 64]> = generate_bishop_attacks();
}

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

pub fn line(from: Square, target: Square) -> BitBoard {
    SQUARES_LINE[from as usize][target as usize]
}

#[cfg(test)]
mod test {
    use crate::tables::bishop_move::generate_bishop_attacks;
    use crate::tables::king_move::generate_king_attacks;
    use crate::tables::knight_move::generate_knight_attacks;
    use crate::tables::pawn_move::generate_pawn_attacks;
    use crate::tables::rays_between::generate_rays_between;
    use crate::tables::rook_move::generate_rook_attacks;
    use std::time::Instant;

    #[test]
    fn test_generation() {
        let time = Instant::now();
        generate_pawn_attacks();
        generate_king_attacks();
        generate_knight_attacks();
        generate_bishop_attacks();
        generate_rook_attacks();
        generate_rays_between();
        println!("elapsed: {:?}", time.elapsed());
    }
}

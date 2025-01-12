use std::{
    fmt::{self, Formatter},
    fs::File,
    io::Write,
    mem,
    ops::BitOrAssign,
};

use king_move::generate_king_attacks;
use knight_move::generate_knight_attacks;
use magics::{Magic, BISHOP_MAGICS, ROOK_MAGICS, SLIDER_ATTACK_TABLE_SIZE};
use pawn_move::generate_pawn_attacks;
use rays_between::generate_squares_between;
use slider_move::generate_slider_attacks;
use xray_line::generate_squares_line;

mod king_move;
mod knight_move;
mod magics;
mod pawn_move;
mod rays_between;
mod slider_move;
mod xray_line;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoard(pub u64);

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    A8=56, B8, C8, D8, E8, F8, G8, H8,
    A7=48, B7, C7, D7, E7, F7, G7, H7, 
    A6=40, B6, C6, D6, E6, F6, G6, H6, 
    A5=32, B5, C5, D5, E5, F5, G5, H5,  
    A4=24, B4, C4, D4, E4, F4, G4, H4,  
    A3=16, B3, C3, D3, E3, F3, G3, H3, 
    A2= 8, B2, C2, D2, E2, F2, G2, H2,  
    A1= 0, B1, C1, D1, E1, F1, G1, H1, 
}

impl Square {
    pub const fn to_index(&self) -> u8 {
        *self as u8
    }

    pub const fn from_index(index: u8) -> Square {
        assert!(index < 64);
        unsafe { std::mem::transmute::<u8, Square>(index) }
    }
}

pub const NUM_SQUARES: usize = 64;

impl BitBoard {
    pub const fn from_square(square: Square) -> BitBoard {
        BitBoard(1 << square.to_index())
    }

    pub const EMPTY: BitBoard = BitBoard(0);

    pub const NOT_1ST_RANK: BitBoard = BitBoard(18446744073709551360);
    pub const NOT_8TH_RANK: BitBoard = BitBoard(72057594037927935);

    pub const NOT_A_FILE: BitBoard = BitBoard(18374403900871474942);
    pub const NOT_H_FILE: BitBoard = BitBoard(9187201950435737471);
    pub const NOT_AB_FILE: BitBoard = BitBoard(18229723555195321596);
    pub const NOT_GH_FILE: BitBoard = BitBoard(4557430888798830399);
}

impl BitOrAssign<Square> for BitBoard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= BitBoard::from_square(rhs).0
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            write!(f, "{}   ", rank + 1)?;
            for file in 0..8 {
                let square = rank * 8 + file;
                let value = (self.0 & (1 << square)) == (1 << square);
                let symbol = if value { 'X' } else { '.' };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{} ", file)?;
        }

        write!(f, "\n\nBitboard: {}", self.0)
    }
}

#[repr(C)]
pub struct Tables {
    pawn_attacks: [[BitBoard; 64]; 2],
    king_attacks: [BitBoard; 64],
    squares_between: [[BitBoard; 64]; 64],
    squares_line: [[BitBoard; 64]; 64],
    knight_attacks: [BitBoard; 64],
    slider_attacks: [BitBoard; SLIDER_ATTACK_TABLE_SIZE],
    bishop_magics: [Magic; 64],
    rook_magics: [Magic; 64],
}

fn main() {
    let pawn_attacks: [[BitBoard; 64]; 2] = generate_pawn_attacks();
    let king_attacks: [BitBoard; 64] = generate_king_attacks();
    let squares_between: [[BitBoard; 64]; 64] = generate_squares_between();
    let squares_line: [[BitBoard; 64]; 64] = generate_squares_line();
    let knight_attacks: [BitBoard; 64] = generate_knight_attacks();
    let slider_attacks: [BitBoard; SLIDER_ATTACK_TABLE_SIZE] = generate_slider_attacks();

    let tables = Tables {
        pawn_attacks,
        king_attacks,
        squares_between,
        squares_line,
        knight_attacks,
        slider_attacks,
        bishop_magics: BISHOP_MAGICS,
        rook_magics: ROOK_MAGICS,
    };

    let mut file = File::create("tables.bin").unwrap();

    unsafe {
        let size = mem::size_of::<Tables>();
        let ptr = &tables as *const Tables as *const u8;
        let bytes = std::slice::from_raw_parts(ptr, size);
        file.write_all(bytes).unwrap();
    }
}

use std::{
    fmt::{self, Formatter},
    ops::BitOrAssign,
};

pub mod king_move;
pub mod knight_move;
pub mod magics;
pub mod pawn_move;
pub mod rays_between;
pub mod slider_move;
pub mod xray_line;
pub mod zobrist;

// STUB TYPES

/// Fixed shift fancy magic number
#[repr(C)]
pub struct Magic {
    pub magic: u64,
    pub mask: u64,
    pub offset: u64,
}

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
pub const NUM_FILES: usize = 8;
pub const NUM_CASTLING_CONFIGS: usize = 16;
pub const NUM_COLORS: usize = 2;
pub const NUM_PIECES: usize = 6;

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

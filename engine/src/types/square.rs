use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use crate::declare_per_type;
use crate::types::bitboard::BitBoard;
use crate::types::color::Color;

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

pub const NUM_SQUARES: usize = 64;

impl Square {
    pub fn from(rank: Rank, file: File) -> Square {
        Self::from_index(rank as u8 * 8 + file as u8)
    }

    pub const fn from_index(index: u8) -> Square {
        assert!(index < 64);
        unsafe { std::mem::transmute::<u8, Square>(index) }
    }

    #[inline]
    pub const fn to_index(&self) -> u8 {
        *self as u8
    }

    pub const fn forward(&self, color: Color) -> Square {
        let new_index = match color {
            Color::White => *self as i8 + 8,
            Color::Black => *self as i8 - 8,
        };
        assert!(new_index >= 0 && new_index <= 63);
        Square::from_index(new_index as u8)
    }

    pub const fn file(&self) -> File {
        unsafe { std::mem::transmute(*self as u8 % 8) }
    }

    pub const fn rank(&self) -> Rank {
        unsafe { std::mem::transmute(*self as u8 / 8) }
    }

    pub const fn mirror_vertically(&self) -> Square {
        Square::from_index((*self as u8) ^ 56)
    }
}

impl From<Square> for usize {
    fn from(value: Square) -> Self {
        value as usize
    }
}

declare_per_type!(PerSquare, Square, NUM_SQUARES);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (rank, file) = (self.to_index() / 8, self.to_index() % 8);
        write!(f, "{}{}", (b'a' + file) as char, (b'1' + rank) as char)
    }
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    R1, R2, R3, R4, R5, R6, R7, R8
}

impl Rank {
    pub fn mask(self) -> BitBoard {
        BitBoard::mask_rank(self)
    }
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum File {
    A, B, C, D, E, F, G, H
}

impl File {
    pub fn mask(self) -> BitBoard {
        BitBoard::mask_file(self)
    }
}

pub const NUM_RANKS: usize = 8;
pub const NUM_FILES: usize = 8;

impl From<Rank> for usize {
    fn from(value: Rank) -> Self {
        value as usize
    }
}

impl From<File> for usize {
    fn from(value: File) -> Self {
        value as usize
    }
}

#[derive(Debug, PartialEq)]
pub enum ParsePositionError {
    InvalidLength,
    InvalidRank,
    InvalidFile,
}

impl FromStr for Square {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [file, rank] => {
                let file = file.to_ascii_lowercase();
                if !(b'a'..=b'h').contains(&file) {
                    return Err(ParsePositionError::InvalidFile);
                }
                if !(b'1'..=b'8').contains(rank) {
                    return Err(ParsePositionError::InvalidRank);
                }
                let file_idx = file - b'a';
                let rank_idx = rank - b'1';
                Ok(Square::from_index(rank_idx * 8 + file_idx))
            }
            _ => Err(ParsePositionError::InvalidLength),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_square_parsing() {
        assert_eq!("e2".parse::<Square>(), Ok(Square::E2));
        assert_eq!("D4".parse::<Square>(), Ok(Square::D4));
        assert_eq!("a1".parse::<Square>(), Ok(Square::A1));
        assert_eq!("h8".parse::<Square>(), Ok(Square::H8));
    }
}

use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

use crate::bitboard::BitBoard;
use crate::color::Color;
use crate::square::Square::{A1, A2, A7, A8, H1, H2, H7, H8};

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
    pub fn to_file(&self) -> File {
        unsafe { std::mem::transmute::<u8, File>(*self as u8 % 8) }
    }
    pub fn to_rank(&self) -> Rank {
        unsafe { std::mem::transmute::<u8, Rank>(*self as u8 / 8) }
    }
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    R1, R2, R3, R4, R5, R6, R7, R8
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum File {
    A, B, C, D, E, F, G, H
}

impl Square {
    #[inline]
    pub const fn to_index(&self) -> u8 {
        *self as u8
    }

    pub const fn from_index(index: u8) -> Square {
        assert!(index < 64);
        unsafe { std::mem::transmute::<u8, Square>(index) }
    }

    pub const fn forward(&self, color: Color) -> Option<Square> {
        let new_index = match color {
            Color::White => self.to_index() as i8 + 8,
            Color::Black => self.to_index() as i8 - 8,
        };

        if new_index < 0 || new_index > 63 {
            None
        } else {
            Some(Square::from_index(new_index as u8))
        }
    }

    pub const fn on_promotion_rank(&self, color: Color) -> bool {
        match color {
            Color::White => self.to_index() >= A8.to_index() && self.to_index() <= H8.to_index(),
            Color::Black => self.to_index() >= A1.to_index() && self.to_index() <= H1.to_index(),
        }
    }

    pub const fn on_initial_pawn_rank(&self, color: Color) -> bool {
        match color {
            Color::White => self.to_index() >= A2.to_index() && self.to_index() <= H2.to_index(),
            Color::Black => self.to_index() >= A7.to_index() && self.to_index() <= H7.to_index(),
        }
    }

    pub const fn file_mask(&self) -> BitBoard {
        BitBoard(!BitBoard::NOT_A_FILE.0 << (*self as i32 & 7))
    }

    pub fn from(rank: Rank, file: File) -> Square {
        Self::from_index(rank as u8 * 8 + file as u8)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (rank, file) = (self.to_index() / 8, self.to_index() % 8);
        write!(f, "{}{}", (b'a' + file) as char, (b'1' + rank) as char)
    }
}

#[derive(Debug)]
pub enum ParsePositionError {
    InvalidLength,
    InvalidRank,
    InvalidFile,
}

impl FromStr for Square {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(ParsePositionError::InvalidLength);
        }
        let mut chars = s.chars();

        let file = (chars.next().unwrap().to_ascii_lowercase() as i8) - ('a' as i8);
        let rank = (chars.next().unwrap().to_ascii_lowercase() as i8) - ('1' as i8);

        if !(0..=7).contains(&file) {
            return Err(ParsePositionError::InvalidFile);
        }

        if !(0..=7).contains(&rank) {
            return Err(ParsePositionError::InvalidRank);
        }

        Ok(Square::from_index(rank as u8 * 8 + file as u8))
    }
}

#[cfg(test)]
mod test {
    use crate::square::Square;

    #[test]
    fn test_square_parsing() {
        assert_eq!("e2".parse::<Square>().unwrap(), Square::E2);
        assert_eq!("D4".parse::<Square>().unwrap(), Square::D4);
        assert_eq!("a1".parse::<Square>().unwrap(), Square::A1);
        assert_eq!("h8".parse::<Square>().unwrap(), Square::H8);
    }
}

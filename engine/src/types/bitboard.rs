use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

use crate::types::square::{File, Rank, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const fn contains(&self, square: Square) -> bool {
        (self.0 & (1 << square.to_index())) != 0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn from_square(square: Square) -> BitBoard {
        BitBoard(1 << square.to_index())
    }

    pub const fn count(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub const fn bit_scan(&self) -> Square {
        Square::from_index(self.0.trailing_zeros() as u8)
    }

    pub const fn shift(self, offset: i32) -> BitBoard {
        BitBoard(if offset >= 0 {
            self.0 << offset
        } else {
            self.0 >> -offset
        })
    }

    pub fn iter(&self) -> BitBoardIterator {
        BitBoardIterator(*self)
    }

    pub const fn mask_rank(rank: Rank) -> BitBoard {
        Self::ALL_RANKS[rank as usize]
    }

    pub const fn mask_file(file: File) -> BitBoard {
        Self::ALL_FILES[file as usize]
    }

    pub const EMPTY: BitBoard = BitBoard(0);

    pub const FULL: BitBoard = BitBoard(!0);

    pub const ALL_RANKS: [BitBoard; 8] = generate_all_ranks();
    pub const ALL_FILES: [BitBoard; 8] = generate_all_files();
}

const fn generate_all_ranks() -> [BitBoard; 8] {
    let mut result = [BitBoard(0); 8];

    let mut rank = 0;
    while rank < 8 {
        result[rank] = BitBoard(0xFF << (8 * rank));
        rank += 1;
    }
    result
}

const fn generate_all_files() -> [BitBoard; 8] {
    let mut result = [BitBoard(0); 8];

    const FILE_A: u64 = 0x01_01_01_01_01_01_01_01;
    let mut file = 0;
    while file < 8 {
        result[file] = BitBoard(FILE_A << file as i32);
        file += 1;
    }

    result
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
                write!(f, "{symbol} ")?;
            }
            writeln!(f)?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{file} ")?;
        }

        write!(f, "\n\nBitboard: {}", self.0)
    }
}

pub struct BitBoardIterator(BitBoard);

impl Iterator for BitBoardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0.is_empty() {
            None
        } else {
            let square = self.0.bit_scan();
            self.0 ^= square;
            Some(square)
        }
    }
}

macro_rules! impl_bitwise_op {
    ($struct_name:ident, $op_trait:ident, $op_assign_trait:ident, $op_fn:ident, $op_assign_fn:ident, $op_sym:tt, $op_assign_sym:tt) => {
        impl $op_trait for $struct_name {
            type Output = $struct_name;

            #[inline]
            fn $op_fn(self, other: $struct_name) -> $struct_name {
                $struct_name(self.0 $op_sym other.0)
            }
        }
        impl $op_assign_trait for $struct_name {
            #[inline]
            fn $op_assign_fn(&mut self, other: $struct_name) {
                self.0 $op_assign_sym other.0
            }
        }
    };
}

impl_bitwise_op!(BitBoard, BitAnd, BitAndAssign, bitand, bitand_assign, &, &=);
impl_bitwise_op!(BitBoard, BitOr, BitOrAssign, bitor, bitor_assign, |, |=);
impl_bitwise_op!(BitBoard, BitXor, BitXorAssign, bitxor, bitxor_assign, ^, ^=);

impl Shl<i32> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: i32) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<i32> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: i32) -> Self::Output {
        BitBoard(self.0 >> rhs)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(self.0.not())
    }
}

impl BitOrAssign<Square> for BitBoard {
    fn bitor_assign(&mut self, rhs: Square) {
        self.0 |= 1 << rhs as u64;
    }
}

impl BitXorAssign<Square> for BitBoard {
    fn bitxor_assign(&mut self, rhs: Square) {
        self.0 ^= 1 << rhs as u64;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_bit() {
        let mut bitboard = BitBoard(0);
        bitboard |= Square::H1;
        bitboard |= Square::E2;
        bitboard |= Square::H8;
        assert_eq!(bitboard, BitBoard(9223372036854780032));
    }

    #[test]
    fn test_get_bit() {
        let bitboard = BitBoard(9223372036854780032);
        assert!(bitboard.contains(Square::H1));
        assert!(bitboard.contains(Square::E2));
        assert!(bitboard.contains(Square::H8));
        assert!(!bitboard.contains(Square::F4))
    }

    #[test]
    fn test_display() {
        let bitboard = BitBoard(9223372036854780032);
        let expected = "
8   . . . . . . . X 
7   . . . . . . . . 
6   . . . . . . . . 
5   . . . . . . . . 
4   . . . . . . . . 
3   . . . . . . . . 
2   . . . . X . . . 
1   . . . . . . . X 

    a b c d e f g h 

Bitboard: 9223372036854780032";
        println!("{bitboard}");
        assert_eq!(bitboard.to_string(), expected);
    }

    #[test]
    fn test_all_files() {
        for file in 0..8 {
            let mut expected = BitBoard::EMPTY;
            for rank in 0..8 {
                let square = rank * 8 + file;
                expected |= Square::from_index(square);
            }
            println!("file {file}:\n{}", BitBoard::ALL_FILES[file as usize]);
            assert_eq!(expected, BitBoard::ALL_FILES[file as usize]);
        }
    }
}

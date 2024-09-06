use std::fmt;
use std::fmt::Formatter;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr,
};

use crate::square::Square;

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
        } else if offset >= -63 {
            self.0 >> -offset
        } else {
            0
        })
    }

    pub fn iter(&self) -> BitBoardIterator {
        BitBoardIterator(*self)
    }


    pub const EMPTY: BitBoard = BitBoard(0);

    pub const NOT_1ST_RANK: BitBoard = BitBoard(18446744073709551360);
    pub const NOT_8TH_RANK: BitBoard = BitBoard(72057594037927935);

    pub const NOT_A_FILE: BitBoard = BitBoard(18374403900871474942);
    pub const NOT_H_FILE: BitBoard = BitBoard(9187201950435737471);
    pub const NOT_AB_FILE: BitBoard = BitBoard(18229723555195321596);
    pub const NOT_GH_FILE: BitBoard = BitBoard(4557430888798830399);

    pub const ALL_RANKS: [BitBoard; 8] = generate_all_ranks();
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
        // regular
        impl $op_trait for $struct_name {
            type Output = $struct_name;

            #[inline]
            fn $op_fn(self, other: $struct_name) -> $struct_name {
                $struct_name(self.0 $op_sym other.0)
            }
        }

        impl $op_trait for &$struct_name {
            type Output = $struct_name;

            #[inline]
            fn $op_fn(self, other: &$struct_name) -> $struct_name {
                $struct_name(self.0 $op_sym other.0)
            }
        }

        impl $op_trait<$struct_name> for &$struct_name {
            type Output = $struct_name;

            #[inline]
            fn $op_fn(self, other: $struct_name) -> $struct_name {
                $struct_name(self.0 $op_sym other.0)
            }
        }

        impl $op_trait<&$struct_name> for $struct_name {
            type Output = $struct_name;

            #[inline]
            fn $op_fn(self, other: &$struct_name) -> $struct_name {
                $struct_name(self.0 $op_sym other.0)
            }
        }

        // assign

        impl $op_assign_trait for $struct_name {
            #[inline]
            fn $op_assign_fn(&mut self, other: $struct_name) {
                self.0 $op_assign_sym other.0
            }
        }

        impl $op_assign_trait<&$struct_name> for $struct_name {
            #[inline]
            fn $op_assign_fn(&mut self, other: &$struct_name)  {
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
    use crate::bitboard::BitBoard;
    use crate::square::Square;

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
    fn test_not_a_file() {
        let mut expected = BitBoard(0);
        let file = 0;
        for rank in 0..8 {
            let square = rank * 8 + file;
            expected |= Square::from_index(square);
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_A_FILE);
        assert_eq!(expected, BitBoard::NOT_A_FILE);
    }

    #[test]
    fn test_not_h_file() {
        let mut expected = BitBoard(0);
        let file = 7;
        for rank in 0..8 {
            let square = rank * 8 + file;
            expected |= Square::from_index(square);
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_H_FILE);
        assert_eq!(expected, BitBoard::NOT_H_FILE);
    }

    #[test]
    fn test_not_ab_file() {
        let mut expected = BitBoard(0);
        for file in 0..2 {
            for rank in 0..8 {
                let square = rank * 8 + file;
                expected |= Square::from_index(square);
            }
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_AB_FILE);
        assert_eq!(expected, BitBoard::NOT_AB_FILE);
    }

    #[test]
    fn test_not_gh_file() {
        let mut expected = BitBoard(0);
        for file in 6..8 {
            for rank in 0..8 {
                let square = rank * 8 + file;
                expected |= Square::from_index(square);
            }
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_GH_FILE);
        assert_eq!(expected, BitBoard::NOT_GH_FILE);
    }

    #[test]
    fn test_not_1st_rank() {
        let mut expected = BitBoard(0);
        let rank = 0;
        for file in 0..8 {
            let square = rank * 8 + file;
            expected |= Square::from_index(square);
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_1ST_RANK);
        assert_eq!(expected, BitBoard::NOT_1ST_RANK);
    }

    #[test]
    fn test_not_8th_rank() {
        let mut expected = BitBoard(0);
        let rank = 7;
        for file in 0..8 {
            let square = rank * 8 + file;
            expected |= Square::from_index(square);
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_8TH_RANK);
        assert_eq!(expected, BitBoard::NOT_8TH_RANK);
    }
}

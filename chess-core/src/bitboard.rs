use crate::square::Square;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Mul, Not, Shl, Shr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, square: Square) -> bool {
        (self.0 & (1 << square.to_index())) == (1 << square.to_index())
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub const fn from_square(square: Square) -> BitBoard {
        BitBoard(1 << square.to_index())
    }

    pub const fn popcnt(&self) -> u8 {
        self.0.count_ones() as u8
    }

    // TODO: come up with a better name
    /// Fetch first set square in bitboard
    pub const fn fetch_first_square(&self) -> Square {
        Square::from_index(self.0.trailing_zeros() as u8)
    }

    pub fn iter(&self) -> BitBoardIterator {
        BitBoardIterator(*self)
    }

    pub fn iter_masked(&self, mask: u64) -> BitBoardIteratorMasked {
        BitBoardIteratorMasked {
            index: 0,
            bitboard: *self,
            mask,
        }
    }

    pub const NOT_A_FILE: BitBoard = BitBoard(18374403900871474942);
    pub const NOT_H_FILE: BitBoard = BitBoard(9187201950435737471);
    pub const NOT_AB_FILE: BitBoard = BitBoard(18229723555195321596);
    pub const NOT_GH_FILE: BitBoard = BitBoard(4557430888798830399);
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?;
        for rank in (0..8).rev() {
            write!(f, "{}   ", rank + 1)?;
            for file in 0..8 {
                let square = rank * 8 + file;
                let value = (self.0 & (1 << square)) == (1 << square);
                let symbol = if value { 'X' } else { '.' };
                write!(f, "{} ", symbol)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{} ", file)?;
        }

        write!(f, "\n\nBitboard: {}", self.0)
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

impl Mul<u64> for BitBoard {
    type Output = BitBoard;

    fn mul(self, rhs: u64) -> Self::Output {
        BitBoard(self.0.wrapping_mul(rhs))
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

pub struct BitBoardIterator(BitBoard);

pub struct BitBoardIteratorMasked {
    index: u8,
    bitboard: BitBoard,
    mask: u64,
}

impl Iterator for BitBoardIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        if self.0.is_empty() {
            None
        } else {
            let square = self.0.fetch_first_square();
            self.0 ^= square;
            Some(square)
        }
    }
}

impl Iterator for BitBoardIteratorMasked {
    type Item = Square;

    fn next(&mut self) -> Option<Square> {
        while !self.bitboard.is_empty() {
            let square = self.bitboard.fetch_first_square();
            self.bitboard ^= square;

            let overlaps_mask = (self.mask & (1 << self.index)) != 0;
            self.index += 1;

            if overlaps_mask {
                return Some(square);
            }
        }
        None
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
        assert!(bitboard.get_bit(Square::H1));
        assert!(bitboard.get_bit(Square::E2));
        assert!(bitboard.get_bit(Square::H8));
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
}

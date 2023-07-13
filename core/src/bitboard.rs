use crate::square::Square;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, square: Square) -> bool {
        (self.0 & (1 << square.to_index())) == (1 << square.to_index())
    }

    // pub const fn with_bit_set(&self, square: Square) -> BitBoard {
    //     BitBoard(self.0 | (1 << square.to_index() as u64))
    // }

    pub const fn from_square(square: Square) -> BitBoard {
        BitBoard(1 << square.to_index())
    }

    pub const NOT_A_FILE: BitBoard = BitBoard(18374403900871474942);
    pub const NOT_H_FILE: BitBoard = BitBoard(9187201950435737471);
    pub const NOT_AB_FILE: BitBoard = BitBoard(18229723555195321596);
    pub const NOT_GH_FILE: BitBoard = BitBoard(4557430888798830399);
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?;
        for rank in 0..8 {
            write!(f, "{}   ", 8 - rank)?;
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

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::square::Square;

    #[test]
    fn test_set_bit() {
        let mut bitboard = BitBoard(0);
        bitboard |= BitBoard::from_square(Square::H1);
        bitboard |= BitBoard::from_square(Square::E2);
        bitboard |= BitBoard::from_square(Square::H8);
        assert_eq!(bitboard, BitBoard(9227875636482146432));
    }

    #[test]
    fn test_get_bit() {
        let bitboard = BitBoard(9227875636482146432);
        assert!(bitboard.get_bit(Square::H1));
        assert!(bitboard.get_bit(Square::E2));
        assert!(bitboard.get_bit(Square::H8));
    }

    #[test]
    fn test_display() {
        let bitboard = BitBoard(9227875636482146432);
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

Bitboard: 9227875636482146432";
        assert_eq!(bitboard.to_string(), expected);
    }

    #[test]
    fn test_not_a_file() {
        let mut expected = BitBoard(0);
        let file = 0;
        for rank in 0..8 {
            let square = rank * 8 + file;
            expected |= BitBoard::from_square(Square::from_index(square));
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
            expected |= BitBoard::from_square(Square::from_index(square));
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
                expected |= BitBoard::from_square(Square::from_index(square));
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
                expected |= BitBoard::from_square(Square::from_index(square));
            }
        }
        expected = !expected;

        println!("{}", BitBoard::NOT_GH_FILE);
        assert_eq!(expected, BitBoard::NOT_GH_FILE);
    }
}

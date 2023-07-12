use crate::square::Square;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn get_bit(&self, square: Square) -> bool {
        (self.0 & (1 << square.to_index())) == (1 << square.to_index())
    }

    pub fn set_bit(&mut self, square: Square) {
        self.0 |= 1 << square.to_index() as u64;
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

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::square::Square;

    #[test]
    fn test_set_bit() {
        let mut bitboard = BitBoard(0);
        bitboard.set_bit(Square::H1);
        bitboard.set_bit(Square::E2);
        bitboard.set_bit(Square::H8);
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
}

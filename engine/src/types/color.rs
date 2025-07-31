use std::ops::Not;

use crate::{declare_per_type, types::square::Rank};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;

    #[inline]
    fn not(self) -> Self::Output {
        unsafe { std::mem::transmute::<u8, Color>(self as u8 ^ 1) }
    }
}

impl Color {
    pub const fn backrank(self) -> Rank {
        unsafe { self.unchecked_relative_rank(0) }
    }

    pub const fn initial_pawn_rank(self) -> Rank {
        unsafe { self.unchecked_relative_rank(1) }
    }

    pub const fn double_pawn_push_rank(self) -> Rank {
        unsafe { self.unchecked_relative_rank(3) }
    }

    pub const fn promotion_rank(self) -> Rank {
        unsafe { self.unchecked_relative_rank(7) }
    }

    /// # Safety
    ///
    /// `index` must be between 0 to 7
    pub const unsafe fn unchecked_relative_rank(self, index: u8) -> Rank {
        let index = match self {
            Color::White => index,
            Color::Black => 7 - index,
        };
        unsafe { std::mem::transmute(index) }
    }
}

pub const NUM_COLORS: usize = 2;
pub const ALL_COLORS: [Color; 2] = [Color::White, Color::Black];

impl From<Color> for usize {
    fn from(value: Color) -> Self {
        value as usize
    }
}

declare_per_type!(PerColor, Color, NUM_COLORS);

impl<T: PartialEq> PerColor<T> {
    pub fn contains(&self, x: &T) -> bool {
        self.inner.contains(x)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_not() {
        assert_eq!(!Color::Black, Color::White);
        assert_eq!(!Color::White, Color::Black);
    }

    #[test]
    fn test_special_ranks() {
        assert_eq!(Color::White.backrank(), Rank::R1);
        assert_eq!(Color::Black.backrank(), Rank::R8);

        assert_eq!(Color::White.initial_pawn_rank(), Rank::R2);
        assert_eq!(Color::Black.initial_pawn_rank(), Rank::R7);

        assert_eq!(Color::White.double_pawn_push_rank(), Rank::R4);
        assert_eq!(Color::Black.double_pawn_push_rank(), Rank::R5);

        assert_eq!(Color::White.promotion_rank(), Rank::R8);
        assert_eq!(Color::Black.promotion_rank(), Rank::R1);
    }
}

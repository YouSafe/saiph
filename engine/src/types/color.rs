use crate::types::square::Rank;
use std::ops::Not;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        unsafe { std::mem::transmute::<u8, Color>(self as u8 ^ 1) }
    }
}

impl Color {
    pub const fn backrank(&self) -> Rank {
        unsafe { self.unchecked_relative_rank(0) }
    }

    pub const fn initial_pawn_rank(&self) -> Rank {
        unsafe { self.unchecked_relative_rank(1) }
    }

    pub const fn double_pawn_push_rank(&self) -> Rank {
        unsafe { self.unchecked_relative_rank(3) }
    }

    /// # Safety
    ///
    /// `index` must be between 0 to 7
    pub const unsafe fn unchecked_relative_rank(&self, index: u8) -> Rank {
        let index = (*self as u8) * (7 - 2 * index) + index;
        unsafe { std::mem::transmute(index) }
    }
}

pub const NUM_COLORS: usize = 2;
pub const ALL_COLORS: [Color; 2] = [Color::White, Color::Black];

#[cfg(test)]
mod test {
    use crate::types::color::Color;
    use crate::types::square::Rank;

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
    }
}

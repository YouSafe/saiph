use std::ops::Not;

use crate::square::Rank;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        // TODO: benchmark if this is really faster than a regular match expression
        unsafe { std::mem::transmute::<u8, Color>(self as u8 ^ 1) }
    }
}

impl Color {
    pub const fn backrank(&self) -> Rank {
        [Rank::R1, Rank::R8][*self as usize]
    }
}

pub const NUM_COLORS: usize = 2;

#[cfg(test)]
mod test {
    use crate::color::Color;

    #[test]
    fn test_not() {
        assert_eq!(!Color::Black, Color::White);
        assert_eq!(!Color::White, Color::Black);
    }
}

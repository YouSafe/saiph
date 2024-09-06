use crate::bitboard::BitBoard;
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
        unsafe { std::mem::transmute::<u8, Color>(self as u8 ^ 1) }
    }
}

impl Color {
    pub const fn backrank(&self) -> Rank {
        [Rank::R1, Rank::R8][*self as usize]
    }

    pub const fn forward_shift(&self) -> i32 {
        [8, -8][*self as usize]
    }

    pub const fn initial_pawn_rank(&self) -> BitBoard {
        [BitBoard::ALL_RANKS[1], BitBoard::ALL_RANKS[6]][*self as usize]
    }

    pub const fn double_pawn_push_rank(&self) -> BitBoard {
        [BitBoard::ALL_RANKS[3], BitBoard::ALL_RANKS[4]][*self as usize]
    }

    pub const fn bankrank_rank(&self) -> BitBoard {
        [BitBoard::ALL_RANKS[0], BitBoard::ALL_RANKS[7]][*self as usize]
    }
}

pub const NUM_COLORS: usize = 2;
pub const ALL_COLORS: [Color; 2] = [Color::White, Color::Black];

#[cfg(test)]
mod test {
    use crate::color::Color;

    #[test]
    fn test_not() {
        assert_eq!(!Color::Black, Color::White);
        assert_eq!(!Color::White, Color::Black);
    }
}

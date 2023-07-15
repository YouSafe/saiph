use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const WHITE_KING_SIDE: Self = Self(1);
    pub const WHITE_QUEEN_SIDE: Self = Self(2);
    pub const BLACK_KING_SIDE: Self = Self(4);
    pub const BLACK_QUEEN_SIDE: Self = Self(8);

    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::WHITE_KING_SIDE
            | CastlingRights::WHITE_QUEEN_SIDE
            | CastlingRights::BLACK_KING_SIDE
            | CastlingRights::BLACK_QUEEN_SIDE
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        const RIGHTS: [(CastlingRights, char); 4] = [
            (CastlingRights::WHITE_KING_SIDE, 'K'),
            (CastlingRights::WHITE_QUEEN_SIDE, 'Q'),
            (CastlingRights::BLACK_KING_SIDE, 'k'),
            (CastlingRights::BLACK_QUEEN_SIDE, 'q'),
        ];
        for (right, symbol) in RIGHTS {
            if self.contains(right) {
                write!(f, "{}", symbol)?;
            } else {
                write!(f, "-")?
            }
        }
        Ok(())
    }
}

impl BitAnd for CastlingRights {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl BitAndAssign for CastlingRights {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0);
    }
}
impl BitOr for CastlingRights {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl BitOrAssign for CastlingRights {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0);
    }
}
impl BitXor for CastlingRights {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl BitXorAssign for CastlingRights {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 ^ rhs.0);
    }
}
impl Sub for CastlingRights {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 & !rhs.0)
    }
}
impl SubAssign for CastlingRights {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & !rhs.0);
    }
}

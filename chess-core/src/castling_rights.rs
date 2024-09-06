use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Sub, SubAssign};
use std::str::FromStr;

use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights(u8);

pub static UPDATE_CASTLING_RIGHT_TABLE: [CastlingRights; 64] =
    generate_update_castling_right_table();

const fn generate_update_castling_right_table() -> [CastlingRights; 64] {
    // start out with every square keeping all the rights
    // those castling rights represent upperbounds for the rights

    // updating castling rights works by taking the old castling rights and union them with the
    // entry of this table for the `from` and `to` square
    let mut result = [CastlingRights::all(); 64];

    // remove colors castling rights if the corresponding king moves into or away from their square
    result[Square::E1 as usize] = CastlingRights::all().subtract(CastlingRights::WHITE_BOTH_SIDES);
    result[Square::E8 as usize] = CastlingRights::all().subtract(CastlingRights::BLACK_BOTH_SIDES);

    // remove castling rights if a piece moves out or into the rook square
    result[Square::A1 as usize] = CastlingRights::all().subtract(CastlingRights::WHITE_QUEEN_SIDE);
    result[Square::H1 as usize] = CastlingRights::all().subtract(CastlingRights::WHITE_KING_SIDE);

    result[Square::A8 as usize] = CastlingRights::all().subtract(CastlingRights::BLACK_QUEEN_SIDE);
    result[Square::H8 as usize] = CastlingRights::all().subtract(CastlingRights::BLACK_KING_SIDE);

    result
}

impl CastlingRights {
    pub const WHITE_KING_SIDE: Self = Self(1);
    pub const WHITE_QUEEN_SIDE: Self = Self(2);
    pub const BLACK_KING_SIDE: Self = Self(4);
    pub const BLACK_QUEEN_SIDE: Self = Self(8);

    pub const WHITE_BOTH_SIDES: Self = Self(3);
    pub const BLACK_BOTH_SIDES: Self = Self(12);

    pub const fn all() -> CastlingRights {
        Self (15)
    }

    pub const fn empty() -> CastlingRights {
        Self(0)
    }

    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub const fn subtract(&self, other: Self) -> CastlingRights {
        Self(self.0 & !other.0)
    }

    pub const fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::all()
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

impl FromStr for CastlingRights {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const RIGHTS: [(CastlingRights, char); 4] = [
            (CastlingRights::WHITE_KING_SIDE, 'K'),
            (CastlingRights::WHITE_QUEEN_SIDE, 'Q'),
            (CastlingRights::BLACK_KING_SIDE, 'k'),
            (CastlingRights::BLACK_QUEEN_SIDE, 'q'),
        ];

        let mut castling_rights = CastlingRights::empty();

        for (right, symbol) in RIGHTS {
            if s.contains(symbol) {
                castling_rights |= right;
            }
        }
        Ok(castling_rights)
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
        self.subtract(rhs)
    }
}

impl SubAssign for CastlingRights {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.subtract(rhs);
    }
}

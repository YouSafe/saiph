use std::fmt;
use std::fmt::Formatter;

use crate::types::color::Color;
use crate::types::promotion::Promotion;
use crate::types::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal = 0b0000,
    DoublePawnPush = 0b0001,
    Castling = 0b0010,

    Capture = 0b0100,
    EnPassant = 0b0101,

    KnightPromotion = 0b1000,
    BishopPromotion = 0b1001,
    RookPromotion = 0b1010,
    QueenPromotion = 0b1011,

    KnightPromotionCapture = 0b1100,
    BishopPromotionCapture = 0b1101,
    RookPromotionCapture = 0b1110,
    QueenPromotionCapture = 0b1111,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move(u16);

impl Move {
    pub const NULL: Move = Move(0);

    pub const fn new(from: Square, to: Square, flag: MoveFlag) -> Self {
        Move(((flag as u16) << 12) | ((to as u16) << 6) | (from as u16))
    }

    pub const fn from(&self) -> Square {
        Square::from_index((self.0 & 0x3f) as u8)
    }

    pub const fn to(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0x3f) as u8)
    }

    pub const fn flag(&self) -> MoveFlag {
        unsafe { std::mem::transmute((self.0 >> 12) as u8) }
    }

    pub const fn promotion(&self) -> Option<Promotion> {
        match self.flag() {
            MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Some(Promotion::Knight),
            MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Some(Promotion::Bishop),
            MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Some(Promotion::Rook),
            MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => Some(Promotion::Queen),
            _ => None,
        }
    }

    pub const fn is_capture(&self) -> bool {
        (self.0 >> 14) & 1 != 0
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(promotion) = self.promotion() {
            write!(
                f,
                "{}{}{}",
                self.from(),
                self.to(),
                promotion.as_piece().to_ascii(Color::Black)
            )
        } else {
            write!(f, "{}{}", self.from(), self.to())
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Move")
            .field("from", &self.from())
            .field("to", &self.to())
            .field("flag", &self.flag())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_packed_move() {
        let m = Move::new(Square::A5, Square::B6, MoveFlag::Capture);
        assert_eq!(m.0, 0b0100_101001_100000);
        assert_eq!(m.from(), Square::A5);
        assert_eq!(m.to(), Square::B6);
        assert_eq!(m.flag(), MoveFlag::Capture)
    }

    #[test]
    fn test_promotion_flags() {
        let promotion_cases = [
            (MoveFlag::BishopPromotion, Promotion::Bishop, false),
            (MoveFlag::KnightPromotion, Promotion::Knight, false),
            (MoveFlag::RookPromotion, Promotion::Rook, false),
            (MoveFlag::QueenPromotion, Promotion::Queen, false),
            (MoveFlag::BishopPromotionCapture, Promotion::Bishop, true),
            (MoveFlag::KnightPromotionCapture, Promotion::Knight, true),
            (MoveFlag::RookPromotionCapture, Promotion::Rook, true),
            (MoveFlag::QueenPromotionCapture, Promotion::Queen, true),
        ];

        for (flag, expected_promotion, is_capture) in promotion_cases {
            let m = Move::new(
                Square::A7,
                if is_capture { Square::B8 } else { Square::A8 },
                flag,
            );
            assert_eq!(m.promotion(), Some(expected_promotion));
            if is_capture {
                assert!(m.is_capture());
            }
        }
    }
}

use std::fmt;
use std::fmt::Formatter;

use crate::color::Color;
use crate::piece::Piece;
use crate::promotion::Promotion;
use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal,
    DoublePawnPush,
    Checking,
    Capture,
    EnPassant,
    Castling,
}

// TODO: pack data into a single u32 or even better: u16
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
    pub piece: Piece,
    pub flags: MoveFlag,
}

impl Move {
    pub fn source(&self) -> Square {
        self.from
    }

    pub fn destination(&self) -> Square {
        self.to
    }
    
    pub fn is_capture(&self) -> bool {
        self.flags == MoveFlag::Capture
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(promotion) = self.promotion {
            write!(
                f,
                "{}{}{}",
                self.from,
                self.to,
                promotion.as_piece().to_ascii(Color::Black)
            )
        } else {
            write!(f, "{}{}", self.from, self.to)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::chess_move::Move;

    #[test]
    fn test() {
        eprintln!("move size in bytes: {}", std::mem::size_of::<Move>());
    }
}

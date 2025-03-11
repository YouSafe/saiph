use std::fmt::Debug;

use self::PieceType::*;
use crate::color::Color;

use crate::declare_per_type;

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

pub const NUM_PIECES: usize = 6;
pub const ALL_PIECES: [PieceType; 6] = [Pawn, Knight, Bishop, Rook, Queen, King];

impl PieceType {
    pub fn to_piece(self, color: Color) -> Piece {
        Piece::new(self, color)
    }
}

impl From<PieceType> for usize {
    fn from(value: PieceType) -> Self {
        value as usize
    }
}

declare_per_type!(PerPieceType, PieceType, NUM_PIECES);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Piece {
    WhitePawn   = pack(Pawn, Color::White),
    WhiteKnight = pack(Knight, Color::White),
    WhiteBishop = pack(Bishop, Color::White),
    WhiteRook   = pack(Rook, Color::White),
    WhiteQueen  = pack(Queen, Color::White),
    WhiteKing   = pack(King, Color::White),
    BlackPawn   = pack(Pawn, Color::Black),
    BlackKnight = pack(Knight, Color::Black),
    BlackBishop = pack(Bishop, Color::Black),
    BlackRook   = pack(Rook, Color::Black),
    BlackQueen  = pack(Queen, Color::Black),
    BlackKing   = pack(King, Color::Black),
}

const fn pack(ty: PieceType, color: Color) -> u8 {
    ((color as u8) << 3) | ty as u8
}

impl Piece {
    pub const fn new(ty: PieceType, color: Color) -> Self {
        unsafe { std::mem::transmute(pack(ty, color)) }
    }

    pub const fn color(&self) -> Color {
        unsafe { std::mem::transmute(*self as u8 >> 3 & 1) }
    }

    pub const fn ty(&self) -> PieceType {
        unsafe { std::mem::transmute(*self as u8 & 7) }
    }

    pub fn from_algebraic(letter: char) -> Option<Piece> {
        match letter {
            'p' => Some(Piece::new(Pawn, Color::Black)),
            'P' => Some(Piece::new(Pawn, Color::White)),
            'b' => Some(Piece::new(Bishop, Color::Black)),
            'B' => Some(Piece::new(Bishop, Color::White)),
            'n' => Some(Piece::new(Knight, Color::Black)),
            'N' => Some(Piece::new(Knight, Color::White)),
            'r' => Some(Piece::new(Rook, Color::Black)),
            'R' => Some(Piece::new(Rook, Color::White)),
            'q' => Some(Piece::new(Queen, Color::Black)),
            'Q' => Some(Piece::new(Queen, Color::White)),
            'k' => Some(Piece::new(King, Color::Black)),
            'K' => Some(Piece::new(King, Color::White)),
            _ => None,
        }
    }

    pub fn to_unicode(&self) -> char {
        use crate::color::Color::{Black, White};

        match (self.color(), self.ty()) {
            (White, Pawn) => '♙',
            (White, Knight) => '♘',
            (White, Bishop) => '♗',
            (White, Rook) => '♖',
            (White, Queen) => '♕',
            (White, King) => '♔',

            (Black, Pawn) => '♟',
            (Black, Knight) => '♞',
            (Black, Bishop) => '♝',
            (Black, Rook) => '♜',
            (Black, Queen) => '♛',
            (Black, King) => '♚',
        }
    }

    pub fn to_ascii(&self) -> char {
        use crate::color::Color::{Black, White};
        match (self.color(), self.ty()) {
            (White, Pawn) => 'P',
            (White, Knight) => 'N',
            (White, Bishop) => 'B',
            (White, Rook) => 'R',
            (White, Queen) => 'Q',
            (White, King) => 'K',

            (Black, Pawn) => 'p',
            (Black, Knight) => 'n',
            (Black, Bishop) => 'b',
            (Black, Rook) => 'r',
            (Black, Queen) => 'q',
            (Black, King) => 'k',
        }
    }
}

impl Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Piece")
            .field("ty", &self.ty())
            .field("color", &self.color())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use crate::{color::ALL_COLORS, piece::ALL_PIECES};

    use super::Piece;

    #[test]
    fn constuction() {
        use crate::color::Color::*;
        use crate::piece::Piece::*;
        use crate::piece::PieceType::*;

        let lookup = [
            (WhitePawn, (Pawn, White)),
            (WhiteKnight, (Knight, White)),
            (WhiteBishop, (Bishop, White)),
            (WhiteRook, (Rook, White)),
            (WhiteQueen, (Queen, White)),
            (WhiteKing, (King, White)),
            (BlackPawn, (Pawn, Black)),
            (BlackKnight, (Knight, Black)),
            (BlackBishop, (Bishop, Black)),
            (BlackRook, (Rook, Black)),
            (BlackQueen, (Queen, Black)),
            (BlackKing, (King, Black)),
        ];

        for ty in ALL_PIECES {
            for color in ALL_COLORS {
                let piece = Piece::new(ty, color);
                assert_eq!(piece.color(), color);
                assert_eq!(piece.ty(), ty);
            }
        }

        for (piece, (ty, color)) in lookup {
            assert_eq!(piece.color(), color);
            assert_eq!(piece.ty(), ty);
        }
    }
}

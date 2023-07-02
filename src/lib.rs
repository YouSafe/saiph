use std::str::FromStr;

pub mod engine_uci;
mod player;
mod board;
mod move_generator;
mod fen_parser;

#[derive(Debug, PartialEq)]
struct Position {
    rank: u8,
    file: u8
}

struct ParsePositionErr(String);

impl FromStr for Position {
    type Err = ParsePositionErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(ParsePositionErr(s.to_owned()));
        }

        let file = (s.chars().nth(0).unwrap() as u8) - ('a' as u8) + 1;
        let rank = (s.chars().nth(1).unwrap() as u8) - ('1' as u8) + 1;

        if rank > 7 || file > 7 {
            return Err(ParsePositionErr(s.to_owned()));
        }

        Ok(Position { file, rank })
    }
}

#[derive(Debug, PartialEq)]
pub struct CastlingRights {
    can_queen_side: bool,
    can_king_side: bool,
}

#[derive(Debug, PartialEq)]
enum Promotion {
    Queen,
    Bishop,
    Rook,
    Knight,
}

// TODO: pack move data tighter
#[derive(Debug)]
struct Move {
    from: Position,
    to: Position,
    promotion: Option<Promotion>
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Color {
    White,
    Black,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}
use crate::types::chess_move::Move;
use crate::types::promotion::Promotion;
use crate::types::promotion::Promotion::{Bishop, Knight, Queen, Rook};
use crate::types::square::Square;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct UCIMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
}

impl PartialEq<&Move> for UCIMove {
    fn eq(&self, other: &&Move) -> bool {
        self.from == other.from() && self.to == other.to() && self.promotion == other.promotion()
    }
}

#[derive(Debug)]
pub struct UCIMoveParseError;

impl FromStr for UCIMove {
    type Err = UCIMoveParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 && s.len() != 5 {
            return Err(UCIMoveParseError);
        }
        let from = s[0..2].parse::<Square>().map_err(|_| UCIMoveParseError)?;

        let to = s[2..4].parse::<Square>().map_err(|_| UCIMoveParseError)?;

        let promotion = if s.len() == 5 {
            match &s[4..5] {
                "q" => Some(Queen),
                "r" => Some(Rook),
                "b" => Some(Bishop),
                "n" => Some(Knight),
                _ => return Err(UCIMoveParseError),
            }
        } else {
            None
        };
        Ok(UCIMove {
            from,
            to,
            promotion,
        })
    }
}

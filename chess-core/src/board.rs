use crate::bitboard::BitBoard;
use crate::castling_rights::CastlingRights;
use crate::color::{Color, NUM_COLORS};
use crate::piece::{Piece, ALL_PIECES, NUM_PIECES};
use crate::square::Square;
use std::fmt;
use std::fmt::Formatter;

pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    occupancies: [BitBoard; NUM_COLORS],
    combined: BitBoard,
    side_to_move: Color,
    en_passant_square: Option<Square>,
    castling_rights: CastlingRights,
}

impl Default for Board {
    fn default() -> Self {
        let board = Board {
            pieces: [BitBoard(0); NUM_PIECES],
            occupancies: [BitBoard(0); NUM_COLORS],
            combined: BitBoard(0),
            side_to_move: Color::White,
            en_passant_square: None,
            castling_rights: Default::default(),
        };

        board
    }
}

impl Board {
    pub fn piece_on_square(&self, square: Square) -> Option<Piece> {
        let bitboard = BitBoard::from_square(square);
        if (self.combined & bitboard) == BitBoard(0) {
            None
        } else {
            for piece in ALL_PIECES {
                if (self.pieces[piece as usize] & bitboard) != BitBoard(0) {
                    return Some(piece);
                }
            }
            None
        }
    }

    pub fn color_on_square(&self, square: Square) -> Option<Color> {
        let bitboard = BitBoard::from_square(square);
        if (self.occupancies[Color::White as usize] & bitboard) != BitBoard(0) {
            Some(Color::White)
        } else if (self.occupancies[Color::Black as usize] & bitboard) != BitBoard(0) {
            Some(Color::Black)
        } else {
            None
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\n")?;
        for rank in (0..8).rev() {
            write!(f, "{}   ", rank + 1)?;
            for file in 0..8 {
                let square = Square::from_index(rank * 8 + file);
                let symbol = if let Some(piece) = self.piece_on_square(square) {
                    let color = self
                        .color_on_square(square)
                        .expect("piece must have a color");
                    piece.to_ascii(color)
                } else {
                    '.'
                };
                write!(f, "{} ", symbol)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{} ", file)?;
        }

        writeln!(f, "\n")?;
        writeln!(f, "En passant square:\t{:?}", self.en_passant_square)?;
        writeln!(f, "Side to move:\t\t{:?}", self.side_to_move)?;
        writeln!(f, "Castling rights:\t{}", self.castling_rights)?;
        writeln!(f, "\nFEN: WIP")
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;

    #[test]
    fn test_display() {
        let board = Board::default();
        println!("{}", board);
    }
}

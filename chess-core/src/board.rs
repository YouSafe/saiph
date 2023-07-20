use crate::bitboard::BitBoard;
use crate::castling_rights::CastlingRights;
use crate::chess_move::Move;
use crate::chess_move::MoveFlag::{Capture, Castling, DoublePawnPush, EnPassant};
use crate::color::{Color, NUM_COLORS};
use crate::movgen::calculate_pinned_checkers_pinners;
use crate::piece::{Piece, ALL_PIECES, NUM_PIECES};
use crate::square::{File, Square};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy)]
pub struct Board {
    pieces: [BitBoard; NUM_PIECES],
    occupancies: [BitBoard; NUM_COLORS],
    combined: BitBoard,
    side_to_move: Color,
    en_passant_target: Option<Square>,
    castling_rights: CastlingRights,
    pinned: BitBoard,
    checkers: BitBoard,
    pinners: BitBoard,
}

impl Default for Board {
    fn default() -> Self {
        Board::STARTING_POS_FEN.parse().unwrap()
    }
}

impl Board {
    pub const STARTING_POS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const KILLER_POS_FEN: &'static str =
        "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";

    pub const EMPTY: &'static str = "8/8/8/8/8/8/8/8 w - - 0 1";

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

    pub fn pieces(&self, piece: Piece) -> &BitBoard {
        &self.pieces[piece as usize]
    }

    pub fn occupancies(&self, color: Color) -> &BitBoard {
        &self.occupancies[color as usize]
    }

    pub fn combined(&self) -> &BitBoard {
        &self.combined
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn checkers(&self) -> BitBoard {
        self.checkers
    }

    pub fn pinned(&self) -> BitBoard {
        self.pinned
    }

    pub fn pinners(&self) -> BitBoard {
        self.pinners
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    pub fn en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    pub fn make_move(&self, mov: Move) -> Board {
        // read from old board
        // write to new board
        // copy
        let mut result = *self;

        // remove piece from from
        result.pieces[mov.piece as usize] ^= mov.from;
        result.occupancies[self.side_to_move as usize] ^= mov.from;
        result.combined ^= mov.from;

        // set piece in to
        result.pieces[mov.piece as usize] |= mov.to;
        result.occupancies[self.side_to_move as usize] |= mov.to;
        result.combined |= mov.to;

        const ROOK_TO_CASTLING_RIGHT: [[CastlingRights; 2]; NUM_COLORS] = [
            [
                CastlingRights::WHITE_QUEEN_SIDE,
                CastlingRights::WHITE_KING_SIDE,
            ],
            [
                CastlingRights::BLACK_QUEEN_SIDE,
                CastlingRights::BLACK_KING_SIDE,
            ],
        ];

        if mov.flags == Capture {
            // replace opponents piece with your own
            let target_piece = self
                .piece_on_square(mov.to)
                .expect("captures require a piece on the target square");

            if target_piece != mov.piece {
                result.pieces[target_piece as usize] ^= mov.to;
                result.occupancies[!self.side_to_move as usize] ^= mov.to;
                // combined is unchanged here
            }

            // remove castling right for that side
            if target_piece == Piece::Rook {
                let target_file = mov.to.to_file();
                let right =
                    ROOK_TO_CASTLING_RIGHT[!self.side_to_move as usize][target_file as usize / 4];
                result.castling_rights -= right;
            }
        }

        if let Some(promotion) = mov.promotion {
            // remove old piece type
            result.pieces[mov.piece as usize] ^= mov.to;
            // add to new piece type
            result.pieces[promotion.as_piece() as usize] |= mov.to;
        }

        if mov.flags == DoublePawnPush {
            // update en_passant_target when double pushing
            result.en_passant_target = Some(mov.to.forward(!self.side_to_move).unwrap());
        }

        if mov.flags == EnPassant {
            let capture_piece = mov.to.forward(!self.side_to_move).unwrap();
            result.pieces[Piece::Pawn as usize] ^= capture_piece;
            result.occupancies[!self.side_to_move as usize] ^= capture_piece;
            result.combined ^= capture_piece;
        }

        // en passant is cleared after doing any move
        result.en_passant_target = None;

        const CASTLE_CONFIG: [(File, File); 2] = [(File::A, File::D), (File::H, File::F)];

        if mov.flags == Castling {
            let backrank = result.side_to_move.backrank();
            let target_file = mov.to.to_file();
            let (rook_start_file, rook_end_file) = CASTLE_CONFIG[target_file as usize / 4];
            let (rook_start_square, rook_end_square) = (
                Square::from(backrank, rook_start_file),
                Square::from(backrank, rook_end_file),
            );

            // remove piece from from
            result.pieces[Piece::Rook as usize] ^= rook_start_square;
            result.occupancies[self.side_to_move as usize] ^= rook_start_square;
            result.combined ^= rook_start_square;

            // set piece in to
            result.pieces[Piece::Rook as usize] |= rook_end_square;
            result.occupancies[self.side_to_move as usize] |= rook_end_square;
            result.combined |= rook_end_square;
        }

        // update castling rights
        if mov.piece == Piece::Rook {
            // rook moved
            let target_file = mov.from.to_file();
            let right =
                ROOK_TO_CASTLING_RIGHT[self.side_to_move as usize][target_file as usize / 4];
            result.castling_rights -= right;
        } else if mov.piece == Piece::King {
            // remove castling rights for side if king moved
            result.castling_rights -= match result.side_to_move {
                Color::White => CastlingRights::WHITE_BOTH_SIDES,
                Color::Black => CastlingRights::BLACK_BOTH_SIDES,
            }
        }

        // TODO: update incrementally instead
        let (pinned, checkers, pinners) = calculate_pinned_checkers_pinners(&result);
        // update pinned, checkers
        result.pinned = pinned;
        result.checkers = checkers;
        result.pinners = pinners;

        // update side
        result.side_to_move = !result.side_to_move;

        result
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
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
            writeln!(f)?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{} ", file)?;
        }

        writeln!(f, "\n")?;
        writeln!(f, "En passant square:\t{:?}", self.en_passant_target)?;
        writeln!(f, "Side to move:\t\t{:?}", self.side_to_move)?;
        writeln!(f, "Castling rights:\t{}", self.castling_rights)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParseFenError {
    PartsMissing,
    BadPlacement,
    NoSuchSide,
    BadCastlingRights,
    BadFullMoveNumber,
    BadHalfMoveClock,
    BadEnPassant,
    TooManyPiecesInRank,
    InvalidPiece,
}

impl FromStr for Board {
    type Err = ParseFenError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err(ParseFenError::PartsMissing);
        }

        let (
            placement,
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
        ) = (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);

        let ranks_str: Vec<&str> = placement.split('/').collect();
        if ranks_str.len() != 8 {
            return Err(ParseFenError::BadPlacement);
        }

        let mut pieces = [BitBoard(0); NUM_PIECES];
        let mut occupancies = [BitBoard(0); NUM_COLORS];
        let mut combined = BitBoard(0);

        // reverse iterator to start with rank 1
        for (rank, rank_pieces) in (0u8..).zip(ranks_str.iter().rev()) {
            let mut file: u8 = 0;
            for piece_char in rank_pieces.chars() {
                match piece_char.to_digit(10) {
                    // blanks
                    Some(n) => {
                        file += n as u8;
                        if file > 8 {
                            return Err(ParseFenError::TooManyPiecesInRank);
                        }
                    }
                    // piece
                    None => {
                        let color = if piece_char.is_uppercase() {
                            Color::White
                        } else {
                            Color::Black
                        };

                        let piece = match piece_char.to_ascii_lowercase() {
                            'p' => Piece::Pawn,
                            'n' => Piece::Knight,
                            'b' => Piece::Bishop,
                            'r' => Piece::Rook,
                            'q' => Piece::Queen,
                            'k' => Piece::King,
                            _ => return Err(ParseFenError::InvalidPiece),
                        };

                        let square = Square::from_index(rank * 8 + file);

                        // place piece
                        pieces[piece as usize] |= square;
                        occupancies[color as usize] |= square;
                        combined |= square;

                        file += 1;
                    }
                }
            }
        }

        let side_to_move = match side_to_move {
            "w" => Ok(Color::White),
            "b" => Ok(Color::Black),
            _ => Err(ParseFenError::NoSuchSide),
        }?;

        let castling_rights = castling_rights
            .parse::<CastlingRights>()
            .map_err(|_| ParseFenError::BadCastlingRights)?;

        let en_passant_target = match en_passant_target {
            "-" => None,
            target => Some(
                target
                    .parse::<Square>()
                    .map_err(|_| ParseFenError::BadEnPassant)?,
            ),
        };

        let _fullmove_number = fullmove_number
            .parse::<u64>()
            .map_err(|_| ParseFenError::BadFullMoveNumber)?;

        let _halfmove_clock = halfmove_clock
            .parse::<u64>()
            .map_err(|_| ParseFenError::BadHalfMoveClock)?;

        let mut board = Board {
            pieces,
            occupancies,
            combined,
            side_to_move,
            en_passant_target,
            castling_rights,
            pinned: Default::default(),
            checkers: Default::default(),
            pinners: Default::default(),
        };

        // TODO: check if board is sane

        let (pinned, checkers, pinners) = calculate_pinned_checkers_pinners(&board);

        board.pinned = pinned;
        board.checkers = checkers;
        board.pinners = pinners;

        Ok(board)
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::color::Color;
    use crate::piece::Piece;
    use crate::square::Square;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        let expected = "
8   r n b q k b n r 
7   p p p p p p p p 
6   . . . . . . . . 
5   . . . . . . . . 
4   . . . . . . . . 
3   . . . . . . . . 
2   P P P P P P P P 
1   R N B Q K B N R 

    a b c d e f g h 

En passant square:	None
Side to move:		White
Castling rights:	KQkq
";
        let board = Board::default();
        println!("{}", board);
        assert_eq!(expected, board.to_string());
    }

    #[test]
    fn test_fen_parsing() {
        let board = Board::from_str("2r5/8/8/3R4/2P1k3/2K5/8/8 b - - 0 1").unwrap();

        assert_eq!(board.piece_on_square(Square::C3), Some(Piece::King));
        assert_eq!(board.piece_on_square(Square::E4), Some(Piece::King));
        assert_eq!(board.piece_on_square(Square::C4), Some(Piece::Pawn));
        assert_eq!(board.piece_on_square(Square::D5), Some(Piece::Rook));
        assert_eq!(board.piece_on_square(Square::C8), Some(Piece::Rook));

        assert_eq!(board.color_on_square(Square::C3), Some(Color::White));
        assert_eq!(board.color_on_square(Square::E4), Some(Color::Black));
        assert_eq!(board.color_on_square(Square::C4), Some(Color::White));
        assert_eq!(board.color_on_square(Square::D5), Some(Color::White));
        assert_eq!(board.color_on_square(Square::C8), Some(Color::Black));

        println!("{board}");
    }
}

use std::str::FromStr;
use crate::{CastlingRights, Color, Piece, PieceKind, Position};

#[derive(Debug, PartialEq)]
pub struct Fen {
    pieces: [Option<Piece>; 64],
    side_to_play: Color,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
    en_passant_target: Option<Position>,
    half_move_clock: u64,
    full_move_number: u64,
}

impl Fen {
    pub fn starting_pos() -> Fen {
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse()
            .unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseFenError {
    PartsMissing,
    BadPlacement,
    TooManyPiecesInRank(u8),
    InvalidPiece(char),
    NoSuchSide(String),
    BadEnPassant,
    BadFullMoveNumber(String),
    BadHalfMoveClock(String),
}

impl FromStr for Fen {
    type Err = ParseFenError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() != 6 {
            return Err(ParseFenError::PartsMissing)
        }

        let pieces = parse_piece_placement(parts[0])?;
        let side_to_play = parse_side_to_play(parts[1])?;
        let (white_castling_rights, black_castling_rights) =
            parse_castling_rights(parts[2])?;
        let en_passant_target = parse_en_passant_target(parts[3])?;
        let half_move_clock = parse_half_move_clock(parts[4])?;
        let full_move_number = parse_full_move_number(parts[5])?;

        Ok(Fen {
            pieces,
            side_to_play,
            white_castling_rights,
            black_castling_rights,
            en_passant_target,
            half_move_clock,
            full_move_number
        })
    }
}

fn parse_full_move_number(full_move_number_str: &str) -> Result<u64, ParseFenError> {
    full_move_number_str
        .parse::<u64>()
        .map_err(|_| ParseFenError::BadFullMoveNumber(full_move_number_str.to_owned()))
}

fn parse_half_move_clock(half_move_clock_str: &str) -> Result<u64, ParseFenError> {
    half_move_clock_str
        .parse::<u64>()
        .map_err(|_| ParseFenError::BadHalfMoveClock(half_move_clock_str.to_owned()))
}

fn parse_en_passant_target(en_passant_target_str: &str) -> Result<Option<Position>, ParseFenError> {
    if en_passant_target_str == "-" {
        return Ok(None);
    }

    let position = en_passant_target_str
        .parse::<Position>()
        .map_err(|_| ParseFenError::BadEnPassant)?;

    Ok(Some(position))
}

fn parse_castling_rights(castling_rights_str: &str) -> Result<(CastlingRights, CastlingRights), ParseFenError> {
    let white_castling_rights = CastlingRights {
        can_queen_side: castling_rights_str.contains('Q'),
        can_king_side: castling_rights_str.contains('K'),
    };

    let black_castling_rights = CastlingRights {
        can_queen_side: castling_rights_str.contains('q'),
        can_king_side: castling_rights_str.contains('k'),
    };

    Ok((white_castling_rights, black_castling_rights))
}

fn parse_side_to_play(side_to_play_str: &str) -> Result<Color, ParseFenError> {
    match side_to_play_str {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(ParseFenError::NoSuchSide(side_to_play_str.to_owned()))
    }
}

fn parse_piece_placement(pieces_str: &str) -> Result<[Option<Piece>; 64], ParseFenError> {
    let mut pieces = [None; 64];

    let ranks_str: Vec<&str> = pieces_str.split('/').collect();
    if ranks_str.len() != 8 {
        return Err(ParseFenError::BadPlacement);
    }

    // reverse iterator to start with rank 1
    for (rank, rank_pieces) in (0u8..).zip(ranks_str.iter().rev()) {
        let mut file: u8 = 0;
        for piece_char in rank_pieces.chars() {
            match piece_char.to_digit(10) {
                // blanks
                Some(n) => {
                    file += n as u8;
                    if file > 8 {
                        return Err(ParseFenError::TooManyPiecesInRank(rank))
                    }
                }
                // piece
                None => {
                    let piece = parse_piece_char(piece_char)?;
                    pieces[(rank * 8 + file) as usize] = Some(piece);
                    file += 1;
                }
            }
        }

    }

    Ok(pieces)
}

fn parse_piece_char(piece: char) -> Result<Piece, ParseFenError> {
    let color = match piece.is_uppercase() {
        true => Color::White,
        false => Color::Black
    };

    let kind = match piece.to_ascii_lowercase() {
        'p' => PieceKind::Pawn,
        'n' => PieceKind::Knight,
        'b' => PieceKind::Bishop,
        'r' => PieceKind::Rook,
        'q' => PieceKind::Queen,
        'k' => PieceKind::King,
        _ => return Err(ParseFenError::InvalidPiece(piece))
    };

    Ok(Piece { color, kind })
}

#[cfg(test)]
mod test {
    use crate::{Color, Piece, PieceKind};
    use crate::fen_parser::{CastlingRights, Fen, ParseFenError};

    #[test]
    fn test_empty_board() -> Result<(), ParseFenError> {
        let empty = Fen {
            pieces: [None; 64],
            side_to_play: Color::White,
            white_castling_rights: CastlingRights { can_king_side: false, can_queen_side: false},
            black_castling_rights:  CastlingRights { can_king_side: false, can_queen_side: false},
            en_passant_target: None,
            half_move_clock: 0,
            full_move_number: 1,
        };

        assert_eq!("8/8/8/8/8/8/8/8 w - - 0 1".parse::<Fen>()?, empty);
        Ok(())
    }

    #[test]
    fn test_starting_pos_board() -> Result<(), ParseFenError> {
        let mut pieces = [None; 64];

        // set white pieces
        // pawns
        for file in 0..8 {
            pieces[1 * 8 + file] = Some(Piece { kind: PieceKind::Pawn, color: Color::White});
        }

        // set rooks
        pieces[0 * 8 + 0] = Some(Piece { kind: PieceKind::Rook, color: Color::White});
        pieces[0 * 8 + 7] = Some(Piece { kind: PieceKind::Rook, color: Color::White});

        // set knights
        pieces[0 * 8 + 1] = Some(Piece { kind: PieceKind::Knight, color: Color::White});
        pieces[0 * 8 + 6] = Some(Piece { kind: PieceKind::Knight, color: Color::White});

        // set bishops
        pieces[0 * 8 + 2] = Some(Piece { kind: PieceKind::Bishop, color: Color::White});
        pieces[0 * 8 + 5] = Some(Piece { kind: PieceKind::Bishop, color: Color::White});

        // set king & queen
        pieces[0 * 8 + 3] = Some(Piece { kind: PieceKind::Queen, color: Color::White});
        pieces[0 * 8 + 4] = Some(Piece { kind: PieceKind::King, color: Color::White});

        // mirror for black
        for rank in 0..2 {
            for file in 0..8 {
                pieces[(7 - rank) * 8 + file] = pieces[rank * 8 + file]
                    .map(
                        |piece| Piece { color: Color::Black, ..piece}
                    );
            }
        }

        let starting_pos = Fen {
            pieces,
            side_to_play: Color::White,
            white_castling_rights: CastlingRights { can_queen_side: true, can_king_side: true },
            black_castling_rights: CastlingRights { can_queen_side: true, can_king_side: true },
            en_passant_target: None,
            half_move_clock: 0,
            full_move_number: 1,
        };


        assert_eq!(Fen::starting_pos(), starting_pos);
        Ok(())
    }

    #[test]
    fn test_pawn_endgame() -> Result<(), ParseFenError> {
        let mut pieces = [None; 64];
        pieces[7 * 8 + 5] = Some(Piece { color: Color::Black, kind: PieceKind::King });
        pieces[5 * 8 + 4] = Some(Piece { color: Color::White, kind: PieceKind::King });
        pieces[4 * 8 + 5] = Some(Piece { color: Color::White, kind: PieceKind::Pawn });

        let endgame = Fen {
            pieces,
            side_to_play: Color::White,
            white_castling_rights: CastlingRights { can_queen_side: false, can_king_side: false },
            black_castling_rights: CastlingRights { can_queen_side: false, can_king_side: false },
            en_passant_target: None,
            half_move_clock: 0,
            full_move_number: 1,
        };

        assert_eq!( "5k2/8/4K3/5P2/8/8/8/8 w - - 0 1".parse::<Fen>()?, endgame);
        Ok(())
    }
}
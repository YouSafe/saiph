use crate::movegen::attacks::{
    between, bishop_attacks, knight_attacks, pawn_attacks, rook_attacks,
};
use crate::movegen::{MoveList, generate_moves};
use crate::types::bitboard::BitBoard;
use crate::types::castling_rights::{CastlingRights, UPDATE_CASTLING_RIGHT_TABLE};
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::color::{Color, PerColor};
use crate::types::piece::{ALL_PIECES, PerPieceType, Piece, PieceType};
use crate::types::square::{File, PerSquare, Square};
use crate::zobrist::{self};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct BoardState {
    hash: u64,
    en_passant_target: Option<Square>,
    castling_rights: CastlingRights,
    rule50: u8,
    checkers: BitBoard,
    pinned: BitBoard,
    last_move: Option<Move>,
    captured_piece: Option<Piece>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pieces: PerPieceType<BitBoard>,
    occupancies: PerColor<BitBoard>,
    combined: BitBoard,
    mailbox: PerSquare<Option<Piece>>,
    side_to_move: Color,
    history: Vec<BoardState>,
    state: BoardState,
    game_ply: u16,
}

impl Board {
    pub const STARTING_POS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const KILLER_POS_FEN: &'static str =
        "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";

    pub fn apply_move(&mut self, mv: Move) {
        let mut new_state = self.state.clone();
        new_state.last_move = Some(mv);

        let from = mv.from();
        let to = mv.to();

        self.game_ply += 1;
        new_state.rule50 += 1;

        new_state.en_passant_target = None;
        if let Some(en_passant_target) = self.en_passant_target() {
            new_state.hash ^= zobrist::en_passant_keys(en_passant_target.file());
        }

        new_state.captured_piece = None;

        let mut source_piece = self
            .piece_at(from)
            .expect("source square should not be empty");

        let target_piece = self.piece_at(to);

        self.remove_piece(from, source_piece);
        new_state.hash ^= zobrist::piece_keys(self.side_to_move, source_piece.ty(), from);

        let mut capture_target: Option<(Square, Piece)> = None;

        let flag = mv.flag();
        match flag {
            MoveFlag::Normal => (),
            MoveFlag::DoublePawnPush => {
                new_state.en_passant_target = Some(to.forward(!self.side_to_move).unwrap());
                new_state.hash ^= zobrist::en_passant_keys(to.file());
            }
            MoveFlag::Castling => {
                const CASTLE_CONFIG: [(File, File); 2] = [(File::A, File::D), (File::H, File::F)];

                let backrank = self.side_to_move.backrank();
                let target_file = to.file();
                let (rook_start_file, rook_end_file) = CASTLE_CONFIG[target_file as usize / 4];
                let (rook_start_square, rook_end_square) = (
                    Square::from(backrank, rook_start_file),
                    Square::from(backrank, rook_end_file),
                );

                let rook_piece = PieceType::Rook.to_piece(self.side_to_move);

                self.remove_piece(rook_start_square, rook_piece);
                new_state.hash ^=
                    zobrist::piece_keys(self.side_to_move, PieceType::Rook, rook_start_square);

                self.put_piece(rook_end_square, rook_piece);
                new_state.hash ^=
                    zobrist::piece_keys(self.side_to_move, PieceType::Rook, rook_end_square);
            }
            MoveFlag::Capture => {
                capture_target = Some((
                    to,
                    target_piece.expect("target square should not be empty upon capture"),
                ));
            }
            MoveFlag::EnPassant => {
                capture_target = Some((
                    to.forward(!self.side_to_move).unwrap(),
                    Piece::new(PieceType::Pawn, !self.side_to_move),
                ));
            }
            MoveFlag::KnightPromotion
            | MoveFlag::BishopPromotion
            | MoveFlag::RookPromotion
            | MoveFlag::QueenPromotion => {
                source_piece = ALL_PIECES
                    [(flag as usize) - (MoveFlag::KnightPromotion as usize) + 1]
                    .to_piece(self.side_to_move);

                // reset early since moved piece was originally a pawn
                new_state.rule50 = 0;
            }

            MoveFlag::KnightPromotionCapture
            | MoveFlag::BishopPromotionCapture
            | MoveFlag::RookPromotionCapture
            | MoveFlag::QueenPromotionCapture => {
                capture_target = Some((
                    to,
                    target_piece.expect("target square should not be empty upon promotion capture"),
                ));

                source_piece = ALL_PIECES
                    [(flag as usize) - (MoveFlag::KnightPromotionCapture as usize) + 1]
                    .to_piece(self.side_to_move);

                // reset early since moved piece was originally a pawn
                new_state.rule50 = 0;
            }
        }

        if let Some((square, piece)) = capture_target {
            new_state.captured_piece = Some(piece);
            self.remove_piece(square, piece);
            new_state.hash ^= zobrist::piece_keys(!self.side_to_move, piece.ty(), square);

            // remove castling right for that side
            if piece.ty() == PieceType::Rook {
                new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);

                new_state.castling_rights &= UPDATE_CASTLING_RIGHT_TABLE[from];
                new_state.castling_rights &= UPDATE_CASTLING_RIGHT_TABLE[to];

                new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);
            }

            new_state.rule50 = 0;
        }

        self.put_piece(to, source_piece);
        new_state.hash ^= zobrist::piece_keys(self.side_to_move, source_piece.ty(), to);

        if source_piece.ty() == PieceType::Pawn {
            new_state.rule50 = 0;
        }

        // update castling rights
        if source_piece.ty() == PieceType::Rook {
            // rook moved
            new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);
            new_state.castling_rights &= UPDATE_CASTLING_RIGHT_TABLE[from];
            new_state.castling_rights &= UPDATE_CASTLING_RIGHT_TABLE[to];
            new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);
        } else if source_piece.ty() == PieceType::King {
            // remove castling rights for side if king moved (includes castling)
            new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);
            new_state.castling_rights -= match self.side_to_move {
                Color::White => CastlingRights::WHITE_BOTH_SIDES,
                Color::Black => CastlingRights::BLACK_BOTH_SIDES,
            };
            new_state.hash ^= zobrist::castle_keys(new_state.castling_rights);
        }

        // update side
        self.side_to_move = !self.side_to_move;
        new_state.hash ^= zobrist::side_key();

        self.update_checker_info(&mut new_state);

        let old_state = std::mem::replace(&mut self.state, new_state);
        self.history.push(old_state);
    }

    pub fn undo_move(&mut self) {
        // revert last move from popped state
        let last_move = self
            .state
            .last_move
            .expect("there should be a move to undo");

        let from = last_move.from();
        let to = last_move.to();

        self.side_to_move = !self.side_to_move;

        let mut capture_target: Option<Square> = None;

        // when moving from one square to another, the left square is left empty
        assert_eq!(self.piece_at(from), None);

        let mut source_piece = self
            .piece_at(to)
            .expect("target square should not be empty after the last move");

        self.remove_piece(to, source_piece);

        match last_move.flag() {
            MoveFlag::Normal => (),
            MoveFlag::DoublePawnPush => {
                // en passant target square is restored via popping the state history
            }
            MoveFlag::Castling => {
                const CASTLE_CONFIG: [(File, File); 2] = [(File::A, File::D), (File::H, File::F)];
                let backrank = self.side_to_move.backrank();
                let target_file = to.file();
                let (rook_start_file, rook_end_file) = CASTLE_CONFIG[target_file as usize / 4];
                let (rook_start_square, rook_end_square) = (
                    Square::from(backrank, rook_start_file),
                    Square::from(backrank, rook_end_file),
                );

                let rook_piece = Piece::new(PieceType::Rook, self.side_to_move);

                self.remove_piece(rook_end_square, rook_piece);
                self.put_piece(rook_start_square, rook_piece);
            }
            MoveFlag::Capture => capture_target = Some(to),
            MoveFlag::EnPassant => {
                capture_target = Some(to.forward(!self.side_to_move).unwrap());
            }
            MoveFlag::KnightPromotion
            | MoveFlag::BishopPromotion
            | MoveFlag::RookPromotion
            | MoveFlag::QueenPromotion => {
                source_piece = Piece::new(PieceType::Pawn, self.side_to_move);
            }
            MoveFlag::KnightPromotionCapture
            | MoveFlag::BishopPromotionCapture
            | MoveFlag::RookPromotionCapture
            | MoveFlag::QueenPromotionCapture => {
                source_piece = Piece::new(PieceType::Pawn, self.side_to_move);
                capture_target = Some(to);
            }
        }

        self.put_piece(from, source_piece);

        if let Some(capture_target) = capture_target {
            let capture_piece = self
                .state
                .captured_piece
                .expect("there should be captured piece set since the last move was a capture");
            self.put_piece(capture_target, capture_piece);
        }

        self.game_ply -= 1;

        if let Some(previous_state) = self.history.pop() {
            self.state = previous_state;
        }
    }

    pub fn make_null(&mut self) {
        assert!(self.checkers().is_empty());

        let mut new_state = self.state.clone();
        if let Some(target) = self.state.en_passant_target {
            new_state.en_passant_target = None;
            new_state.hash ^= zobrist::en_passant_keys(target.file());
        }

        self.side_to_move = !self.side_to_move;
        new_state.hash ^= zobrist::side_key();

        self.update_checker_info(&mut new_state);

        let old_state = std::mem::replace(&mut self.state, new_state);
        self.history.push(old_state);
    }

    pub fn undo_null(&mut self) {
        assert!(self.checkers().is_empty());

        if let Some(previous_state) = self.history.pop() {
            self.state = previous_state;
        }

        self.side_to_move = !self.side_to_move;
    }

    fn update_checker_info(&mut self, new_state: &mut BoardState) {
        let king_square =
            (self.pieces(PieceType::King) & self.occupancies(self.side_to_move())).bit_scan();

        let mut potential_pinners = BitBoard(0);
        let mut pinned = BitBoard(0);

        let mut checkers = BitBoard(0);

        // pretend king is a bishop and see if any other bishop OR queen is attacked by that
        potential_pinners |= bishop_attacks(king_square, BitBoard(0))
            & (self.pieces(PieceType::Bishop) | self.pieces(PieceType::Queen));

        // now pretend the king is a rook and so the same procedure
        potential_pinners |= rook_attacks(king_square, BitBoard(0))
            & (self.pieces(PieceType::Rook) | self.pieces(PieceType::Queen));

        // limit to opponent's pieces
        potential_pinners &= self.occupancies(!self.side_to_move());

        for square in potential_pinners.iter() {
            let potentially_pinned = between(square, king_square) & self.combined();
            if potentially_pinned.is_empty() {
                checkers |= square;
            } else if potentially_pinned.count() == 1 {
                pinned |= potentially_pinned;
            }
        }

        // now pretend the king is a knight and check if it attacks an enemy knight
        checkers |= knight_attacks(king_square)
            & self.pieces(PieceType::Knight)
            & self.occupancies(!self.side_to_move());

        // do the same thing for pawns
        checkers |= pawn_attacks(king_square, self.side_to_move())
            & self.pieces(PieceType::Pawn)
            & self.occupancies(!self.side_to_move());

        // update pinned, checkers
        new_state.pinned = pinned;
        new_state.checkers = checkers;
    }

    fn put_piece(&mut self, sq: Square, piece: Piece) {
        debug_assert_eq!(self.piece_at(sq), None);

        self.pieces[piece.ty()] ^= sq;
        self.occupancies[piece.color()] ^= sq;
        self.combined ^= sq;

        self.mailbox[sq] = Some(piece);
    }

    fn remove_piece(&mut self, sq: Square, piece: Piece) {
        debug_assert_eq!(self.piece_at(sq), Some(piece));

        self.pieces[piece.ty()] ^= sq;
        self.occupancies[piece.color()] ^= sq;
        self.combined ^= sq;

        self.mailbox[sq] = None;
    }

    pub fn generate_moves(&self) -> MoveList {
        generate_moves::<false>(self)
    }

    pub fn generate_moves_captures_only(&self) -> MoveList {
        generate_moves::<true>(self)
    }

    pub fn is_repetition(&self) -> bool {
        self.history
            .iter()
            .rev()
            .take(self.state.rule50 as usize)
            .filter(|c| self.state.hash == c.hash)
            .count()
            >= 1
    }

    pub fn game_ply(&self) -> u16 {
        self.game_ply
    }

    pub fn piece_at(&self, square: Square) -> Option<Piece> {
        self.mailbox[square]
    }

    pub fn pieces(&self, piece: PieceType) -> BitBoard {
        self.pieces[piece]
    }

    pub fn occupancies(&self, color: Color) -> BitBoard {
        self.occupancies[color]
    }

    pub fn combined(&self) -> BitBoard {
        self.combined
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn checkers(&self) -> BitBoard {
        self.state.checkers
    }

    pub fn pinned(&self) -> BitBoard {
        self.state.pinned
    }

    pub fn castling_rights(&self) -> CastlingRights {
        self.state.castling_rights
    }

    pub fn en_passant_target(&self) -> Option<Square> {
        self.state.en_passant_target
    }

    pub fn is_draw_by_fifty_move_rule(&self) -> bool {
        self.state.rule50 >= 100
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.state.hash
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            write!(f, "{}   ", rank + 1)?;
            for file in 0..8 {
                let square = Square::from_index(rank * 8 + file);
                let symbol = if let Some(piece) = self.mailbox[square] {
                    piece.to_ascii()
                } else {
                    '.'
                };
                write!(f, "{symbol} ")?;
            }
            writeln!(f)?;
        }
        write!(f, "\n    ")?;
        for file in 'a'..='h' {
            write!(f, "{file} ")?;
        }

        writeln!(f, "\n")?;
        writeln!(f, "En passant square:\t{:?}", self.state.en_passant_target)?;
        writeln!(f, "Side to move:\t\t{:?}", self.side_to_move)?;
        writeln!(f, "Castling rights:\t{}", self.state.castling_rights)?;
        writeln!(f, "Captured piece:\t{:?}", self.state.captured_piece)?;
        writeln!(f, "Last move:\t{:?}", self.state.last_move)?;
        writeln!(f, "Hash: \t{:#018x}", self.state.hash)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ParseFenError {
    PartMissing(&'static str),
    BadPlacement,
    NoSuchSide,
    BadCastlingRights,
    BadFullMoveNumber,
    BadHalfMoveClock,
    BadEnPassant,
    TooManyFiles,
    TooManyRanks,
    InvalidPiece(char),
}

impl FromStr for Board {
    type Err = ParseFenError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let mut parts = fen.split(" ");

        let mut pieces = PerPieceType::<BitBoard>::default();
        let mut occupancies = PerColor::<BitBoard>::default();
        let mut combined = BitBoard(0);

        let mut mailbox = PerSquare::<Option<Piece>>::default();

        let mut hash = 0;

        let piece_placement_data = parts
            .next()
            .ok_or(ParseFenError::PartMissing("piece placement data"))?;

        let mut rank: u8 = 7;
        let mut file: u8 = 0;

        for char in piece_placement_data.chars() {
            match char {
                'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                    let square = Square::from_index(rank * 8 + file);
                    let piece =
                        Piece::from_algebraic(char).ok_or(ParseFenError::InvalidPiece(char))?;

                    let piece_type = piece.ty();
                    let color = piece.color();

                    pieces[piece_type] |= square;
                    occupancies[color] |= square;
                    combined |= square;

                    mailbox[square] = Some(piece);

                    hash ^= zobrist::piece_keys(color, piece_type, square);

                    file += 1;
                    if file > 8 {
                        return Err(ParseFenError::TooManyFiles);
                    }
                }
                '1'..='8' => {
                    file += char.to_digit(10).unwrap() as u8;

                    if file > 8 {
                        return Err(ParseFenError::TooManyFiles);
                    }
                }
                '/' => {
                    if rank == 0 {
                        return Err(ParseFenError::TooManyRanks);
                    }

                    rank -= 1;
                    file = 0;
                }
                _ => {
                    return Err(ParseFenError::BadPlacement);
                }
            }
        }

        let side_to_move = match parts
            .next()
            .ok_or(ParseFenError::PartMissing("active color"))?
        {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(ParseFenError::NoSuchSide),
        };

        let castling_rights = parts
            .next()
            .ok_or(ParseFenError::PartMissing("castling rights"))?
            .parse::<CastlingRights>()
            .map_err(|_| ParseFenError::BadCastlingRights)?;

        let en_passant_target = match parts
            .next()
            .ok_or(ParseFenError::PartMissing("en passant target"))?
        {
            "-" => None,
            target => Some(
                target
                    .parse::<Square>()
                    .map_err(|_| ParseFenError::BadEnPassant)?,
            ),
        };

        let halfmove_clock = parts
            .next()
            .ok_or(ParseFenError::PartMissing("halfmove clock"))?
            .parse::<u8>()
            .map_err(|_| ParseFenError::BadHalfMoveClock)?;

        let fullmove_number = parts
            .next()
            .ok_or(ParseFenError::PartMissing("fullmove number"))?
            .parse::<u16>()
            .map_err(|_| ParseFenError::BadFullMoveNumber)?;

        if let Some(en_passant_target) = en_passant_target {
            hash ^= zobrist::en_passant_keys(en_passant_target.file());
        }

        hash ^= zobrist::castle_keys(castling_rights);

        if side_to_move == Color::Black {
            hash ^= zobrist::side_key();
        }

        // ========== CALCULATE PINNED & CHECKERS ==========
        let king_square = (pieces[PieceType::King] & occupancies[side_to_move]).bit_scan();

        let mut potential_pinners = BitBoard(0);
        let mut pinned = BitBoard(0);

        let mut checkers = BitBoard(0);

        // pretend king is a bishop and see if any other bishop OR queen is attacked by that
        potential_pinners |= bishop_attacks(king_square, BitBoard(0))
            & (pieces[PieceType::Bishop] | pieces[PieceType::Queen]);

        // now pretend the king is a rook and so the same procedure
        potential_pinners |= rook_attacks(king_square, BitBoard(0))
            & (pieces[PieceType::Rook] | pieces[PieceType::Queen]);

        // limit to opponent's pieces
        potential_pinners &= occupancies[!side_to_move];

        for square in potential_pinners.iter() {
            let potentially_pinned = between(square, king_square) & combined;
            if potentially_pinned.is_empty() {
                checkers |= square;
            } else if potentially_pinned.count() == 1 {
                pinned |= potentially_pinned;
            }
        }

        // now pretend the king is a knight and check if it attacks an enemy knight
        checkers |=
            knight_attacks(king_square) & pieces[PieceType::Knight] & occupancies[!side_to_move];

        // do the same thing for pawns
        checkers |= pawn_attacks(king_square, side_to_move)
            & pieces[PieceType::Pawn]
            & occupancies[!side_to_move];

        // ======================================================

        let board = Board {
            pieces,
            occupancies,
            combined,
            side_to_move,
            mailbox,
            state: BoardState {
                hash,
                en_passant_target,
                castling_rights,
                rule50: halfmove_clock,
                checkers,
                pinned,
                last_move: None,
                captured_piece: None,
            },
            history: vec![],
            game_ply: 2 * fullmove_number.saturating_sub(1) + (side_to_move == Color::Black) as u16,
        };

        // TODO: check if board is sane

        Ok(board)
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.pieces == other.pieces
            && self.occupancies == other.occupancies
            && self.combined == other.combined
            && self.side_to_move == other.side_to_move
    }
}

impl Eq for Board {}

impl Default for Board {
    fn default() -> Self {
        Board::STARTING_POS_FEN.parse().unwrap()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;

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
Captured piece:	None
Last move:	None
Hash: 	0x4a887e3c9bc2624a
";
        let board = Board::default();
        println!("{board}");
        assert_eq!(expected, board.to_string());
    }

    #[test]
    fn test_fen_parsing() {
        let board = Board::from_str("2r5/8/8/3R4/2P1k3/2K5/8/8 b - - 0 1").unwrap();

        use crate::types::piece::Piece::*;
        use crate::types::square::Square::*;

        assert_eq!(board.piece_at(C3), Some(WhiteKing));
        assert_eq!(board.piece_at(E4), Some(BlackKing));
        assert_eq!(board.piece_at(C4), Some(WhitePawn));
        assert_eq!(board.piece_at(D5), Some(WhiteRook));
        assert_eq!(board.piece_at(C8), Some(BlackRook));

        println!("{board}");
    }

    #[test]
    fn test_make_null() {
        let board = Board::from_str("2r5/8/8/3R4/2P1k3/2K5/8/8 b - - 0 1").unwrap();
        let mut clone = board.clone();
        clone.make_null();
        clone.undo_null();
        println!("{board}, {clone}");
        assert_eq!(board, clone);
    }
}

mod castling;
mod en_passant;
mod king;
mod knight;
mod pawn_capture;
mod quiet_pawn;
mod slider;

use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::chess_move::Move;
use crate::color::Color;
use crate::movgen::pawn_capture::PawnCaptureMoveGenerator;
use crate::movgen::quiet_pawn::QuietPawnMoveGenerator;
use crate::piece::{Piece, ALL_PIECES};
use crate::square::Square;
use crate::tables::{
    between, get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks,
    get_queen_attacks, get_rook_attacks,
};

type MoveList = Vec<Move>;

pub fn generate_attack_bitboard(board: &Board, attacking_color: Color) -> BitBoard {
    let mut attacked = BitBoard(0);

    let king = board.pieces(Piece::King) & board.occupancies(!attacking_color);

    // remove opponent king from blockers to simulate xray attack
    let blockers = *board.combined() & !king;

    for piece in ALL_PIECES {
        let piece_bitboard = *board.pieces(piece) & board.occupancies(attacking_color);
        for square in piece_bitboard.iter() {
            let piece = board
                .piece_on_square(square)
                .expect("piece must not be none");

            attacked |= match piece {
                Piece::Pawn => get_pawn_attacks(square, attacking_color),
                Piece::Knight => get_knight_attacks(square),
                Piece::Bishop => get_bishop_attacks(square, blockers),
                Piece::Rook => get_rook_attacks(square, blockers),
                Piece::Queen => get_queen_attacks(square, blockers),
                Piece::King => get_king_attacks(square),
            };
        }
    }

    attacked
}

pub fn calculate_pinned_checkers_pinners(board: &Board) -> (BitBoard, BitBoard, BitBoard) {
    let king_square =
        (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

    let mut potential_pinners = BitBoard(0);
    let mut pinned = BitBoard(0);

    let mut checkers = BitBoard(0);

    // pretend king is a bishop and see if any other bishop OR queen is attacked by that
    potential_pinners |= get_bishop_attacks(king_square, BitBoard(0))
        & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen));

    // now pretend the king is a rook and so the same procedure
    potential_pinners |= get_rook_attacks(king_square, BitBoard(0))
        & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen));

    // limit to opponent's pieces
    potential_pinners &= board.occupancies(!board.side_to_move());

    let mut pinners = BitBoard(0);

    for square in potential_pinners.iter() {
        let potentially_pinned = between(square, king_square) & board.combined();
        if potentially_pinned.is_empty() {
            checkers |= square;
        } else if potentially_pinned.popcnt() == 1 {
            pinned |= potentially_pinned;
            pinners |= potential_pinners;
        }
    }

    // now pretend the king is a knight and check if it attacks an enemy knight
    checkers |= get_knight_attacks(king_square)
        & board.pieces(Piece::Knight)
        & board.occupancies(!board.side_to_move());

    // do the same thing for pawns
    checkers |= get_pawn_attacks(king_square, board.side_to_move())
        & board.pieces(Piece::Pawn)
        & board.occupancies(!board.side_to_move());

    (pinned, checkers, pinners)
}

trait CheckState {}

struct InCheck;
struct NotInCheck;

impl CheckState for InCheck {}
impl CheckState for NotInCheck {}

trait PieceMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList);
}

pub fn generate_moves(board: &Board) -> MoveList {
    let mut move_list = vec![];

    let checkers = board.checkers();
    if checkers.popcnt() == 0 {
        // differentiate between pinned and not pinned
        // if not pinned create all the moves
        // if pinned calculate mask where piece can go to
        // pinned means masking the to squares to those between the pinner and the king

        // king can only go to squares that are not attacked (only the squares 1 away from the
        // king need to be considered)

        // pinned pieces can only move towards or away from the pinner
        // if pinned
        // finally get the bitboard for the squares between the king and the pinner and use it as a mask

        // edge-case en-passant move that leads to a discovered attack (deal with this separately)

        // PAWN MOVES
        QuietPawnMoveGenerator::generate::<NotInCheck>(board, &mut move_list);
        PawnCaptureMoveGenerator::generate::<NotInCheck>(board, &mut move_list);

        // KNIGHT MOVES

        // SLIDERS MOVES

        // KING MOVES
    } else if checkers.popcnt() == 1 {
        // a single check can be evaded by capturing the checker

        // stop castling when king is in check

        // PAWN MOVES
        QuietPawnMoveGenerator::generate::<InCheck>(board, &mut move_list);
        PawnCaptureMoveGenerator::generate::<InCheck>(board, &mut move_list);

        // KNIGHT MOVES

        // SLIDERS MOVES

        // KING MOVES
    } else {
        // double and more checkers
        // only the king can move

        // KING MOVES
    }

    move_list
}

pub fn is_square_attacked(board: &Board, attacked_square: Square, attacking_side: Color) -> bool {
    // attacked by pawns?
    if (get_pawn_attacks(attacked_square, !attacking_side)
        & board.pieces(Piece::Pawn)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by knight?
    if (get_knight_attacks(attacked_square)
        & board.pieces(Piece::Knight)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by king?
    if (get_king_attacks(attacked_square)
        & board.pieces(Piece::King)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by bishop or queen?
    if (get_bishop_attacks(attacked_square, *board.combined())
        & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by rook or queen?
    if (get_rook_attacks(attacked_square, *board.combined())
        & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    false
}

pub fn build_attacked_bitboard(board: &Board, attacking_side: Color) -> BitBoard {
    let mut bitboard = BitBoard(0);
    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::from_index(rank * 8 + file);

            if is_square_attacked(board, square, attacking_side) {
                bitboard |= square;
            }
        }
    }
    bitboard
}

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::board::Board;
    use crate::color::Color;
    use crate::movgen::{
        build_attacked_bitboard, calculate_pinned_checkers_pinners, generate_attack_bitboard,
        generate_moves, is_square_attacked,
    };
    use crate::square::Square;
    use std::str::FromStr;

    #[test]
    fn test_is_square_attacked_pawn_attack() {
        let board = Board::from_str("k7/8/8/3p4/8/8/8/K7 w - - 0 1").unwrap();
        println!("board: {board}");

        let is_e4_attacked_by_black = is_square_attacked(&board, Square::E4, Color::Black);

        assert!(is_e4_attacked_by_black);
        println!("is e4 attacked by black pawn? {is_e4_attacked_by_black}",);
    }

    #[test]
    fn test_is_square_attacked_bishop_attack() {
        let board = Board::from_str("7k/8/2P5/8/8/8/6b1/K7 w - - 0 1").unwrap();
        println!("board: {board}");

        let is_c6_attacked_by_black = is_square_attacked(&board, Square::C6, Color::Black);

        assert!(is_c6_attacked_by_black);

        println!(
            "is the pawn on c6 attacked by the bishop on g2? {}",
            is_c6_attacked_by_black
        );
    }

    #[test]
    fn test_is_square_attacked_rook_attack() {
        let board = Board::from_str("7k/8/8/8/2R2b2/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_f4_attacked_by_white = is_square_attacked(&board, Square::F4, Color::White);

        assert!(is_f4_attacked_by_white);

        println!(
            "is the bishop on f4 attacked by the rook on c4? {}",
            is_f4_attacked_by_white
        );
    }

    #[test]
    fn test_is_square_attacked_knight_attack() {
        let board = Board::from_str("7k/8/4q3/8/3N4/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_e6_attacked_by_white = is_square_attacked(&board, Square::E6, Color::White);

        assert!(is_e6_attacked_by_white);

        println!(
            "is the queen on e6 attacked by the knight on d4? {}",
            is_e6_attacked_by_white
        );
    }

    #[test]
    fn test_is_square_attacked_queen_attack() {
        let board = Board::from_str("7k/P7/8/8/8/8/8/K5q1 w - - 0 1").unwrap();
        println!("{board}");

        let is_a7_attacked_by_black = is_square_attacked(&board, Square::A7, Color::Black);

        assert!(is_a7_attacked_by_black);

        println!(
            "is the pawn on a7 attacked by the queen on g1? {}",
            is_a7_attacked_by_black
        );
    }

    #[test]
    fn test_build_attacked_bitboard() {
        let board = Board::default();
        let attacked = build_attacked_bitboard(&board, Color::White);
        println!("{attacked}");

        let expected = BitBoard(16777086);
        println!("expected: {expected}");
        assert_eq!(attacked, expected);
    }

    #[test]
    fn test_generate_attack_bitboard() {
        let board = Board::default();
        let attacked = generate_attack_bitboard(&board, Color::White);

        println!("{}", board.combined());
        println!("{attacked}");

        let test = build_attacked_bitboard(&board, Color::White);
        println!("{test}");
        assert_eq!(attacked, test);
    }

    #[test]
    fn test_generate_pinned_checkers() {
        let board = Board::from_str("Q2k3Q/1N2PN2/1QN1NQ2/8/3R4/3K4/8/8 b - - 0 1").unwrap();

        let (pinned, checkers, _pinners) = calculate_pinned_checkers_pinners(&board);

        println!("{board}");

        println!("pinned: {pinned}");

        println!("checkers: {checkers}");
    }

    #[test]
    fn test_mov() {
        let board = Board::from_str("4k3/8/6n1/4R3/8/8/8/4K3 b - - 0 1").unwrap();
        // let board = Board::default();
        generate_moves(&board);
    }
}

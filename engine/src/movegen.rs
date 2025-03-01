use arrayvec::ArrayVec;
use castling::generate_castling_moves;
use en_passant::generate_en_passant_move;
use king::generate_king_moves;
use knight::generate_knight_moves;
use pawn_capture::generate_pawn_capture_moves;
use quiet_pawn::generate_quiet_pawn_moves;
use slider::generate_slider_moves;

use crate::board::Board;
use crate::movegen::attacks::{
    get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks, get_queen_attacks,
    get_rook_attacks,
};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::types::piece::{Piece, ALL_PIECES};
use crate::types::square::Square;
use crate::Printer;

pub(crate) mod attacks;
mod castling;
mod en_passant;
mod king;
mod knight;
mod pawn_capture;
mod quiet_pawn;
mod slider;

pub type MoveList = ArrayVec<Move, 256>;

pub type OrderingList = ArrayVec<i32, 256>;

pub fn generate_attack_bitboard(board: &Board, attacking_color: Color) -> BitBoard {
    let mut attacked = BitBoard(0);

    let king = board.pieces(Piece::King) & board.occupancies(!attacking_color);

    // remove opponent king from blockers to simulate xray attack
    let blockers = board.combined() & !king;

    for piece in ALL_PIECES {
        let piece_bitboard = board.pieces(piece) & board.occupancies(attacking_color);
        for square in piece_bitboard.iter() {
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

pub fn generate_moves<const CAPTURE_ONLY: bool>(board: &Board) -> MoveList {
    let mut move_list = MoveList::new();

    let checkers = board.checkers();
    if checkers.count() == 0 {
        if !CAPTURE_ONLY {
            generate_quiet_pawn_moves::<false>(board, &mut move_list);
        }

        generate_pawn_capture_moves::<false>(board, &mut move_list);
        generate_en_passant_move::<false>(board, &mut move_list);

        generate_knight_moves::<false, CAPTURE_ONLY>(board, &mut move_list);

        generate_slider_moves::<false, CAPTURE_ONLY>(board, &mut move_list);

        if !CAPTURE_ONLY {
            generate_castling_moves::<false>(board, &mut move_list);
        }

        generate_king_moves::<false, CAPTURE_ONLY>(board, &mut move_list);
    } else if checkers.count() == 1 {
        // a single check can be evaded by capturing the checker, blocking the check or by moving the king

        generate_quiet_pawn_moves::<true>(board, &mut move_list);
        generate_pawn_capture_moves::<true>(board, &mut move_list);
        generate_en_passant_move::<true>(board, &mut move_list);

        generate_knight_moves::<true, CAPTURE_ONLY>(board, &mut move_list);

        generate_slider_moves::<true, CAPTURE_ONLY>(board, &mut move_list);

        // castling is not allowed when the king is in check
        generate_king_moves::<true, CAPTURE_ONLY>(board, &mut move_list);
    } else {
        // double and more checkers
        // only the king can move
        generate_king_moves::<true, CAPTURE_ONLY>(board, &mut move_list);
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
    if (get_bishop_attacks(attacked_square, board.combined())
        & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by rook or queen?
    if (get_rook_attacks(attacked_square, board.combined())
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
    for square in 0..64 {
        let square = Square::from_index(square);

        if is_square_attacked(board, square, attacking_side) {
            bitboard |= square;
        }
    }
    bitboard
}

pub fn perf_test<P: Printer>(board: &mut Board, depth: u8) {
    let mut total_nodes = 0;

    let moves = board.generate_moves();
    for mov in moves {
        let mut nodes = 0;

        board.apply_move(mov);
        perf_driver(board, depth - 1, &mut nodes);
        board.undo_move();

        P::println(&format!("{mov} {nodes}"));
        total_nodes += nodes;
    }

    P::println("");
    P::println(&format!("{total_nodes}"));
}

pub fn perf_driver(board: &mut Board, depth: u8, nodes: &mut u64) {
    if depth == 0 {
        *nodes += 1;
        return;
    }

    let moves = board.generate_moves();
    if depth == 1 {
        *nodes += moves.len() as u64;
        return;
    }
    for mov in moves {
        board.apply_move(mov);
        perf_driver(board, depth - 1, nodes);
        board.undo_move();
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::movegen::{
        build_attacked_bitboard, generate_attack_bitboard, generate_moves, is_square_attacked,
    };
    use crate::types::bitboard::BitBoard;
    use crate::types::color::Color;
    use crate::types::square::Square;

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
    fn moves_at_startpos() {
        let board = Board::default();
        let moves = generate_moves::<false>(&board);

        println!("{:#?}", moves);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn moves_at_kiwipete() {
        let board = Board::from_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        let moves = generate_moves::<false>(&board);
        println!("{:#?}", moves);

        assert_eq!(moves.len(), 46);
    }

    #[test]
    fn captures_only() {
        let board = Board::from_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        let moves = generate_moves::<true>(&board);
        println!("{:#?}", moves);

        assert_eq!(moves.len(), 4);
    }
}

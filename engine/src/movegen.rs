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
    between, get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks,
    get_queen_attacks, get_rook_attacks,
};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::types::piece::{PieceType, ALL_PIECES};
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

pub fn generate_moves<const CAPTURE_ONLY: bool>(board: &Board) -> MoveList {
    let mut move_list = MoveList::new();

    let checkers = board.checkers();

    let PushCaptureMasks {
        push_mask,
        capture_mask,
    } = compute_push_capture_mask::<CAPTURE_ONLY>(board);

    let PushCaptureMasks {
        push_mask: king_push_mask,
        capture_mask: king_capture_mask,
    } = compute_king_push_capture_masks::<CAPTURE_ONLY>(board);

    if checkers.count() == 0 {
        if !CAPTURE_ONLY {
            generate_quiet_pawn_moves(board, &mut move_list, push_mask);
            generate_castling_moves(board, &mut move_list);
        }

        generate_pawn_capture_moves(board, &mut move_list, capture_mask);
        generate_en_passant_move(board, &mut move_list);
        generate_knight_moves(board, &mut move_list, capture_mask, push_mask);
        generate_slider_moves(board, &mut move_list, capture_mask, push_mask);
        generate_king_moves(board, &mut move_list, king_capture_mask, king_push_mask);
    } else if checkers.count() == 1 {
        // a single check can be evaded by capturing the checker, blocking the check or by moving the king
        generate_quiet_pawn_moves(board, &mut move_list, push_mask);
        generate_pawn_capture_moves(board, &mut move_list, capture_mask);
        generate_en_passant_move(board, &mut move_list);
        generate_knight_moves(board, &mut move_list, capture_mask, push_mask);
        generate_slider_moves(board, &mut move_list, capture_mask, push_mask);
        // castling is not allowed when the king is in check
        generate_king_moves(board, &mut move_list, king_capture_mask, king_push_mask);
    } else {
        // double and more checkers
        // only the king can move
        generate_king_moves(board, &mut move_list, king_capture_mask, king_push_mask);
    }

    move_list
}

pub fn generate_attack_bitboard(board: &Board, attacking_color: Color) -> BitBoard {
    let mut attacked = BitBoard(0);

    let king = board.pieces(PieceType::King) & board.occupancies(!attacking_color);

    // remove opponent king from blockers to simulate xray attack
    let blockers = board.combined() & !king;

    for piece in ALL_PIECES {
        let piece_bitboard = board.pieces(piece) & board.occupancies(attacking_color);
        for square in piece_bitboard.iter() {
            attacked |= match piece {
                PieceType::Pawn => get_pawn_attacks(square, attacking_color),
                PieceType::Knight => get_knight_attacks(square),
                PieceType::Bishop => get_bishop_attacks(square, blockers),
                PieceType::Rook => get_rook_attacks(square, blockers),
                PieceType::Queen => get_queen_attacks(square, blockers),
                PieceType::King => get_king_attacks(square),
            };
        }
    }

    attacked
}

pub struct PushCaptureMasks {
    pub push_mask: BitBoard,
    pub capture_mask: BitBoard,
}

pub fn compute_push_capture_mask<const CAPTURE_ONLY: bool>(board: &Board) -> PushCaptureMasks {
    let checkers = board.checkers();

    let mut push_mask = BitBoard::FULL;
    let mut capture_mask = BitBoard::FULL;

    // limit captures to the opponent pieces
    capture_mask &= board.occupancies(!board.side_to_move());
    // avoid opponent pieces on quiet moves
    push_mask &= !board.occupancies(!board.side_to_move());

    if checkers.count() == 1 {
        let king_square =
            (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

        capture_mask = checkers;
        push_mask = between(king_square, checkers.bit_scan());
    }

    if CAPTURE_ONLY {
        push_mask &= BitBoard::EMPTY;
    }

    PushCaptureMasks {
        push_mask,
        capture_mask,
    }
}

pub fn compute_king_push_capture_masks<const CAPTURE_ONLY: bool>(
    board: &Board,
) -> PushCaptureMasks {
    let side_to_move = board.side_to_move();

    let attacked = generate_attack_bitboard(board, !side_to_move);

    let mut push_mask = !attacked;
    let mut capture_mask = !attacked;

    // limit captures to the opponent pieces
    capture_mask &= board.occupancies(!side_to_move);
    // avoid opponent pieces on quiet moves
    push_mask &= !board.occupancies(!side_to_move);

    if CAPTURE_ONLY {
        push_mask &= BitBoard::EMPTY;
    }

    PushCaptureMasks {
        push_mask,
        capture_mask,
    }
}

pub fn is_square_attacked(board: &Board, attacked_square: Square, attacking_side: Color) -> bool {
    // attacked by pawns?
    if (get_pawn_attacks(attacked_square, !attacking_side)
        & board.pieces(PieceType::Pawn)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by knight?
    if (get_knight_attacks(attacked_square)
        & board.pieces(PieceType::Knight)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by king?
    if (get_king_attacks(attacked_square)
        & board.pieces(PieceType::King)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by bishop or queen?
    if (get_bishop_attacks(attacked_square, board.combined())
        & (board.pieces(PieceType::Bishop) | board.pieces(PieceType::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by rook or queen?
    if (get_rook_attacks(attacked_square, board.combined())
        & (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen))
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
        MoveList, PushCaptureMasks,
    };
    use crate::types::bitboard::BitBoard;
    use crate::types::chess_move::Move;
    use crate::types::color::Color;
    use crate::types::square::Square;

    pub fn test_move_generator<F, M, const CAPTURES_ONLY: bool>(
        generator: F,
        mask_fn: M,
        fen: &str,
        expected_moves: &[Move],
    ) where
        F: Fn(&Board, &mut MoveList, &PushCaptureMasks),
        M: Fn(&Board) -> PushCaptureMasks,
    {
        let board = Board::from_str(fen).unwrap();
        let mut move_list = MoveList::new();

        let masks = mask_fn(&board);
        generator(&board, &mut move_list, &masks);

        println!("{:#?}", move_list);

        for m in expected_moves {
            assert!(
                move_list.contains(m),
                "Expected move {:?} not found in move list: {:#?}",
                m,
                move_list
            );
        }

        assert_eq!(
            move_list.len(),
            expected_moves.len(),
            "Unexpected number of generated moves: got {}, expected {}.\nMoves: {:#?}",
            move_list.len(),
            expected_moves.len(),
            move_list
        );
    }

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

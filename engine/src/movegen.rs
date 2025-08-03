use arrayvec::ArrayVec;
use castling::generate_castling_moves;
use en_passant::generate_en_passant_move;
use king::generate_king_moves;
use knight::generate_knight_moves;
use pawn_capture::generate_pawn_capture_moves;
use quiet_pawn::generate_quiet_pawn_moves;
use slider::generate_slider_moves;

use crate::Printer;
use crate::board::Board;
use crate::movegen::attacks::{
    between, bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks,
};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::types::piece::PieceType;
use crate::types::square::Square;

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
        king_push_mask,
        king_capture_mask,
        attacked,
    } = compute_masks::<CAPTURE_ONLY>(board);

    if checkers.count() == 0 {
        if !CAPTURE_ONLY {
            generate_quiet_pawn_moves(board, &mut move_list, push_mask);
            generate_castling_moves(board, &mut move_list, attacked);
        }

        generate_pawn_capture_moves(board, &mut move_list, capture_mask);
        generate_en_passant_move(board, &mut move_list, push_mask);
        generate_knight_moves(board, &mut move_list, capture_mask, push_mask);
        generate_slider_moves(board, &mut move_list, capture_mask, push_mask);
        generate_king_moves(board, &mut move_list, king_capture_mask, king_push_mask);
    } else if checkers.count() == 1 {
        // a single check can be evaded by capturing the checker, blocking the check or by moving the king
        generate_quiet_pawn_moves(board, &mut move_list, push_mask);
        generate_pawn_capture_moves(board, &mut move_list, capture_mask);
        generate_en_passant_move(board, &mut move_list, push_mask);
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

pub struct PushCaptureMasks {
    pub push_mask: BitBoard,
    pub capture_mask: BitBoard,
    pub king_push_mask: BitBoard,
    pub king_capture_mask: BitBoard,
    pub attacked: BitBoard,
}

pub fn compute_masks<const CAPTURE_ONLY: bool>(board: &Board) -> PushCaptureMasks {
    let checkers = board.checkers();
    let side_to_move = board.side_to_move();

    // limit captures to the opponent pieces
    let mut capture_mask = board.occupancies(!side_to_move);
    // avoid opponent pieces on quiet moves
    let mut push_mask = !board.occupancies(!side_to_move);

    if checkers.count() == 1 {
        let king_square =
            (board.pieces(PieceType::King) & board.occupancies(side_to_move)).bit_scan();

        capture_mask = checkers;
        push_mask = between(king_square, checkers.bit_scan());
    }

    let attacked = generate_attack_bitboard(board, !side_to_move);

    let mut king_push_mask = !attacked;
    let mut king_capture_mask = !attacked;

    // limit captures to the opponent pieces
    king_capture_mask &= board.occupancies(!side_to_move);
    // avoid opponent pieces on quiet moves
    king_push_mask &= !board.occupancies(!side_to_move);

    if CAPTURE_ONLY {
        push_mask &= BitBoard::EMPTY;
        king_push_mask &= BitBoard::EMPTY;
    }

    PushCaptureMasks {
        push_mask,
        capture_mask,
        king_push_mask,
        king_capture_mask,
        attacked,
    }
}

pub fn sq_attacked(board: &Board, attacked_square: Square, attacking_side: Color) -> bool {
    sq_attacked_given_blockers(board, attacked_square, attacking_side, board.combined())
}

pub fn sq_attacked_given_blockers(
    board: &Board,
    attacked_square: Square,
    attacking_side: Color,
    blockers: BitBoard,
) -> bool {
    // attacked by knight?
    if (knight_attacks(attacked_square)
        & board.pieces(PieceType::Knight)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by king?
    if (king_attacks(attacked_square)
        & board.pieces(PieceType::King)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by pawns?
    if (pawn_attacks(attacked_square, !attacking_side)
        & board.pieces(PieceType::Pawn)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by rook or queen?
    if (rook_attacks(attacked_square, blockers)
        & (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by bishop or queen?
    if (bishop_attacks(attacked_square, blockers)
        & (board.pieces(PieceType::Bishop) | board.pieces(PieceType::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    false
}

pub fn generate_attack_bitboard(board: &Board, attacking_side: Color) -> BitBoard {
    let king_bb = board.pieces(PieceType::King) & board.occupancies(!attacking_side);

    // remove opponent king from blockers to simulate xray attack
    let blockers = board.combined() ^ king_bb;

    let pawns = board.pieces(PieceType::Pawn) & board.occupancies(attacking_side);
    let pawn_attacks = attacks::pawn_attacks_all(pawns, attacking_side);

    let knights = board.pieces(PieceType::Knight) & board.occupancies(attacking_side);
    let knight_attacks = attacks::knight_attacks_all(knights);

    let kings = board.pieces(PieceType::King) & board.occupancies(attacking_side);
    let king_attacks = attacks::king_attacks_all(kings);

    let mut attacked = pawn_attacks | knight_attacks | king_attacks;

    let rook_sliders = (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen))
        & board.occupancies(attacking_side);

    for square in rook_sliders {
        attacked |= rook_attacks(square, blockers);
    }

    let bishop_sliders = (board.pieces(PieceType::Bishop) | board.pieces(PieceType::Queen))
        & board.occupancies(attacking_side);

    for square in bishop_sliders {
        attacked |= bishop_attacks(square, blockers);
    }

    attacked
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
        MoveList, PushCaptureMasks, compute_masks, generate_attack_bitboard, generate_moves,
        sq_attacked,
    };
    use crate::types::bitboard::BitBoard;
    use crate::types::chess_move::Move;
    use crate::types::color::Color;
    use crate::types::square::Square;

    pub fn test_move_generator<F, const CAPTURES_ONLY: bool>(
        generator: F,
        fen: &str,
        expected_moves: &[Move],
    ) where
        F: Fn(&Board, &mut MoveList, &PushCaptureMasks),
    {
        let board = Board::from_str(fen).unwrap();
        let mut move_list = MoveList::new();

        let masks = compute_masks::<CAPTURES_ONLY>(&board);
        generator(&board, &mut move_list, &masks);

        println!("{move_list:#?}");

        for m in expected_moves {
            assert!(
                move_list.contains(m),
                "Expected move {m:?} not found in move list: {move_list:#?}"
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

        let is_e4_attacked_by_black = sq_attacked(&board, Square::E4, Color::Black);

        assert!(is_e4_attacked_by_black);
        println!("is e4 attacked by black pawn? {is_e4_attacked_by_black}",);
    }

    #[test]
    fn test_is_square_attacked_bishop_attack() {
        let board = Board::from_str("7k/8/2P5/8/8/8/6b1/K7 w - - 0 1").unwrap();
        println!("board: {board}");

        let is_c6_attacked_by_black = sq_attacked(&board, Square::C6, Color::Black);

        assert!(is_c6_attacked_by_black);

        println!("is the pawn on c6 attacked by the bishop on g2? {is_c6_attacked_by_black}");
    }

    #[test]
    fn test_is_square_attacked_rook_attack() {
        let board = Board::from_str("7k/8/8/8/2R2b2/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_f4_attacked_by_white = sq_attacked(&board, Square::F4, Color::White);

        assert!(is_f4_attacked_by_white);

        println!("is the bishop on f4 attacked by the rook on c4? {is_f4_attacked_by_white}");
    }

    #[test]
    fn test_is_square_attacked_knight_attack() {
        let board = Board::from_str("7k/8/4q3/8/3N4/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_e6_attacked_by_white = sq_attacked(&board, Square::E6, Color::White);

        assert!(is_e6_attacked_by_white);

        println!("is the queen on e6 attacked by the knight on d4? {is_e6_attacked_by_white}");
    }

    #[test]
    fn test_is_square_attacked_queen_attack() {
        let board = Board::from_str("7k/P7/8/8/8/8/8/K5q1 w - - 0 1").unwrap();
        println!("{board}");

        let is_a7_attacked_by_black = sq_attacked(&board, Square::A7, Color::Black);

        assert!(is_a7_attacked_by_black);

        println!("is the pawn on a7 attacked by the queen on g1? {is_a7_attacked_by_black}");
    }

    #[test]
    fn test_attack_bitboard() {
        let board =
            Board::from_str("2r1k2r/pp3p2/1n2bq1p/2bp2p1/8/2NBP1P1/PPQN1PP1/1KR4R w k - 0 1")
                .unwrap();
        let attacked = generate_attack_bitboard(&board, Color::Black);
        println!("{attacked}");

        let expected = BitBoard(0xfffcdf7cffb52000);
        println!("expected: {expected}");
        assert_eq!(attacked, expected);
    }

    #[test]
    fn moves_at_startpos() {
        let board = Board::default();
        let moves = generate_moves::<false>(&board);

        println!("{moves:#?}");
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn moves_at_kiwipete() {
        let board = Board::from_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        let moves = generate_moves::<false>(&board);
        println!("{moves:#?}");

        assert_eq!(moves.len(), 46);
    }

    #[test]
    fn captures_only() {
        let board = Board::from_str(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        )
        .unwrap();
        let moves = generate_moves::<true>(&board);
        println!("{moves:#?}");

        assert_eq!(moves.len(), 4);
    }
}

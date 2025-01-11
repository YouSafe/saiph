use crate::board::Board;
use crate::move_generation::MoveList;
use crate::tables::{get_pawn_attacks, line};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::Piece;

pub fn generate_pawn_capture_moves<const CHECK: bool>(board: &Board, move_list: &mut MoveList) {
    let mut capture_mask = !BitBoard::EMPTY;

    let side_to_move = board.side_to_move();
    let current_sides_pawns = board.pieces(Piece::Pawn) & board.occupancies(side_to_move);

    let pinned = board.pinned();

    let king_square =
        (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

    if CHECK {
        capture_mask = board.checkers();
    }

    // splitting the loop with an if inside into two for pinned and non-pinned
    // resulted in a ~9% increase in move generation performance
    for source in (current_sides_pawns & pinned).iter() {
        let attacks = get_pawn_attacks(source, side_to_move)
            & board.occupancies(!side_to_move)
            & capture_mask
            & line(king_square, source);

        let promotion_rank = BitBoard::mask_rank((!side_to_move).backrank());

        for target in (attacks & promotion_rank).iter() {
            // fill in promotion moves
            move_list.push(Move::new(source, target, MoveFlag::KnightPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::BishopPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::RookPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::QueenPromotionCapture));
        }

        for target in (attacks & !promotion_rank).iter() {
            // regular pawn capture
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }
    }

    for source in (current_sides_pawns & !pinned).iter() {
        let attacks = get_pawn_attacks(source, side_to_move)
            & board.occupancies(!side_to_move)
            & capture_mask;

        let promotion_rank = BitBoard::mask_rank((!side_to_move).backrank());

        for target in (attacks & promotion_rank).iter() {
            // fill in promotion moves
            move_list.push(Move::new(source, target, MoveFlag::KnightPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::BishopPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::RookPromotionCapture));
            move_list.push(Move::new(source, target, MoveFlag::QueenPromotionCapture));
        }

        for target in (attacks & !promotion_rank).iter() {
            // regular pawn capture
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::move_generation::pawn_capture::generate_pawn_capture_moves;
    use crate::move_generation::MoveList;
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square::*;

    #[test]
    fn capture_pinner() {
        let board = Board::from_str("6k1/8/8/8/8/2b5/1P6/K7 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move::new(B2, C3, MoveFlag::Capture)));
    }

    #[test]
    fn test_capture_promotion() {
        let board = Board::from_str("3b2k1/2P5/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 4);

        assert!(move_list.contains(&Move::new(C7, D8, MoveFlag::BishopPromotionCapture)));
        assert!(move_list.contains(&Move::new(C7, D8, MoveFlag::KnightPromotionCapture)));
        assert!(move_list.contains(&Move::new(C7, D8, MoveFlag::RookPromotionCapture)));
        assert!(move_list.contains(&Move::new(C7, D8, MoveFlag::QueenPromotionCapture)));
    }

    #[test]
    fn test_blocked_capture_by_pin() {
        let board = Board::from_str("6k1/8/8/8/2K1r3/3P4/4q3/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_force_knight_capture() {
        let board = Board::from_str("6k1/8/8/8/2K5/4n1q1/3P3P/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<true>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move::new(D2, E3, MoveFlag::Capture)));
    }

    #[test]
    fn test_multiple_captures() {
        let board = Board::from_str("6k1/8/8/8/2K5/2p1p3/3P4/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 2);

        assert!(move_list.contains(&Move::new(D2, C3, MoveFlag::Capture)));
        assert!(move_list.contains(&Move::new(D2, E3, MoveFlag::Capture)));
    }

    #[test]
    fn test_capture_own_pawn() {
        let board = Board::from_str("8/8/k7/8/8/2N1P3/3P4/3K4 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_capture_with_pinned_pawn() {
        let board = Board::from_str("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_pawn_capture_moves::<true>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert!(!move_list.contains(&Move::new(F4, G3, MoveFlag::Capture)));
    }
}

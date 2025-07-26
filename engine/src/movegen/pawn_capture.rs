use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::{line, pawn_attacks};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::PieceType;

pub fn generate_pawn_capture_moves(
    board: &Board,
    move_list: &mut MoveList,
    capture_mask: BitBoard,
) {
    let side_to_move = board.side_to_move();
    let current_sides_pawns = board.pieces(PieceType::Pawn) & board.occupancies(side_to_move);

    let pinned = board.pinned();

    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    // splitting the loop with an if inside into two for pinned and non-pinned
    // resulted in a ~9% increase in move generation performance
    for source in (current_sides_pawns & pinned).iter() {
        let attacks = pawn_attacks(source, side_to_move)
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
        let attacks =
            pawn_attacks(source, side_to_move) & board.occupancies(!side_to_move) & capture_mask;

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
    use crate::board::Board;
    use crate::movegen::pawn_capture::generate_pawn_capture_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks, compute_push_capture_mask};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square::*;

    fn test_pawn_capture_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_pawn_capture_moves(board, moves_list, masks.capture_mask)
            },
            compute_push_capture_mask::<false>,
            fen,
            expected_moves,
        )
    }

    #[test]
    fn capture_pinner() {
        test_pawn_capture_moves(
            "6k1/8/8/8/8/2b5/1P6/K7 w - - 0 1",
            &[Move::new(B2, C3, MoveFlag::Capture)],
        );
    }

    #[test]
    fn test_capture_promotion() {
        test_pawn_capture_moves(
            "3b2k1/2P5/8/8/8/8/8/K7 w - - 0 1",
            &[
                Move::new(C7, D8, MoveFlag::BishopPromotionCapture),
                Move::new(C7, D8, MoveFlag::KnightPromotionCapture),
                Move::new(C7, D8, MoveFlag::RookPromotionCapture),
                Move::new(C7, D8, MoveFlag::QueenPromotionCapture),
            ],
        );
    }

    #[test]
    fn test_blocked_capture_by_pin() {
        test_pawn_capture_moves("6k1/8/8/8/2K1r3/3P4/4q3/8 w - - 0 1", &[]);
    }

    #[test]
    fn test_force_knight_capture() {
        test_pawn_capture_moves(
            "6k1/8/8/8/2K5/4n1q1/3P3P/8 w - - 0 1",
            &[Move::new(D2, E3, MoveFlag::Capture)],
        );
    }

    #[test]
    fn test_multiple_captures() {
        test_pawn_capture_moves(
            "6k1/8/8/8/2K5/2p1p3/3P4/8 w - - 0 1",
            &[
                Move::new(D2, C3, MoveFlag::Capture),
                Move::new(D2, E3, MoveFlag::Capture),
            ],
        );
    }

    #[test]
    fn test_capture_own_pawn() {
        test_pawn_capture_moves("8/8/k7/8/8/2N1P3/3P4/3K4 w - - 0 1", &[]);
    }

    #[test]
    fn test_capture_with_pinned_pawn() {
        test_pawn_capture_moves("8/8/8/K7/1R3p1k/6P1/8/8 b - - 0 1", &[]);
    }
}

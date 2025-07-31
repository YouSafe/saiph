use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::knight_attacks;
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::PieceType;

pub fn generate_knight_moves(
    board: &Board,
    move_list: &mut MoveList,
    capture_mask: BitBoard,
    push_mask: BitBoard,
) {
    let side_to_move = board.side_to_move();
    let current_sides_knights = board.pieces(PieceType::Knight) & board.occupancies(side_to_move);

    let pinned = board.pinned();

    // pinned knights can't move at all
    for source in (current_sides_knights & !pinned).into_iter() {
        let attacks = knight_attacks(source) & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).into_iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture))
        }

        // quiet
        for target in (attacks & push_mask).into_iter() {
            move_list.push(Move::new(source, target, MoveFlag::Normal));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::knight::generate_knight_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks, compute_push_capture_mask};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square::*;

    fn test_knight_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_knight_moves(board, moves_list, masks.capture_mask, masks.push_mask)
            },
            compute_push_capture_mask::<false>,
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_check_evasion() {
        test_knight_moves(
            "4k2n/8/6n1/4R3/8/8/8/K7 b - - 0 1",
            &[
                Move::new(G6, E5, MoveFlag::Capture),
                Move::new(G6, E7, MoveFlag::Normal),
            ],
        );
    }

    #[test]
    fn test_self_capture_prevention() {
        test_knight_moves("4kr1n/4rr2/6n1/4r3/5r1r/8/8/K7 b - - 0 1", &[]);
    }

    #[test]
    fn test_pinned_knight_can_not_move() {
        test_knight_moves("4k3/8/4n3/8/8/8/8/K3R3 b - - 0 1", &[]);
    }

    #[test]
    fn test_knight_does_not_capture_empty_squares() {
        test_knight_moves(
            "3pkp2/2p3p1/4n3/2p3p1/3p4/8/8/K7 b - - 0 1",
            &[Move::new(E6, F4, MoveFlag::Normal)],
        );
    }

    #[test]
    fn test_capture_marked_as_quiet() {
        test_knight_moves(
            "3BkB2/2P3P1/4n3/2P3P1/3P4/8/8/K7 b - - 0 1",
            &[
                Move::new(E6, D8, MoveFlag::Capture),
                Move::new(E6, C7, MoveFlag::Capture),
                Move::new(E6, C5, MoveFlag::Capture),
                Move::new(E6, D4, MoveFlag::Capture),
                Move::new(E6, F4, MoveFlag::Normal),
                Move::new(E6, G5, MoveFlag::Capture),
                Move::new(E6, G7, MoveFlag::Capture),
                Move::new(E6, F8, MoveFlag::Capture),
            ],
        );
    }
}

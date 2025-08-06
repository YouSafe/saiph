use crate::board::Board;
use crate::movegen::attacks::pawn_attacks;
use crate::movegen::{MoveList, MoveListExt};
use crate::types::chess_move::MoveFlag;
use crate::types::piece::PieceType;

pub fn generate_en_passant_move(board: &Board, move_list: &mut MoveList) {
    if let Some(ep_square) = board.en_passant_target() {
        let side_to_move = board.side_to_move();

        let king_square =
            (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

        let king_diagonals = king_square.main_diagonal() | king_square.anti_diagonal();
        let dest_safe_mask = king_diagonals.contains_mask(ep_square);

        let pawns = board.pieces(PieceType::Pawn) & board.occupancies(side_to_move);
        let pinned = board.pinned();
        let potential_sources = pawn_attacks(ep_square, !side_to_move) & pawns;
        let sources = potential_sources & (!pinned | (pinned & king_diagonals & dest_safe_mask));

        for source in sources.into_iter() {
            move_list.push_move(source, ep_square, MoveFlag::EnPassant);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::en_passant::generate_en_passant_move;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_en_passant_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, false>(
            |board: &Board, moves_list: &mut MoveList, _masks: &PushCaptureMasks| {
                generate_en_passant_move(board, moves_list)
            },
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_valid_en_passant() {
        test_en_passant_moves(
            "8/8/k7/8/2Pp4/8/8/3K4 b - c3 0 1",
            &[Move::new(Square::D4, Square::C3, MoveFlag::EnPassant)],
        );
    }

    #[test]
    fn test_two_en_passant_moves() {
        test_en_passant_moves(
            "8/8/k7/8/1pPp4/8/8/3K4 b - c3 0 1",
            &[
                Move::new(Square::B4, Square::C3, MoveFlag::EnPassant),
                Move::new(Square::D4, Square::C3, MoveFlag::EnPassant),
            ],
        );
    }

    #[test]
    fn test_invalid_en_passant_horizontal() {
        test_en_passant_moves("8/8/8/8/k1Pp3R/8/8/3K4 b - c3 0 1", &[]);
    }

    #[test]
    fn test_invalid_en_passant_vertical() {
        test_en_passant_moves("5q2/8/8/4pP2/8/8/8/5K2 w - e6 0 1", &[]);
    }

    #[test]
    fn test_invalid_en_passant_diagonal() {
        test_en_passant_moves("8/7q/8/4pP2/8/8/8/1K6 w - e6 0 1", &[]);
    }

    #[test]
    fn test_en_passant_edge() {
        test_en_passant_moves(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1",
            &[Move::new(Square::B4, Square::A3, MoveFlag::EnPassant)],
        );
    }

    #[test]
    fn test_en_passant_in_check() {
        test_en_passant_moves("1kb5/p7/P7/2Ppb2B/7P/7K/8/8 w - d6 0 4", &[]);
    }

    #[test]
    fn test_en_passsant_block_check() {
        // En passant can not block checks
        test_en_passant_moves("k7/7q/8/6pP/8/8/8/1K6 w - g6 0 1", &[]);
    }

    #[test]
    fn test_target_but_no_capturer() {
        test_en_passant_moves("k7/8/8/8/6P1/8/8/K7 b - g3 0 1", &[]);
    }

    #[test]
    fn test_capture_checker() {
        test_en_passant_moves(
            "8/8/8/4kp1p/5PpP/3B2K1/8/8 b - f3 0 1",
            &[Move::new(Square::G4, Square::F3, MoveFlag::EnPassant)],
        );
    }
}

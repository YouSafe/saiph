use crate::board::Board;
use crate::movegen::attacks::{bishop_attacks, rook_attacks};
use crate::movegen::{MoveList, MoveListExt};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::MoveFlag;
use crate::types::piece::PieceType;

pub fn generate_slider_moves(
    board: &Board,
    move_list: &mut MoveList,
    capture_mask: BitBoard,
    push_mask: BitBoard,
) {
    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    let side_to_move = board.side_to_move();

    let pinned = board.pinned();

    let bishops = board.pieces(PieceType::Bishop) & board.occupancies(side_to_move);
    let rooks = board.pieces(PieceType::Rook) & board.occupancies(side_to_move);
    let queens = board.pieces(PieceType::Queen) & board.occupancies(side_to_move);

    let combined = board.combined();
    let non_friendly_squares = !board.occupancies(side_to_move);

    // diagonal attackers
    for source in ((bishops | queens) & !pinned).into_iter() {
        let attacks = bishop_attacks(source, combined) & non_friendly_squares;

        // captures
        for target in (attacks & capture_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Capture);
        }

        // quiet
        for target in (attacks & push_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Normal);
        }
    }

    for source in ((bishops | queens) & pinned).into_iter() {
        // At most 4 pieces can be diagonally pinned to the king.
        // Since this is a cold path, recomputing the king's diagonals in each
        // iteration is faster than prefiltering sources that lie on a
        // diagonal with the king.
        let king_diagonals = king_square.anti_diagonal() | king_square.main_diagonal();
        let src_safe_mask = king_diagonals.contains_mask(source);
        let attacks = bishop_attacks(source, combined)
            & king_diagonals
            & non_friendly_squares
            & src_safe_mask;

        // captures
        for target in (attacks & capture_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Capture);
        }

        // quiet
        for target in (attacks & push_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Normal);
        }
    }

    // orthogonal attackers
    for source in ((rooks | queens) & !pinned).into_iter() {
        let attacks = rook_attacks(source, combined) & non_friendly_squares;

        // captures
        for target in (attacks & capture_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Capture)
        }

        // quiet
        for target in (attacks & push_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Normal);
        }
    }

    for source in ((rooks | queens) & pinned).into_iter() {
        // At most 4 pieces can be orthogonally pinned to the king.
        // Since this is such a path, recomputing the king's rank and file
        // in each iteration is faster than prefiltering sources that lie on
        // the same rank or file.
        let king_orthogonals = king_square.file().mask() | king_square.rank().mask();
        let src_safe_mask = king_orthogonals.contains_mask(source);
        let attacks = rook_attacks(source, combined)
            & king_orthogonals
            & src_safe_mask
            & non_friendly_squares;

        // captures
        for target in (attacks & capture_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Capture)
        }

        // quiet
        for target in (attacks & push_mask).into_iter() {
            move_list.push_move(source, target, MoveFlag::Normal);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::slider::generate_slider_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_slider_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_slider_moves(board, moves_list, masks.capture_mask, masks.push_mask)
            },
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_move_along_pin_ray() {
        test_slider_moves(
            "4k3/8/7b/3P4/8/8/3B4/2K5 w - - 3 2",
            &[
                Move::new(Square::D2, Square::E3, MoveFlag::Normal),
                Move::new(Square::D2, Square::F4, MoveFlag::Normal),
                Move::new(Square::D2, Square::G5, MoveFlag::Normal),
                Move::new(Square::D2, Square::H6, MoveFlag::Capture),
            ],
        );
    }

    #[test]
    fn test_pinned_bishop_captures() {
        test_slider_moves("8/2p5/3p4/KP5r/1R3b1k/6P1/4P3/8 b - - 0 1", &[]);
    }

    #[test]
    fn test_pinned_rook_captures() {
        test_slider_moves("8/2p5/3p4/KP5r/1R4rk/6P1/4P3/8 b - - 0 1", &[]);
    }

    #[test]
    fn test_pinned_queen_captures() {
        test_slider_moves("8/2p5/3p4/KP5r/1R4qk/6P1/4P3/8 b - - 0 1", &[]);
    }

    #[test]
    fn test_wtf() {
        let mut kiwi = <Board as std::str::FromStr>::from_str(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap();

        let boards = [
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/1R2K2R b Kkq - 1 1",
            "r3k2r/p1ppqpb1/1n2pnp1/1b1PN3/1p2P3/2N2Q1p/PPPBBPPP/1R2K2R w Kkq - 2 2",
            "r3k2r/p1ppqpb1/1n2pnp1/1B1PN3/1p2P3/2N2Q1p/PPPB1PPP/1R2K2R b Kkq - 0 2",
            "r3k2r/2ppqpb1/1n2pnp1/pB1PN3/1p2P3/2N2Q1p/PPPB1PPP/1R2K2R w Kkq - 0 3",
        ]
        .map(|fen| <Board as std::str::FromStr>::from_str(fen).unwrap());

        let moves = ["a1b1", "a6b5", "e2b5", "a7a5"]
            .map(|s| <crate::types::uci_move::UCIMove as std::str::FromStr>::from_str(s).unwrap());
        for (i, mv) in moves.iter().enumerate() {
            println!("{i}: {mv:?}");
            let chess_move = kiwi
                .generate_moves()
                .into_iter()
                .find(|m| *mv == m)
                .unwrap();
            kiwi.apply_move(chess_move);
            assert_eq!(boards[i].hash(), kiwi.hash());
        }

        let expected = <Board as std::str::FromStr>::from_str(
            "r3k2r/2ppqpb1/1n2pnp1/pB1PN3/1p2P3/2N2Q1p/PPPB1PPP/1R2K2R w Kkq - 0 3",
        )
        .unwrap();
        assert_eq!(kiwi.generate_moves(), expected.generate_moves());
    }
}

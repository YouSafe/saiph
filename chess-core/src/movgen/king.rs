use crate::board::Board;
use crate::chess_move::{Move, MoveFlag};
use crate::movgen::{generate_attack_bitboard, CheckState, MoveList, PieceMoveGenerator};
use crate::piece::Piece;
use crate::tables::get_king_attacks;

pub struct KingMoveGenerator;

impl PieceMoveGenerator for KingMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        let attacked = generate_attack_bitboard(board, !board.side_to_move());

        let mut capture_mask = !attacked;
        let mut push_mask = !attacked;

        let king_square =
            (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

        let side_to_move = board.side_to_move();

        // limit captures to the opponent pieces
        capture_mask &= board.occupancies(!side_to_move);
        // avoid opponent pieces on quiet moves
        push_mask &= !*board.occupancies(!side_to_move);

        // quiet
        for target in (get_king_attacks(king_square) & push_mask).iter() {
            move_list.push(Move {
                from: king_square,
                to: target,
                promotion: None,
                piece: Piece::King,
                flags: MoveFlag::Normal,
            });
        }

        // capture
        for target in (get_king_attacks(king_square) & capture_mask).iter() {
            move_list.push(Move {
                from: king_square,
                to: target,
                promotion: None,
                piece: Piece::King,
                flags: MoveFlag::Capture,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::chess_move::{Move, MoveFlag};
    use crate::movgen::king::KingMoveGenerator;
    use crate::movgen::{InCheck, PieceMoveGenerator};
    use crate::piece::Piece;
    use crate::square::Square;
    use std::str::FromStr;

    #[test]
    fn test_xray_attack() {
        let board = Board::from_str("8/4k3/8/8/8/4R3/8/K7 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KingMoveGenerator::generate::<InCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 6);
        assert!(!move_list.contains(&Move {
            from: Square::E7,
            to: Square::E8,
            promotion: None,
            piece: Piece::King,
            flags: MoveFlag::Normal,
        }));
    }

    #[test]
    fn test_forced_capture() {
        let board = Board::from_str("6Qk/8/8/8/8/2q5/8/1K6 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KingMoveGenerator::generate::<InCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);
        assert!(move_list.contains(&Move {
            from: Square::H8,
            to: Square::G8,
            promotion: None,
            piece: Piece::King,
            flags: MoveFlag::Capture,
        }))
    }
}

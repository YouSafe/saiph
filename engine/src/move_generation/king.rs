use crate::board::Board;
use crate::move_generation::{generate_attack_bitboard, MoveList};
use crate::tables::get_king_attacks;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::Piece;

pub fn generate_king_moves<const CHECK: bool>(board: &Board, move_list: &mut MoveList) {
    let attacked = generate_attack_bitboard(board, !board.side_to_move());

    let mut capture_mask = !attacked;
    let mut push_mask = !attacked;

    let king_square =
        (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

    let side_to_move = board.side_to_move();

    // limit captures to the opponent pieces
    capture_mask &= board.occupancies(!side_to_move);
    // avoid opponent pieces on quiet moves
    push_mask &= !board.occupancies(!side_to_move);

    let attacks = get_king_attacks(king_square) & !board.occupancies(side_to_move);

    // quiet
    for target in (attacks & push_mask).iter() {
        move_list.push(Move {
            from: king_square,
            to: target,
            promotion: None,
            piece: Piece::King,
            flags: MoveFlag::Normal,
        });
    }

    // capture
    for target in (attacks & capture_mask).iter() {
        move_list.push(Move {
            from: king_square,
            to: target,
            promotion: None,
            piece: Piece::King,
            flags: MoveFlag::Capture,
        });
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::move_generation::king::generate_king_moves;
    use crate::move_generation::MoveList;
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::piece::Piece;
    use crate::types::square::Square;

    #[test]
    fn test_xray_attack() {
        let board = Board::from_str("8/4k3/8/8/8/4R3/8/K7 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_king_moves::<true>(&board, &mut move_list);
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
        let mut move_list = MoveList::new();
        generate_king_moves::<true>(&board, &mut move_list);
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

    #[test]
    fn test_checkmate() {
        let board = Board::from_str("3Q2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_king_moves::<true>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 0);
    }
}

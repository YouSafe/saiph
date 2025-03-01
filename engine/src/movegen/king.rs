use crate::board::Board;
use crate::movegen::attacks::get_king_attacks;
use crate::movegen::{generate_attack_bitboard, MoveList};
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::Piece;

pub fn generate_king_moves<const CHECK: bool, const CAPTURE_ONLY: bool>(
    board: &Board,
    move_list: &mut MoveList,
) {
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

    if !CAPTURE_ONLY {
        // quiet
        for target in (attacks & push_mask).iter() {
            move_list.push(Move::new(king_square, target, MoveFlag::Normal));
        }
    }

    // capture
    for target in (attacks & capture_mask).iter() {
        move_list.push(Move::new(king_square, target, MoveFlag::Capture));
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::movegen::king::generate_king_moves;
    use crate::movegen::MoveList;
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    #[test]
    fn test_xray_attack() {
        let board = Board::from_str("8/4k3/8/8/8/4R3/8/K7 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_king_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 6);
        assert!(!move_list.contains(&Move::new(Square::E7, Square::E8, MoveFlag::Normal)));
        assert!(!move_list.contains(&Move::new(Square::E7, Square::E6, MoveFlag::Normal)));

        assert!(move_list.contains(&Move::new(Square::E7, Square::F6, MoveFlag::Normal)));
        assert!(move_list.contains(&Move::new(Square::E7, Square::F7, MoveFlag::Normal)));
        assert!(move_list.contains(&Move::new(Square::E7, Square::F8, MoveFlag::Normal)));

        assert!(move_list.contains(&Move::new(Square::E7, Square::D6, MoveFlag::Normal)));
        assert!(move_list.contains(&Move::new(Square::E7, Square::D7, MoveFlag::Normal)));
        assert!(move_list.contains(&Move::new(Square::E7, Square::D8, MoveFlag::Normal)));
    }

    #[test]
    fn test_forced_capture() {
        let board = Board::from_str("6Qk/8/8/8/8/2q5/8/1K6 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_king_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);
        assert!(move_list.contains(&Move::new(Square::H8, Square::G8, MoveFlag::Capture)));
    }

    #[test]
    fn test_checkmate() {
        let board = Board::from_str("3Q2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_king_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 0);
    }
}

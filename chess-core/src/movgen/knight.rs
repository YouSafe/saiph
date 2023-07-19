use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::chess_move::{Move, MoveFlag};
use crate::movgen::{CheckState, InCheck, MoveList, PieceMoveGenerator};
use crate::piece::Piece;
use crate::tables::{between, get_knight_attacks};
use std::any::TypeId;

pub struct KnightMoveGenerator;

impl PieceMoveGenerator for KnightMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        let mut capture_mask = !BitBoard::EMPTY;
        let mut push_mask = !BitBoard::EMPTY;

        let king_square =
            (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

        if TypeId::of::<T>() == TypeId::of::<InCheck>() {
            let checkers = board.checkers();
            let checker = checkers.bit_scan();

            capture_mask = checkers;
            push_mask = between(king_square, checker);
        }

        let side_to_move = board.side_to_move();
        let current_sides_knights = board.pieces(Piece::Knight) & board.occupancies(side_to_move);

        let pinned = board.pinned();

        // limit captures to the opponent pieces
        capture_mask &= board.occupancies(!side_to_move);
        // avoid opponent pieces on quiet moves
        push_mask &= !*board.occupancies(!side_to_move);

        // pinned knights can't move at all
        for source in (current_sides_knights & !pinned).iter() {
            let attacks = get_knight_attacks(source) & !*board.occupancies(side_to_move);

            // captures
            for target in (attacks & capture_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: Piece::Knight,
                    flags: MoveFlag::Capture,
                })
            }

            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: Piece::Knight,
                    flags: MoveFlag::Normal,
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::chess_move::{Move, MoveFlag};
    use crate::movgen::knight::KnightMoveGenerator;
    use crate::movgen::{InCheck, NotInCheck, PieceMoveGenerator};
    use crate::piece::Piece;
    use crate::square::Square::*;
    use std::str::FromStr;

    #[test]
    fn test_check_evasion() {
        let board = Board::from_str("4k2n/8/6n1/4R3/8/8/8/K7 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KnightMoveGenerator::generate::<InCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 2);
        assert!(move_list.contains(&Move {
            from: G6,
            to: E5,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Capture,
        }));
        assert!(move_list.contains(&Move {
            from: G6,
            to: E7,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Normal,
        }));
    }

    #[test]
    fn test_self_capture_prevention() {
        let board = Board::from_str("4k2n/8/6n1/8/8/8/8/K7 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KnightMoveGenerator::generate::<NotInCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert!(!move_list.contains(&Move {
            from: G6,
            to: H8,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Capture,
        }));
        assert!(!move_list.contains(&Move {
            from: H8,
            to: G6,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Capture,
        }));
    }

    #[test]
    fn test_pinned_knight_can_not_move() {
        let board = Board::from_str("4k3/8/4n3/8/8/8/8/K3R3 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KnightMoveGenerator::generate::<NotInCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_capture_empty_square() {
        let board = Board::from_str("3pkp2/2p3p1/4n3/2p3p1/3p4/8/8/K7 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KnightMoveGenerator::generate::<NotInCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 1);
        assert!(move_list.contains(&Move {
            from: E6,
            to: F4,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Normal,
        }));
    }

    #[test]
    fn test_capture_marked_as_quiet() {
        let board = Board::from_str("3BkB2/2P3P1/4n3/2P3P1/3P4/8/8/K7 b - - 0 1").unwrap();
        let mut move_list = vec![];
        KnightMoveGenerator::generate::<NotInCheck>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 8);

        assert!(!move_list.contains(&Move {
            from: E6,
            to: D4,
            promotion: None,
            piece: Piece::Knight,
            flags: MoveFlag::Normal,
        }))
    }
}

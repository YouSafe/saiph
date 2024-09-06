use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::chess_move::{Move, MoveFlag};
use crate::movgen::{MoveList, PieceMoveGenerator};
use crate::piece::Piece;
use crate::promotion::ALL_PROMOTIONS;
use crate::tables::{get_pawn_attacks, line};

pub struct PawnCaptureMoveGenerator;

impl PieceMoveGenerator for PawnCaptureMoveGenerator {
    fn generate<const CHECK: bool>(board: &Board, move_list: &mut MoveList) {
        let mut capture_mask = !BitBoard::EMPTY;

        let side_to_move = board.side_to_move();
        let current_sides_pawns = board.pieces(Piece::Pawn) & board.occupancies(side_to_move);

        let pinned = board.pinned();

        let king_square =
            (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

        if CHECK {
            capture_mask = board.checkers();
        }

        for source in current_sides_pawns.iter() {
            let capture_mask = if pinned.contains(source) {
                capture_mask & line(king_square, source)
            } else {
                capture_mask
            };
            let attacks = get_pawn_attacks(source, side_to_move)
                & board.occupancies(!side_to_move)
                & capture_mask;
            for target in attacks.iter() {
                if target.on_promotion_rank(side_to_move) {
                    // fill in promotion moves
                    for promotion in ALL_PROMOTIONS {
                        move_list.push(Move {
                            from: source,
                            to: target,
                            piece: Piece::Pawn,
                            promotion: Some(promotion),
                            flags: MoveFlag::Capture,
                        });
                    }
                } else {
                    // regular pawn capture
                    move_list.push(Move {
                        from: source,
                        to: target,
                        promotion: None,
                        piece: Piece::Pawn,
                        flags: MoveFlag::Capture,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::chess_move::{Move, MoveFlag};
    use crate::movgen::pawn_capture::PawnCaptureMoveGenerator;
    use crate::movgen::{MoveList, PieceMoveGenerator};
    use crate::piece::Piece;
    use crate::promotion::ALL_PROMOTIONS;
    use crate::square::Square::*;

    #[test]
    fn capture_pinner() {
        let board = Board::from_str("6k1/8/8/8/8/2b5/1P6/K7 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move {
            from: B2,
            to: C3,
            promotion: None,
            piece: Piece::Pawn,
            flags: MoveFlag::Capture,
        }));
    }

    #[test]
    fn test_capture_promotion() {
        let board = Board::from_str("3b2k1/2P5/8/8/8/8/8/K7 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 4);

        for promotion in ALL_PROMOTIONS {
            assert!(move_list.contains(&Move {
                from: C7,
                to: D8,
                promotion: Some(promotion),
                piece: Piece::Pawn,
                flags: MoveFlag::Capture,
            }));
        }
    }

    #[test]
    fn test_blocked_capture_by_pin() {
        let board = Board::from_str("6k1/8/8/8/2K1r3/3P4/4q3/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_force_knight_capture() {
        let board = Board::from_str("6k1/8/8/8/2K5/4n1q1/3P3P/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<true>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move {
            from: D2,
            to: E3,
            promotion: None,
            piece: Piece::Pawn,
            flags: MoveFlag::Capture,
        }));
    }

    #[test]
    fn test_multiple_captures() {
        let board = Board::from_str("6k1/8/8/8/2K5/2p1p3/3P4/8 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 2);

        assert!(move_list.contains(&Move {
            from: D2,
            to: C3,
            promotion: None,
            piece: Piece::Pawn,
            flags: MoveFlag::Capture,
        }));

        assert!(move_list.contains(&Move {
            from: D2,
            to: E3,
            promotion: None,
            piece: Piece::Pawn,
            flags: MoveFlag::Capture,
        }));
    }

    #[test]
    fn test_capture_own_pawn() {
        let board = Board::from_str("8/8/k7/8/8/2N1P3/3P4/3K4 w - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_capture_with_pinned_pawn() {
        let board = Board::from_str("8/2p5/3p4/KP5r/1R3p1k/6P1/4P3/8 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        PawnCaptureMoveGenerator::generate::<true>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert!(!move_list.contains(&Move {
            from: F4,
            to: G3,
            promotion: None,
            piece: Piece::Pawn,
            flags: MoveFlag::Capture,
        }));
    }
}

use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::chess_move::{Move, MoveFlag};
use crate::movgen::{CheckState, InCheck, MoveList, PieceMoveGenerator};
use crate::piece::Piece;
use crate::tables::{between, get_bishop_attacks, get_rook_attacks, line};
use std::any::TypeId;

pub struct SliderMoveGenerator;

impl PieceMoveGenerator for SliderMoveGenerator {
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

        let pinned = board.pinned();

        // limit captures to the opponent pieces
        capture_mask &= board.occupancies(!side_to_move);
        // avoid opponent pieces on quiet moves
        push_mask &= !*board.occupancies(!side_to_move);

        let bishops = board.pieces(Piece::Bishop) & board.occupancies(side_to_move);
        let rooks = board.pieces(Piece::Rook) & board.occupancies(side_to_move);
        let queens = board.pieces(Piece::Queen) & board.occupancies(side_to_move);

        let combined = *board.combined();

        // TODO: refactor to avoid code duplication

        // diagonal attackers
        for source in ((bishops | queens) & !pinned).iter() {
            let attacks = get_bishop_attacks(source, combined) & !*board.occupancies(side_to_move);

            let source_piece = board.piece_on_square(source).unwrap();

            // captures
            for target in (attacks & capture_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Capture,
                })
            }

            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Normal,
                });
            }
        }

        for source in ((bishops | queens) & pinned).iter() {
            let attacks = get_bishop_attacks(source, combined)
                & line(source, king_square)
                & !*board.occupancies(side_to_move);

            let source_piece = board.piece_on_square(source).unwrap();

            // captures
            for target in (attacks & capture_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Capture,
                })
            }

            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Normal,
                });
            }
        }

        // line attackers
        for source in ((rooks | queens) & !pinned).iter() {
            let attacks = get_rook_attacks(source, combined) & !*board.occupancies(side_to_move);

            let source_piece = board.piece_on_square(source).unwrap();

            // captures
            for target in (attacks & capture_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Capture,
                })
            }

            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Normal,
                });
            }
        }

        for source in ((rooks | queens) & pinned).iter() {
            let attacks = get_rook_attacks(source, combined)
                & line(source, king_square)
                & !*board.occupancies(side_to_move);

            let source_piece = board.piece_on_square(source).unwrap();

            // captures
            for target in (attacks & capture_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Capture,
                })
            }

            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move {
                    from: source,
                    to: target,
                    promotion: None,
                    piece: source_piece,
                    flags: MoveFlag::Normal,
                });
            }
        }
    }
}

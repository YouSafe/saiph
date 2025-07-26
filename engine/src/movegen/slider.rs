use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::{bishop_attacks, line, rook_attacks};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
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

    // TODO: refactor to avoid code duplication

    // diagonal attackers
    for source in ((bishops | queens) & !pinned).iter() {
        let attacks = bishop_attacks(source, combined) & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }

        // quiet
        for target in (attacks & push_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Normal));
        }
    }

    for source in ((bishops | queens) & pinned).iter() {
        let attacks = bishop_attacks(source, combined)
            & line(king_square, source)
            & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }

        // quiet
        for target in (attacks & push_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Normal));
        }
    }

    // line attackers
    for source in ((rooks | queens) & !pinned).iter() {
        let attacks = rook_attacks(source, combined) & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture))
        }

        // quiet
        for target in (attacks & push_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Normal));
        }
    }

    for source in ((rooks | queens) & pinned).iter() {
        let attacks = rook_attacks(source, combined)
            & line(king_square, source)
            & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture))
        }

        // quiet
        for target in (attacks & push_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Normal));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::slider::generate_slider_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks, compute_push_capture_mask};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_slider_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_slider_moves(board, moves_list, masks.capture_mask, masks.push_mask)
            },
            compute_push_capture_mask::<false>,
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
}

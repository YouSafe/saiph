use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::king_attacks;
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::PieceType;

pub fn generate_king_moves(
    board: &Board,
    move_list: &mut MoveList,
    capture_mask: BitBoard,
    push_mask: BitBoard,
) {
    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    let side_to_move = board.side_to_move();

    let attacks = king_attacks(king_square) & !board.occupancies(side_to_move);

    // quiet
    for target in (attacks & push_mask).into_iter() {
        move_list.push(Move::new(king_square, target, MoveFlag::Normal));
    }

    // capture
    for target in (attacks & capture_mask).into_iter() {
        move_list.push(Move::new(king_square, target, MoveFlag::Capture));
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::king::generate_king_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_king_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_king_moves(
                    board,
                    moves_list,
                    masks.king_capture_mask,
                    masks.king_push_mask,
                )
            },
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_xray_attack() {
        test_king_moves(
            "8/4k3/8/8/8/4R3/8/K7 b - - 0 1",
            &[
                Move::new(Square::E7, Square::F6, MoveFlag::Normal),
                Move::new(Square::E7, Square::F7, MoveFlag::Normal),
                Move::new(Square::E7, Square::F8, MoveFlag::Normal),
                Move::new(Square::E7, Square::D6, MoveFlag::Normal),
                Move::new(Square::E7, Square::D7, MoveFlag::Normal),
                Move::new(Square::E7, Square::D8, MoveFlag::Normal),
            ],
        );
    }

    #[test]
    fn test_forced_capture() {
        test_king_moves(
            "6Qk/8/8/8/8/2q5/8/1K6 b - - 0 1",
            &[Move::new(Square::H8, Square::G8, MoveFlag::Capture)],
        );
    }

    #[test]
    fn test_checkmate() {
        test_king_moves("3Q2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1", &[]);
    }
}

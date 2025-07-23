use crate::board::Board;
use crate::movegen::attacks::{bishop_attacks, pawn_attacks, rook_attacks};
use crate::movegen::MoveList;
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::PieceType;
use crate::types::square::Square;

fn is_valid_ep(board: &Board, capture: Square, source: Square, destination: Square) -> bool {
    // create combined bitboard of board with both source and capture removed.
    // removing the squares simulates the move
    let combined =
        board.combined() & !BitBoard::from_square(capture) & !BitBoard::from_square(source)
            | BitBoard::from_square(destination);

    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    let mut attack = BitBoard(0);

    // pretend like the king is a rook
    attack |= rook_attacks(king_square, combined)
        & (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen))
        & board.occupancies(!board.side_to_move());

    // pretend like the king is a bishop
    attack |= bishop_attacks(king_square, combined)
        & (board.pieces(PieceType::Bishop) | board.pieces(PieceType::Queen))
        & board.occupancies(!board.side_to_move());

    attack == BitBoard::EMPTY
}

pub fn generate_en_passant_move(board: &Board, move_list: &mut MoveList) {
    if let Some(ep_square) = board.en_passant_target() {
        let side_to_move = board.side_to_move();
        let current_sides_pawns = board.pieces(PieceType::Pawn) & board.occupancies(side_to_move);

        for source in current_sides_pawns.iter() {
            let attack = pawn_attacks(source, side_to_move) & BitBoard::from_square(ep_square);

            for destination in attack.iter() {
                let capture = destination.forward(!side_to_move).unwrap();

                if is_valid_ep(board, capture, source, destination) {
                    move_list.push(Move::new(source, destination, MoveFlag::EnPassant));
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::en_passant::generate_en_passant_move;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{compute_push_capture_mask, MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_en_passant_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, _masks: &PushCaptureMasks| {
                generate_en_passant_move(board, moves_list)
            },
            compute_push_capture_mask::<false>,
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
}

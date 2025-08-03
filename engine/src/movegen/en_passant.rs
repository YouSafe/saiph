use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::slider_horizontal;
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::color::Color;
use crate::types::direction::RelativeDir;
use crate::types::piece::PieceType;
use crate::types::square::Square;

fn horizontal_pin_test(
    board: &Board,
    capture: Square,
    source: Square,
    destination: Square,
) -> bool {
    // create combined bitboard of board with both source and capture removed.
    // removing the squares simulates the move
    let combined =
        board.combined() & !BitBoard::from_square(capture) & !BitBoard::from_square(source)
            | BitBoard::from_square(destination);

    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    let mut attack = BitBoard(0);

    attack |= slider_horizontal(king_square, combined)
        & (board.pieces(PieceType::Rook) | board.pieces(PieceType::Queen))
        & board.occupancies(!board.side_to_move());

    attack == BitBoard::EMPTY
}

pub fn generate_en_passant_move(board: &Board, move_list: &mut MoveList, push_mask: BitBoard) {
    if let Some(ep_square) = board.en_passant_target() {
        let side_to_move = board.side_to_move();

        let capture = ep_square.forward(!side_to_move);
        let destination = ep_square;

        let right = RelativeDir::BackwardRight.to_absolute(side_to_move);
        let left = RelativeDir::BackwardLeft.to_absolute(side_to_move);

        let destination_bb = BitBoard::from_square(ep_square) & push_mask;

        let current_sides_pawns = board.pieces(PieceType::Pawn) & board.occupancies(side_to_move);

        let king_square =
            (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

        let (main, anti) = match side_to_move {
            Color::White => (king_square.main_diagonal(), king_square.anti_diagonal()),
            Color::Black => (king_square.anti_diagonal(), king_square.main_diagonal()),
        };

        let pinned = board.pinned();

        let right_source = right.masked_shift(destination_bb) & (!pinned | (pinned & anti));
        let left_source = left.masked_shift(destination_bb) & (!pinned | (pinned & main));

        let mut sources = current_sides_pawns & (right_source | left_source);

        if sources.count() == 1 {
            let source = sources.bit_scan();

            if !horizontal_pin_test(board, capture, source, destination) {
                sources = BitBoard::EMPTY;
            }
        }

        for source in sources {
            move_list.push(Move::new(source, destination, MoveFlag::EnPassant));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::en_passant::generate_en_passant_move;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    fn test_en_passant_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_en_passant_move(board, moves_list, masks.push_mask)
            },
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
    fn test_two_en_passant_moves() {
        test_en_passant_moves(
            "8/8/k7/8/1pPp4/8/8/3K4 b - c3 0 1",
            &[
                Move::new(Square::B4, Square::C3, MoveFlag::EnPassant),
                Move::new(Square::D4, Square::C3, MoveFlag::EnPassant),
            ],
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

    #[test]
    fn test_en_passsant_block_check() {
        test_en_passant_moves(
            "k7/7q/8/6pP/8/8/8/1K6 w - g6 0 1",
            &[Move::new(Square::H5, Square::G6, MoveFlag::EnPassant)],
        );
    }
}

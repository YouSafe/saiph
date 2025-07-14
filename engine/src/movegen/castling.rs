use crate::board::Board;
use crate::movegen::attacks::between;
use crate::movegen::{is_square_attacked, MoveList};
use crate::types::chess_move::{Move, MoveFlag};
use types::bitboard::BitBoard;
use types::castling_rights::CastlingRights;
use types::color::NUM_COLORS;
use types::piece::PieceType;
use types::square::Square;

struct CastlingConfig {
    required_rights: CastlingRights,
    king_target: Square,
    safe_squares: [Square; 2],
    accompanied_rook: Square,
}

static CASTLING_CONFIGS: [[CastlingConfig; 2]; NUM_COLORS] = [
    // white: king side, queen side
    [
        CastlingConfig {
            required_rights: CastlingRights::WHITE_KING_SIDE,
            king_target: Square::G1,
            safe_squares: [Square::F1, Square::G1],
            accompanied_rook: Square::H1,
        },
        CastlingConfig {
            required_rights: CastlingRights::WHITE_QUEEN_SIDE,
            king_target: Square::C1,
            safe_squares: [Square::D1, Square::C1],
            accompanied_rook: Square::A1,
        },
    ],
    // black: king side, queen side
    [
        CastlingConfig {
            required_rights: CastlingRights::BLACK_KING_SIDE,
            king_target: Square::G8,
            safe_squares: [Square::F8, Square::G8],
            accompanied_rook: Square::H8,
        },
        CastlingConfig {
            required_rights: CastlingRights::BLACK_QUEEN_SIDE,
            king_target: Square::C8,
            safe_squares: [Square::D8, Square::C8],
            accompanied_rook: Square::A8,
        },
    ],
];

pub fn generate_castling_moves(board: &Board, move_list: &mut MoveList) {
    let castling_rights = board.castling_rights();

    let side_to_move = board.side_to_move();
    let king_square = (board.pieces(PieceType::King) & board.occupancies(side_to_move)).bit_scan();

    for config in &CASTLING_CONFIGS[side_to_move as usize] {
        if castling_rights.contains(config.required_rights)
            && (board.combined() & between(king_square, config.accompanied_rook)) == BitBoard::EMPTY
            && !is_square_attacked(board, config.safe_squares[0], !side_to_move)
            && !is_square_attacked(board, config.safe_squares[1], !side_to_move)
        {
            move_list.push(Move::new(
                king_square,
                config.king_target,
                MoveFlag::Castling,
            ));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::castling::generate_castling_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{compute_push_capture_mask, MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use types::square::Square::*;

    fn test_castling_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, _masks: &PushCaptureMasks| {
                generate_castling_moves(board, moves_list)
            },
            compute_push_capture_mask::<false>,
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_white_castling() {
        test_castling_moves(
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
            &[
                Move::new(E1, G1, MoveFlag::Castling),
                Move::new(E1, C1, MoveFlag::Castling),
            ],
        );
    }

    #[test]
    fn test_black_castling() {
        test_castling_moves(
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1",
            &[
                Move::new(E8, G8, MoveFlag::Castling),
                Move::new(E8, C8, MoveFlag::Castling),
            ],
        );
    }

    #[test]
    fn test_would_land_on_check_queen_side() {
        test_castling_moves(
            "r3k2r/pppppppp/8/6b1/8/8/PPP1PPPP/R3K2R w KQkq - 0 1",
            &[Move::new(E1, G1, MoveFlag::Castling)],
        );
    }

    #[test]
    fn test_would_land_on_check_king_side() {
        test_castling_moves(
            "r3k2r/pppppppp/8/2b5/8/5P2/PPPPP1PP/R3K2R w KQkq - 0 1",
            &[Move::new(E1, C1, MoveFlag::Castling)],
        );
    }

    #[test]
    fn test_both_sides_blocked() {
        test_castling_moves("r3k2r/pppppppp/8/8/8/1b5b/PP1PPP1P/R3K2R w KQkq - 0 1", &[]);
    }
}

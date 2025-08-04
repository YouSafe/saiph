use crate::board::Board;
use crate::movegen::MoveList;
use crate::movegen::attacks::between;
use crate::types::bitboard::BitBoard;
use crate::types::castling_rights::CastlingRights;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::color::NUM_COLORS;
use crate::types::piece::PieceType;
use crate::types::square::Square;

struct CastlingConfig {
    required_rights: CastlingRights,
    king_target: Square,
    safe_squares: BitBoard,
    cleared_squares_bb: BitBoard,
}

static CASTLING_CONFIGS: [[CastlingConfig; 2]; NUM_COLORS] = [
    // white: king side, queen side
    [
        CastlingConfig {
            required_rights: CastlingRights::WHITE_KING_SIDE,
            king_target: Square::G1,
            safe_squares: between(Square::E1, Square::H1),
            cleared_squares_bb: between(Square::E1, Square::H1),
        },
        CastlingConfig {
            required_rights: CastlingRights::WHITE_QUEEN_SIDE,
            king_target: Square::C1,
            safe_squares: between(Square::E1, Square::B1),
            cleared_squares_bb: between(Square::E1, Square::A1),
        },
    ],
    // black: king side, queen side
    [
        CastlingConfig {
            required_rights: CastlingRights::BLACK_KING_SIDE,
            king_target: Square::G8,
            safe_squares: between(Square::E8, Square::H8),
            cleared_squares_bb: between(Square::E8, Square::H8),
        },
        CastlingConfig {
            required_rights: CastlingRights::BLACK_QUEEN_SIDE,
            king_target: Square::C8,
            safe_squares: between(Square::E8, Square::B8),
            cleared_squares_bb: between(Square::E8, Square::A8),
        },
    ],
];

pub fn generate_castling_moves(board: &Board, move_list: &mut MoveList, attacked: BitBoard) {
    let castling_rights = board.castling_rights();

    let side_to_move = board.side_to_move();
    let king_square = (board.pieces(PieceType::King) & board.occupancies(side_to_move)).bit_scan();

    for config in &CASTLING_CONFIGS[side_to_move as usize] {
        if castling_rights.contains(config.required_rights)
            && (board.combined() & config.cleared_squares_bb) == BitBoard::EMPTY
            && (attacked & config.safe_squares) == BitBoard::EMPTY
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
    use crate::movegen::{MoveList, PushCaptureMasks};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square::*;

    fn test_castling_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_castling_moves(board, moves_list, masks.attacked)
            },
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

use crate::board::Board;
use crate::move_generation::{is_square_attacked, MoveList};
use crate::attacks::between;
use crate::types::bitboard::BitBoard;
use crate::types::castling_rights::CastlingRights;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::color::NUM_COLORS;
use crate::types::piece::Piece;
use crate::types::square::Square;

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

pub fn generate_castling_moves<const CHECK: bool>(board: &Board, move_list: &mut MoveList) {
    assert!(!CHECK, "can not castle in check");

    let castling_rights = board.castling_rights();

    let side_to_move = board.side_to_move();
    let king_square = (board.pieces(Piece::King) & board.occupancies(side_to_move)).bit_scan();

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
    use std::str::FromStr;

    use crate::board::Board;
    use crate::move_generation::castling::generate_castling_moves;
    use crate::move_generation::MoveList;
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square::*;

    #[test]
    fn test_white_castling() {
        let board = Board::from_str("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_castling_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 2);

        assert!(move_list.contains(&Move::new(E1, G1, MoveFlag::Castling)));
        assert!(move_list.contains(&Move::new(E1, C1, MoveFlag::Castling)));
    }

    #[test]
    fn test_black_castling() {
        let board = Board::from_str("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_castling_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 2);

        assert!(move_list.contains(&Move::new(E8, G8, MoveFlag::Castling)));
        assert!(move_list.contains(&Move::new(E8, C8, MoveFlag::Castling)));
    }

    #[test]
    fn test_would_land_on_check_queen_side() {
        let board =
            Board::from_str("r3k2r/pppppppp/8/6b1/8/8/PPP1PPPP/R3K2R w KQkq - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_castling_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move::new(E1, G1, MoveFlag::Castling)));
    }

    #[test]
    fn test_would_land_on_check_king_side() {
        let board =
            Board::from_str("r3k2r/pppppppp/8/2b5/8/5P2/PPPPP1PP/R3K2R w KQkq - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_castling_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 1);

        assert!(move_list.contains(&Move::new(E1, C1, MoveFlag::Castling)));
    }

    #[test]
    fn test_both_sides_blocked() {
        let board =
            Board::from_str("r3k2r/pppppppp/8/8/8/1b5b/PP1PPP1P/R3K2R w KQkq - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_castling_moves::<false>(&board, &mut move_list);
        println!("{:#?}", move_list);

        assert_eq!(move_list.len(), 0);
    }
}

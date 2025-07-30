use crate::board::Board;
use crate::movegen::MoveList;
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::PieceType;

pub fn generate_quiet_pawn_moves(board: &Board, move_list: &mut MoveList, push_mask: BitBoard) {
    let side_to_move = board.side_to_move();
    let current_sides_pawns = board.pieces(PieceType::Pawn) & board.occupancies(side_to_move);

    let pinned = board.pinned();

    let king_square =
        (board.pieces(PieceType::King) & board.occupancies(board.side_to_move())).bit_scan();

    // determine source squares that can move:
    // they have to either be not pinned or pinned with the king being on the same file
    let movable_sources = current_sides_pawns & (!pinned | (pinned & king_square.file().mask()));

    let forward_shift: i32 = 8 - 16 * (side_to_move as i32);

    // those sources are then shifted one square forward and any overlaps with existing pieces
    // on the board are removed
    let single_push = movable_sources.shift(forward_shift) & !board.combined();

    // restrict the single push targets to squares they can actually move to (check evasion)
    let single_push_targets = single_push & push_mask;

    // move the already moved squares, remove overlaps and restrict the final target squares to
    // legal squares, respecting checks
    let double_push_targets = single_push.shift(forward_shift)
        & !board.combined()
        & side_to_move.double_pawn_push_rank().mask()
        & push_mask;

    let promotion_rank = side_to_move.promotion_rank().mask();

    let non_promotions = single_push_targets & !promotion_rank;
    let promotions = single_push_targets & promotion_rank;

    for target in promotions.iter() {
        let source = target.forward(!side_to_move);

        move_list.push(Move::new(source, target, MoveFlag::KnightPromotion));
        move_list.push(Move::new(source, target, MoveFlag::BishopPromotion));
        move_list.push(Move::new(source, target, MoveFlag::RookPromotion));
        move_list.push(Move::new(source, target, MoveFlag::QueenPromotion));
    }

    for target in double_push_targets.iter() {
        let source = target.forward(!side_to_move).forward(!side_to_move);

        move_list.push(Move::new(source, target, MoveFlag::DoublePawnPush));
    }

    for target in non_promotions.iter() {
        let source = target.forward(!side_to_move);

        move_list.push(Move::new(source, target, MoveFlag::Normal));
    }
}

#[cfg(test)]
mod test {
    use crate::board::Board;
    use crate::movegen::quiet_pawn::generate_quiet_pawn_moves;
    use crate::movegen::test::test_move_generator;
    use crate::movegen::{MoveList, PushCaptureMasks, compute_push_capture_mask};
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::color::Color;
    use crate::types::square::Square::{self, *};
    use crate::types::square::{File, NUM_FILES, Rank};

    fn test_quiet_pawn_moves(fen: &str, expected_moves: &[Move]) {
        test_move_generator::<_, _, false>(
            |board: &Board, moves_list: &mut MoveList, masks: &PushCaptureMasks| {
                generate_quiet_pawn_moves(board, moves_list, masks.push_mask)
            },
            compute_push_capture_mask::<false>,
            fen,
            expected_moves,
        )
    }

    #[test]
    fn test_single_and_double_push() {
        test_quiet_pawn_moves(
            "k7/8/8/8/8/8/7P/K7 w - - 0 1",
            &[
                Move::new(H2, H3, MoveFlag::Normal),
                Move::new(H2, H4, MoveFlag::DoublePawnPush),
            ],
        );
    }

    #[test]
    pub fn test_promotion() {
        test_quiet_pawn_moves(
            "k7/7P/8/8/8/8/8/K7 w - - 0 1",
            &[
                Move::new(H7, H8, MoveFlag::BishopPromotion),
                Move::new(H7, H8, MoveFlag::KnightPromotion),
                Move::new(H7, H8, MoveFlag::RookPromotion),
                Move::new(H7, H8, MoveFlag::QueenPromotion),
            ],
        );
    }

    #[test]
    pub fn test_forced_check_block() {
        test_quiet_pawn_moves(
            "6k1/8/8/8/K6r/8/4P3/8 w - - 0 1",
            &[Move::new(E2, E4, MoveFlag::DoublePawnPush)],
        );
    }

    #[test]
    pub fn test_pinned_by_rook_but_can_move_forward() {
        test_quiet_pawn_moves(
            "1K4k1/8/8/1P6/8/1r6/8/8 w - - 0 1",
            &[Move::new(B5, B6, MoveFlag::Normal)],
        )
    }

    #[test]
    pub fn test_rook_backward_pin() {
        test_quiet_pawn_moves(
            "1r4k1/8/8/8/8/8/1P6/1K6 w - - 0 1",
            &[
                Move::new(B2, B3, MoveFlag::Normal),
                Move::new(B2, B4, MoveFlag::DoublePawnPush),
            ],
        );
    }

    #[test]
    fn test_bishop_pin() {
        test_quiet_pawn_moves("6k1/8/5b2/8/8/8/1P6/K7 w - - 0 1", &[]);
    }

    #[test]
    fn test_two_pawns_one_bishop_pin() {
        test_quiet_pawn_moves(
            "6k1/8/5b2/8/8/1P6/1P6/K7 w - - 0 1",
            &[Move::new(B3, B4, MoveFlag::Normal)],
        );
    }

    #[test]
    fn test_check_pawn_can_not_block() {
        test_quiet_pawn_moves("6k1/8/5b2/8/8/1P6/8/K7 w - - 0 1", &[]);
    }

    #[test]
    fn test_pawn_pushes_startpos() {
        let mut moves = vec![];
        let color = Color::White;
        let rank = Rank::R2;

        use File::*;
        pub const ALL_FILES: [File; NUM_FILES] = [A, B, C, D, E, F, G, H];

        for file in ALL_FILES {
            let from = Square::from(rank, file);
            moves.push(Move::new(from, from.forward(color), MoveFlag::Normal));
            moves.push(Move::new(
                from,
                from.forward(color).forward(color),
                MoveFlag::DoublePawnPush,
            ));
        }

        test_quiet_pawn_moves(Board::STARTING_POS_FEN, &moves);
    }
}

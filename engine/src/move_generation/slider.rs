use crate::board::Board;
use crate::move_generation::MoveList;
use crate::attacks::{between, get_bishop_attacks, get_rook_attacks, line};
use crate::types::bitboard::BitBoard;
use crate::types::chess_move::{Move, MoveFlag};
use crate::types::piece::Piece;

pub fn generate_slider_moves<const CHECK: bool, const CAPTURE_ONLY: bool>(
    board: &Board,
    move_list: &mut MoveList,
) {
    let mut capture_mask = !BitBoard::EMPTY;
    let mut push_mask = !BitBoard::EMPTY;

    let king_square =
        (board.pieces(Piece::King) & board.occupancies(board.side_to_move())).bit_scan();

    if CHECK {
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
    push_mask &= !board.occupancies(!side_to_move);

    let bishops = board.pieces(Piece::Bishop) & board.occupancies(side_to_move);
    let rooks = board.pieces(Piece::Rook) & board.occupancies(side_to_move);
    let queens = board.pieces(Piece::Queen) & board.occupancies(side_to_move);

    let combined = board.combined();

    // TODO: refactor to avoid code duplication

    // diagonal attackers
    for source in ((bishops | queens) & !pinned).iter() {
        let attacks = get_bishop_attacks(source, combined) & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }

        if !CAPTURE_ONLY {
            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move::new(source, target, MoveFlag::Normal));
            }
        }
    }

    for source in ((bishops | queens) & pinned).iter() {
        let attacks = get_bishop_attacks(source, combined)
            & line(king_square, source)
            & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture));
        }

        if !CAPTURE_ONLY {
            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move::new(source, target, MoveFlag::Normal));
            }
        }
    }

    // line attackers
    for source in ((rooks | queens) & !pinned).iter() {
        let attacks = get_rook_attacks(source, combined) & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture))
        }

        if !CAPTURE_ONLY {
            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move::new(source, target, MoveFlag::Normal));
            }
        }
    }

    for source in ((rooks | queens) & pinned).iter() {
        let attacks = get_rook_attacks(source, combined)
            & line(king_square, source)
            & !board.occupancies(side_to_move);

        // captures
        for target in (attacks & capture_mask).iter() {
            move_list.push(Move::new(source, target, MoveFlag::Capture))
        }

        if !CAPTURE_ONLY {
            // quiet
            for target in (attacks & push_mask).iter() {
                move_list.push(Move::new(source, target, MoveFlag::Normal));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::board::Board;
    use crate::move_generation::slider::generate_slider_moves;
    use crate::move_generation::MoveList;
    use crate::types::chess_move::{Move, MoveFlag};
    use crate::types::square::Square;

    #[test]
    fn test_move_along_pin_ray() {
        let board = Board::from_str("4k3/8/7b/3P4/8/8/3B4/2K5 w - - 3 2").unwrap();
        let mut move_list = MoveList::new();
        generate_slider_moves::<false, false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 4);

        assert!(move_list.contains(&Move::new(Square::D2, Square::E3, MoveFlag::Normal)));

        assert!(move_list.contains(&Move::new(Square::D2, Square::F4, MoveFlag::Normal)));

        assert!(move_list.contains(&Move::new(Square::D2, Square::G5, MoveFlag::Normal)));

        assert!(move_list.contains(&Move::new(Square::D2, Square::H6, MoveFlag::Capture)));
    }

    #[test]
    fn test_pinned_bishop_captures() {
        let board = Board::from_str("8/2p5/3p4/KP5r/1R3b1k/6P1/4P3/8 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_slider_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_pinned_rook_captures() {
        let board = Board::from_str("8/2p5/3p4/KP5r/1R4rk/6P1/4P3/8 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_slider_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }

    #[test]
    fn test_pinned_queen_captures() {
        let board = Board::from_str("8/2p5/3p4/KP5r/1R4qk/6P1/4P3/8 b - - 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_slider_moves::<true, false>(&board, &mut move_list);
        println!("{:#?}", move_list);
        assert_eq!(move_list.len(), 0);
    }
}

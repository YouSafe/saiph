use crate::bitboard::BitBoard;
use crate::board::Board;
use crate::chess_move::Move;
use crate::color::Color;
use crate::piece::{Piece, ALL_PIECES};
use crate::square::Square;
use crate::tables::{
    get_bishop_attacks, get_king_attacks, get_knight_attacks, get_pawn_attacks, get_queen_attacks,
    get_rook_attacks,
};

type MoveList = Vec<Move>;

pub fn generate_attack_bitboard(board: &Board, attacking_color: Color) -> BitBoard {
    let mut attacked = BitBoard(0);

    let king = board.pieces(Piece::King) & board.occupancies(!attacking_color);

    // remove opponent king from blockers to simulate xray attack
    let blockers = *board.combined() & !king;

    for piece in ALL_PIECES {
        let piece_bitboard = *board.pieces(piece) & board.occupancies(attacking_color);
        for square in piece_bitboard.iter() {
            let piece = board
                .piece_on_square(square)
                .expect("piece must not be none");

            attacked |= match piece {
                Piece::Pawn => get_pawn_attacks(square, attacking_color),
                Piece::Knight => get_knight_attacks(square),
                Piece::Bishop => get_bishop_attacks(square, blockers),
                Piece::Rook => get_rook_attacks(square, blockers),
                Piece::Queen => get_queen_attacks(square, blockers),
                Piece::King => get_king_attacks(square),
            };
        }
    }

    attacked
}

pub fn calculate_pinned_checkers_attacks(board: &Board) {
    // let king_square =
    //     (board.pieces(Piece::King) & board.occupancies(board.side_to_move)).bit_scan();
    //
    // // let pinners =
    //
    // let pinned = BitBoard(0);
}

pub fn generate_pinned_bitboard(board: &Board, side: Color) -> BitBoard {
    let pinned = BitBoard(0);
    todo!();
    pinned
}

pub fn generate_moves(board: &Board) -> MoveList {
    let move_list = vec![];

    move_list
}

pub fn is_square_attacked(board: &Board, attacked_square: Square, attacking_side: Color) -> bool {
    // attacked by pawns?
    if (get_pawn_attacks(attacked_square, !attacking_side)
        & board.pieces(Piece::Pawn)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by knight?
    if (get_knight_attacks(attacked_square)
        & board.pieces(Piece::Knight)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by king?
    if (get_king_attacks(attacked_square)
        & board.pieces(Piece::King)
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by bishop or queen?
    if (get_bishop_attacks(attacked_square, *board.combined())
        & (board.pieces(Piece::Bishop) | board.pieces(Piece::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    // attacked by rook or queen?
    if (get_rook_attacks(attacked_square, *board.combined())
        & (board.pieces(Piece::Rook) | board.pieces(Piece::Queen))
        & board.occupancies(attacking_side))
        != BitBoard(0)
    {
        return true;
    }

    false
}

pub fn build_attacked_bitboard(board: &Board, attacking_side: Color) -> BitBoard {
    let mut bitboard = BitBoard(0);
    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::from_index(rank * 8 + file);

            if is_square_attacked(&board, square, attacking_side) {
                bitboard |= square;
            }
        }
    }
    bitboard
}

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::board::Board;
    use crate::color::Color;
    use crate::movgen::{build_attacked_bitboard, generate_attack_bitboard, is_square_attacked};
    use crate::square::Square;
    use std::str::FromStr;

    #[test]
    fn test_is_square_attacked_pawn_attack() {
        let board = Board::from_str("8/8/8/3p4/8/8/8/8 w - - 0 1").unwrap();
        println!("board: {board}");

        let is_e4_attacked_by_black = is_square_attacked(&board, Square::E4, Color::Black);

        assert!(is_e4_attacked_by_black);
        println!("is e4 attacked by black pawn? {is_e4_attacked_by_black}",);
    }

    #[test]
    fn test_is_square_attacked_bishop_attack() {
        let board = Board::from_str("7k/8/2P5/8/8/8/6b1/K7 w - - 0 1").unwrap();
        println!("board: {board}");

        let is_c6_attacked_by_black = is_square_attacked(&board, Square::C6, Color::Black);

        assert!(is_c6_attacked_by_black);

        println!(
            "is the pawn on c6 attacked by the bishop on g2? {}",
            is_c6_attacked_by_black
        );
    }

    #[test]
    fn test_is_square_attacked_rook_attack() {
        let board = Board::from_str("7k/8/8/8/2R2b2/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_f4_attacked_by_white = is_square_attacked(&board, Square::F4, Color::White);

        assert!(is_f4_attacked_by_white);

        println!(
            "is the bishop on f4 attacked by the rook on c4? {}",
            is_f4_attacked_by_white
        );
    }

    #[test]
    fn test_is_square_attacked_knight_attack() {
        let board = Board::from_str("7k/8/4q3/8/3N4/8/8/K7 w - - 0 1").unwrap();
        println!("{board}");

        let is_e6_attacked_by_white = is_square_attacked(&board, Square::E6, Color::White);

        assert!(is_e6_attacked_by_white);

        println!(
            "is the queen on e6 attacked by the knight on d4? {}",
            is_e6_attacked_by_white
        );
    }

    #[test]
    fn test_is_square_attacked_queen_attack() {
        let board = Board::from_str("7k/P7/8/8/8/8/8/K5q1 w - - 0 1").unwrap();
        println!("{board}");

        let is_a7_attacked_by_black = is_square_attacked(&board, Square::A7, Color::Black);

        assert!(is_a7_attacked_by_black);

        println!(
            "is the pawn on a7 attacked by the queen on g1? {}",
            is_a7_attacked_by_black
        );
    }

    #[test]
    fn test_build_attacked_bitboard() {
        let board = Board::default();
        let attacked = build_attacked_bitboard(&board, Color::White);
        println!("{attacked}");

        let expected = BitBoard(16777086);
        println!("expected: {expected}");
        assert_eq!(attacked, expected);
    }

    #[test]
    fn test_generate_attack_bitboard() {
        let board = Board::default();
        let attacked = generate_attack_bitboard(&board, Color::White);

        println!("{}", board.combined());
        println!("{attacked}");

        let test = build_attacked_bitboard(&board, Color::White);
        println!("{test}");
        assert_eq!(attacked, test);
    }
}

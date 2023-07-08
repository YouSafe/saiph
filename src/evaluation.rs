#![cfg_attr(rustfmt, rustfmt_skip)]
// See: https://www.chessprogramming.org/Simplified_Evaluation_Function

use std::cmp::Ordering;
use chess::{ALL_PIECES, Board, BoardStatus, Color, Piece, Square};
use Evaluation::{Infimum, Supremum};
use crate::evaluation::Evaluation::{Checkmate, MateIn, Regular};

#[derive(PartialEq, Clone, Copy, Eq, Debug)]
pub enum Evaluation {
    /// Color is the color of the player checkmating
    Checkmate(Color),
    MateIn(i8),
    Regular(i16),
    Infimum,
    Supremum
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Infimum, _) => Some(Ordering::Less),
            (Supremum, _) => Some(Ordering::Greater),

            (_, Infimum) => Some(Ordering::Greater),
            (_, Supremum) => Some(Ordering::Less),

            (Checkmate(Color::White), Checkmate(Color::White)) => Some(Ordering::Equal),
            (Checkmate(Color::Black), Checkmate(Color::Black)) => Some(Ordering::Equal),
            
            (Checkmate(Color::White), _) => Some(Ordering::Greater),
            (Checkmate(Color::Black), _) => Some(Ordering::Less),

            (_, Checkmate(Color::White)) => Some(Ordering::Less),
            (_, Checkmate(Color::Black)) => Some(Ordering::Greater),
            
            (MateIn(n), MateIn(m)) => {
                let (n_abs, m_abs) = (n.abs(), m.abs());
                
                let ordering = match (n.is_negative(), m.is_negative()) {
                    (false, false) => n_abs.cmp(&m_abs).reverse(),
                    (false, true) => Ordering::Greater,
                    (true, false) => Ordering::Less,
                    (true, true) => n_abs.cmp(&m_abs),
                };
                
                Some(ordering)
            } ,
            (Regular(n), Regular(m)) => Some(n.cmp(m)),
            
            (MateIn(m), Regular(_)) => Some(m.cmp(&0)),
            (Regular(_), MateIn(m)) => Some(0.cmp(m)),
        }
    }
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn piece_value(piece: Piece, square: Square, piece_color: Color) -> Evaluation {
    let sign = match piece_color {
        Color::White => 1,
        Color::Black => -1,
    };

    let piece_value = match piece {
        Piece::Pawn => 100,
        Piece::Knight => 320,
        Piece::Bishop => 330,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 20000,
    };

    let bonus = piece_square_table(piece, square, piece_color);

    Regular(sign * (piece_value + bonus))
}

pub fn board_value(board: &Board, depth: u8) -> Evaluation {
    let board_status = board.status();
    if let BoardStatus::Checkmate = board_status {
        let color_to_move = board.side_to_move();
        let sign = match color_to_move{
            Color::White => -1,
            Color::Black => 1,
        };
        return MateIn(sign * depth as i8);
    } else if let BoardStatus::Stalemate = board_status {
        return Regular(0);
    }

    let mut result = 0;
    for piece in ALL_PIECES {
        let bitboard = *board.pieces(piece);
        for square in bitboard {
            if let Regular(eval) = piece_value(piece, square, board.color_on(square).unwrap()) {
                result += eval;
            }
        }
    }
    Regular(result)
}

pub fn piece_square_table(piece: Piece, square: Square, piece_color: Color) -> i16 {
    let square_index = square.to_index();

    let rank = square.get_rank().to_index();
    let file = square.get_file().to_index();
    
    let lookup_index = match piece_color {
        Color::White => (7 - rank) * 8 + file,
        Color::Black => square_index,
    };

    let bonus = match piece {
        Piece::Pawn => PAWNS_TABLE[lookup_index],
        Piece::Knight => KNIGHTS_TABLE[lookup_index],
        Piece::Bishop => BISHOP_TABLE[lookup_index],
        Piece::Rook => ROOK_TABLE[lookup_index],
        Piece::Queen => QUEEN_TABLE[lookup_index],
        Piece::King => 0,
    } as i16;
    bonus
}

const PAWNS_TABLE: [i8; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHTS_TABLE: [i8; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_TABLE: [i8; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_TABLE: [i8; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_TABLE: [i8; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -5,  0,  5,  5,  5,  5,  0, -5,
    0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use chess::{Color, Piece, Square};
    use chess::Color::{Black, White};
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use crate::evaluation::Evaluation::{MateIn, Supremum, Infimum, Regular, Checkmate};
    use crate::evaluation::piece_square_table;

    #[test]
    fn pawn_piece_square_value() {
        assert_eq!(piece_square_table(Piece::Pawn, Square::from_str("d4").unwrap(), Color::White), 20);
        assert_eq!(piece_square_table(Piece::Pawn, Square::from_str("c4").unwrap(), Color::White), 0);
        assert_eq!(piece_square_table(Piece::Pawn, Square::from_str("d5").unwrap(), Color::Black), 20);
    }
    
    #[test]
    fn evaluation() {
        assert!(MateIn(10) > Regular(1000));
        assert!(Regular(1000) < MateIn(10));
        assert!(Regular(1000) < Regular(10001));
        assert!(MateIn(1) > MateIn(2));
        assert!(MateIn(1) > Regular(-2300));
        assert_eq!(MateIn(1).min(Regular(-2300)), Regular(-2300));
        assert_eq!(MateIn(1).max(Regular(-2300)), MateIn(1));
        assert_eq!(MateIn(1).max(MateIn(2)), MateIn(1));
        assert_eq!(MateIn(1).max(Supremum), Supremum);
        assert_eq!(MateIn(1).max(Infimum), MateIn(1));
        assert_eq!(MateIn(1).min(Supremum), MateIn(1));
        assert_eq!(MateIn(1).min(Infimum), Infimum);
        assert_eq!(Regular(185).min(Infimum), Infimum);
        assert_eq!(Regular(185).max(Supremum), Supremum);
        assert!(Regular(185) <= Supremum);
        assert!(Regular(185) >= Infimum);
        assert!(MateIn(1) <= Supremum);
        assert!(MateIn(1) >= Infimum);
        assert_eq!(Supremum, Supremum);
        assert_eq!(Infimum, Infimum);
        assert_ne!(Infimum, Supremum);
        assert_ne!(Supremum, Infimum);
        assert!(Infimum < Supremum);
        assert!(Infimum <= Supremum);
        assert_eq!(Infimum.max(Regular(134).max(MateIn(1))), MateIn(1));
        assert_eq!(Supremum.min(Regular(134).min(MateIn(1))), Regular(134));
        assert!(MateIn(-1) < MateIn(-2));
        assert!(Regular(-200) < Regular(-100));
        assert!(Regular(-200) < MateIn(1));
        assert!(Regular(200) > MateIn(-1));
        assert!(Regular(-200) > MateIn(-1));
        
        assert!(Regular(-200) < Checkmate(White));
        assert!(Regular(200) > Checkmate(Black));
        assert!(Checkmate(Black) < Checkmate(White));
        assert!(Regular(200) < Checkmate(White));
        assert!(Regular(-200) > Checkmate(Black));
        assert!(Checkmate(White) > MateIn(1));
        assert!(Checkmate(Black) < MateIn(-1));
        assert_eq!(Infimum.max(Regular(123)).max(MateIn(1)).max(Checkmate(White)), Checkmate(White));
        assert_eq!(Supremum.min(Regular(123)).min(MateIn(1)).min(Checkmate(White)), Regular(123));
        assert_eq!(Supremum.min(Supremum), Supremum);
        assert_eq!(Regular(0), Regular(0));
        assert!(Regular(0) > Regular(-1));
        assert!(Regular(0) < Regular(1));
        assert!(Checkmate(White) > MateIn(-1));
        assert!(Checkmate(Black) < MateIn(1));
        assert_eq!(Checkmate(Black), Checkmate(Black));
        assert_eq!(Checkmate(White), Checkmate(White));
        assert_ne!(Checkmate(Black), Checkmate(White));

        assert!(Regular(-1) > MateIn(-2));
        assert!(MateIn(-1) < MateIn(1));
        assert!(!(MateIn(-2) < MateIn(1) && MateIn(1) < MateIn(-1)) || (MateIn(-2) < MateIn(-1)));
        
        let list = [Infimum, Checkmate(Black), MateIn(-1), MateIn(-2), Regular(-2), Regular(-1), Regular(0), Regular(1), Regular(2), MateIn(2), MateIn(1), Checkmate(White), Supremum];
        let mut sorted = list.clone();
        sorted.reverse();
        sorted.sort_unstable();

        assert_eq!(sorted, list);
    }
}
use chess_core::bitboard::BitBoard;
use chess_core::square::Square;
use std::iter::repeat;

pub fn generate_rook_relevant_occupancy() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_rook_relevant_occupancy(sq);
    }

    result
}

pub fn generate_rook_relevant_bits() -> [u8; 64] {
    let mut result = [0; 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        // TODO: maybe don't calculate it again
        result[square] = mask_rook_relevant_occupancy(sq).0.count_ones() as u8;
    }

    result
}

fn mask_rook_relevant_occupancy(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);

    let (square_rank, square_file) = (square.to_index() / 8, square.to_index() % 8);

    // Note: the outer rim is excluded because pieces on the rim do not block
    let lines: [Box<dyn Iterator<Item = (u8, u8)>>; 4] = [
        // right
        Box::new((repeat(square_rank)).zip(square_file + 1..=6)),
        // left
        Box::new((repeat(square_rank)).zip((1..square_file).rev())),
        // top
        Box::new((square_rank + 1..=6).zip(repeat(square_file))),
        // bottom
        Box::new(((1..square_rank).rev()).zip(repeat(square_file))),
    ];

    for line in lines {
        for (rank, file) in line {
            attacks |= BitBoard::from_square(Square::from_index(rank * 8 + file));
        }
    }

    attacks
}

fn mask_rook_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0);

    let (square_rank, square_file) = (square.to_index() / 8, square.to_index() % 8);

    let lines: [Box<dyn Iterator<Item = (u8, u8)>>; 4] = [
        // right
        Box::new((repeat(square_rank)).zip(square_file + 1..=7)),
        // left
        Box::new((repeat(square_rank)).zip((0..square_file).rev())),
        // top
        Box::new((square_rank + 1..=7).zip(repeat(square_file))),
        // bottom
        Box::new(((0..square_rank).rev()).zip(repeat(square_file))),
    ];

    for line in lines {
        for (rank, file) in line {
            let square_bitboard = BitBoard::from_square(Square::from_index(rank * 8 + file));
            attacks |= square_bitboard;
            if (blockers & square_bitboard) != BitBoard(0) {
                break;
            }
        }
    }

    attacks
}

#[cfg(test)]
mod test {
    use crate::rook_move::{mask_rook_attacks_on_the_fly, mask_rook_relevant_occupancy};
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

    #[test]
    fn test_rook_relevant_occupancy_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 11] = [B4, C4, D4, E6, F4, G4, E6, E7, E3, E2, E5];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(E4);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_relevant_occupancy_h8() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 12] = [B8, C8, D8, E8, F8, G8, H7, H6, H5, H4, H3, H2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(H8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_relevant_occupancy_a8() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 12] = [B8, C8, D8, E8, F8, G8, A7, A6, A5, A4, A3, A2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(A8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_relevant_occupancy_a1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 12] = [A7, A6, A5, A4, A3, A2, B1, C1, D1, E1, F1, G1];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(A1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_relevant_occupancy_h1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 12] = [B1, C1, D1, E1, F1, G1, H7, H6, H5, H4, H3, H2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(H1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_relevant_occupancy_h5() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 11] = [H7, H6, H4, H3, H2, B5, C5, D5, E5, F5, G5];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_rook_relevant_occupancy(H5);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_attacks_on_the_fly_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 9] = [D4, E3, E2, F4, G4, H4, E5, E6, E7];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let mut blockers = BitBoard(0);
        blockers |= BitBoard::from_square(D4);
        blockers |= BitBoard::from_square(E2);
        blockers |= BitBoard::from_square(H4);
        blockers |= BitBoard::from_square(E7);
        let attacks = mask_rook_attacks_on_the_fly(E4, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_rook_attacks_on_the_fly_a1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 2] = [A2, B1];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let mut blockers = BitBoard(0);
        blockers |= BitBoard::from_square(A2);
        blockers |= BitBoard::from_square(B1);
        let attacks = mask_rook_attacks_on_the_fly(A1, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

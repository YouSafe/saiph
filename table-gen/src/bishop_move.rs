use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_bishop_relevant_occupancy() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_bishop_relevant_occupancy(sq);
    }

    result
}

fn mask_bishop_relevant_occupancy(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);

    let (square_rank, square_file) = (square.to_index() / 8, square.to_index() % 8);

    // Note: the outer rim is excluded because pieces on the rim do not block
    let diagonals: [Box<dyn Iterator<Item = (u8, u8)>>; 4] = [
        // top right
        Box::new(((square_rank + 1)..=6).zip(square_file + 1..=6)),
        // top left
        Box::new(((square_rank + 1)..=6).zip((1..square_file).rev())),
        // bottom left
        Box::new(((1..square_rank).rev()).zip((1..square_file).rev())),
        // bottom right
        Box::new(((1..square_rank).rev()).zip(square_file + 1..=6)),
    ];

    for diagonal in diagonals {
        for (rank, file) in diagonal {
            attacks |= BitBoard::from_square(Square::from_index(rank * 8 + file));
        }
    }

    attacks
}

#[cfg(test)]
mod test {
    use crate::bishop_move::mask_bishop_relevant_occupancy;
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

    #[test]
    fn test_bishop_relevant_occupancy_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 9] = [F5, G6, D5, C6, B7, D3, C2, F3, G2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(E4);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_h8() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 6] = [B2, C3, D4, E5, F6, G7];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(H8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_a8() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 6] = [B7, C6, D5, E4, F3, G2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(A8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_a1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 6] = [B2, C3, D4, E5, F6, G7];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(A1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_h1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 6] = [B7, C6, D5, E4, F3, G2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(H1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_h5() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 5] = [G6, F7, G4, F3, E2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let attacks = mask_bishop_relevant_occupancy(H5);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

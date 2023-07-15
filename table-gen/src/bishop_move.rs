use crate::magic_number::{
    find_magic_number, generate_occupancy, BitBoardMapping, Magic, MagicNumberCandidateGenerator,
};
use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

// TODO: maybe think of a better design
#[derive(Debug)]
pub struct BishopAttacks {
    pub magic_number_table: [Magic; 64],
    pub attack_table: Vec<[BitBoard; 64]>,
}

// TODO: document number 512
pub fn generate_bishop_attacks() -> BishopAttacks {
    // TODO: maybe flatten array
    let mut attacks = BishopAttacks {
        magic_number_table: [Magic::default(); 64],
        attack_table: vec![[BitBoard(0); 64]; 512],
    };

    let mut rng = MagicNumberCandidateGenerator::new(1804289383);

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        let relevant_occupancy_bit_mask = mask_bishop_relevant_occupancy(sq);
        let num_relevant_occupancy_bits = relevant_occupancy_bit_mask.popcnt();

        let target_mapping = (0..(1u64 << num_relevant_occupancy_bits))
            .map(|index| {
                let occupancy = generate_occupancy(index, relevant_occupancy_bit_mask);
                let attacks = mask_bishop_attacks_on_the_fly(sq, occupancy);

                BitBoardMapping {
                    from: occupancy,
                    to: attacks,
                }
            })
            .collect::<Vec<_>>();

        let magic = find_magic_number(
            &mut rng,
            relevant_occupancy_bit_mask,
            &target_mapping,
            num_relevant_occupancy_bits,
        );

        for mapping in target_mapping {
            let magic_index = (mapping.from * magic.magic_number).0 >> magic.shift;
            attacks.attack_table[magic_index as usize][square as usize] = mapping.to;
        }
        attacks.magic_number_table[square as usize] = magic;
    }
    attacks
}

pub fn mask_bishop_relevant_occupancy(square: Square) -> BitBoard {
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

pub fn generate_bishop_relevant_occupancy() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_bishop_relevant_occupancy(sq);
    }

    result
}

pub fn mask_bishop_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0);

    let (square_rank, square_file) = (square.to_index() / 8, square.to_index() % 8);

    // TODO: Create iterators for rank and file structs to simplify this
    let diagonals: [Box<dyn Iterator<Item = (u8, u8)>>; 4] = [
        // top right
        Box::new(((square_rank + 1)..=7).zip(square_file + 1..=7)),
        // top left
        Box::new(((square_rank + 1)..=7).zip((0..square_file).rev())),
        // bottom left
        Box::new(((0..square_rank).rev()).zip((0..square_file).rev())),
        // bottom right
        Box::new(((0..square_rank).rev()).zip(square_file + 1..=7)),
    ];

    for diagonal in diagonals {
        for (rank, file) in diagonal {
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
    use crate::bishop_move::{mask_bishop_attacks_on_the_fly, mask_bishop_relevant_occupancy};
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

    #[test]
    fn test_bishop_attack_on_the_fly_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 9] = [F5, D5, C6, B7, D3, C2, F3, G2, H1];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let mut blockers = BitBoard(0);
        blockers |= BitBoard::from_square(B7);
        blockers |= BitBoard::from_square(C2);
        blockers |= BitBoard::from_square(F5);
        blockers |= BitBoard::from_square(H1);
        let attacks = mask_bishop_attacks_on_the_fly(E4, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_attack_on_the_fly_a1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 1] = [B2];
        for square in SQUARES {
            expected |= BitBoard::from_square(square);
        }
        let mut blockers = BitBoard(0);
        blockers |= BitBoard::from_square(B2);
        let attacks = mask_bishop_attacks_on_the_fly(A1, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

use crate::bitboard::BitBoard;
use crate::square::Square;
use crate::tables::magic_number::{generate_occupancy, BitBoardMapping, Magic};

#[rustfmt::skip]
pub const BISHOP_MAGIC_NUMBERS: [Magic; 64] = [
    Magic { magic_number: 0x000208892c040040, shift: 58, mask: BitBoard(18049651735527936), },
    Magic { magic_number: 0x0218100502182110, shift: 59, mask: BitBoard(70506452091904), },
    Magic { magic_number: 0x00420200410004b0, shift: 59, mask: BitBoard(275415828992), },
    Magic { magic_number: 0x14080b4100003080, shift: 59, mask: BitBoard(1075975168), },
    Magic { magic_number: 0x4408484000000042, shift: 59, mask: BitBoard(38021120), },
    Magic { magic_number: 0x01182a3010010104, shift: 59, mask: BitBoard(8657588224), },
    Magic { magic_number: 0x6081008820880000, shift: 59, mask: BitBoard(2216338399232), },
    Magic { magic_number: 0x4a1122024620100d, shift: 58, mask: BitBoard(567382630219776), },
    Magic { magic_number: 0x008040100202104a, shift: 59, mask: BitBoard(9024825867763712), },
    Magic { magic_number: 0x0090052142040100, shift: 59, mask: BitBoard(18049651735527424), },
    Magic { magic_number: 0x2a1218082325a000, shift: 59, mask: BitBoard(70506452221952), },
    Magic { magic_number: 0x00080434018c2000, shift: 59, mask: BitBoard(275449643008), },
    Magic { magic_number: 0x1060011040010002, shift: 59, mask: BitBoard(9733406720), },
    Magic { magic_number: 0x0208022820080800, shift: 59, mask: BitBoard(2216342585344), },
    Magic { magic_number: 0x0080008808025019, shift: 59, mask: BitBoard(567382630203392), },
    Magic { magic_number: 0x980422020a414400, shift: 59, mask: BitBoard(1134765260406784), },
    Magic { magic_number: 0x10c1012002420240, shift: 59, mask: BitBoard(4512412933816832), },
    Magic { magic_number: 0x0062280410420220, shift: 59, mask: BitBoard(9024825867633664), },
    Magic { magic_number: 0x0002090404051200, shift: 57, mask: BitBoard(18049651768822272), },
    Magic { magic_number: 0x400c0050c0408006, shift: 57, mask: BitBoard(70515108615168), },
    Magic { magic_number: 0x0022200400a00020, shift: 57, mask: BitBoard(2491752130560), },
    Magic { magic_number: 0x0102800048044000, shift: 57, mask: BitBoard(567383701868544), },
    Magic { magic_number: 0x10021010484a0804, shift: 59, mask: BitBoard(1134765256220672), },
    Magic { magic_number: 0x0002009490840109, shift: 59, mask: BitBoard(2269530512441344), },
    Magic { magic_number: 0x0402408010900212, shift: 59, mask: BitBoard(2256206450263040), },
    Magic { magic_number: 0x0091040019082800, shift: 59, mask: BitBoard(4512412900526080), },
    Magic { magic_number: 0x000a060209080200, shift: 57, mask: BitBoard(9024834391117824), },
    Magic { magic_number: 0x2011080084004010, shift: 55, mask: BitBoard(18051867805491712), },
    Magic { magic_number: 0x2000840030802022, shift: 55, mask: BitBoard(637888545440768), },
    Magic { magic_number: 0x0401004008080808, shift: 57, mask: BitBoard(1135039602493440), },
    Magic { magic_number: 0x0001060811461008, shift: 59, mask: BitBoard(2269529440784384), },
    Magic { magic_number: 0x102d202101040320, shift: 59, mask: BitBoard(4539058881568768), },
    Magic { magic_number: 0x4044304001348410, shift: 59, mask: BitBoard(1128098963916800), },
    Magic { magic_number: 0x0000882000040440, shift: 59, mask: BitBoard(2256197927833600), },
    Magic { magic_number: 0x0030180400081040, shift: 57, mask: BitBoard(4514594912477184), },
    Magic { magic_number: 0x0aa0c00a00002200, shift: 55, mask: BitBoard(9592139778506752), },
    Magic { magic_number: 0x5460060020020480, shift: 55, mask: BitBoard(19184279556981248), },
    Magic { magic_number: 0x4a100a10200a0080, shift: 57, mask: BitBoard(2339762086609920), },
    Magic { magic_number: 0xc108009400910100, shift: 59, mask: BitBoard(4538784537380864), },
    Magic { magic_number: 0x0048019100028154, shift: 59, mask: BitBoard(9077569074761728), },
    Magic { magic_number: 0x0808019008001002, shift: 59, mask: BitBoard(562958610993152), },
    Magic { magic_number: 0x4050410420001010, shift: 59, mask: BitBoard(1125917221986304), },
    Magic { magic_number: 0x50000e0082001000, shift: 57, mask: BitBoard(2814792987328512), },
    Magic { magic_number: 0x0100004202250800, shift: 57, mask: BitBoard(5629586008178688), },
    Magic { magic_number: 0x5200403091040200, shift: 57, mask: BitBoard(11259172008099840), },
    Magic { magic_number: 0x0071020292004100, shift: 57, mask: BitBoard(22518341868716544), },
    Magic { magic_number: 0x2008011404000090, shift: 59, mask: BitBoard(9007336962655232), },
    Magic { magic_number: 0x0001011212090380, shift: 59, mask: BitBoard(18014673925310464), },
    Magic { magic_number: 0x0424189824300010, shift: 59, mask: BitBoard(2216338399232), },
    Magic { magic_number: 0x2050249828080000, shift: 59, mask: BitBoard(4432676798464), },
    Magic { magic_number: 0x1020102108088210, shift: 59, mask: BitBoard(11064376819712), },
    Magic { magic_number: 0x1002200020881054, shift: 59, mask: BitBoard(22137335185408), },
    Magic { magic_number: 0x3b0b000831240200, shift: 59, mask: BitBoard(44272556441600), },
    Magic { magic_number: 0x8100a022024a0080, shift: 59, mask: BitBoard(87995357200384), },
    Magic { magic_number: 0x0004102408009000, shift: 59, mask: BitBoard(35253226045952), },
    Magic { magic_number: 0x00b0100208803400, shift: 59, mask: BitBoard(70506452091904), },
    Magic { magic_number: 0x60020600450808c0, shift: 58, mask: BitBoard(567382630219776), },
    Magic { magic_number: 0x00a0802208420800, shift: 59, mask: BitBoard(1134765260406784), },
    Magic { magic_number: 0x0000020100880400, shift: 59, mask: BitBoard(2832480465846272), },
    Magic { magic_number: 0x0000100800420200, shift: 59, mask: BitBoard(5667157807464448), },
    Magic { magic_number: 0x00001200a8070411, shift: 59, mask: BitBoard(11333774449049600), },
    Magic { magic_number: 0x0108844002844104, shift: 59, mask: BitBoard(22526811443298304), },
    Magic { magic_number: 0x2082420404008a01, shift: 59, mask: BitBoard(9024825867763712), },
    Magic { magic_number: 0x0002302a02004200, shift: 58, mask: BitBoard(18049651735527936), },
];

// TODO: document number 512
pub fn generate_bishop_attacks() -> Vec<[BitBoard; 64]> {
    // TODO: maybe flatten array
    let mut attacks = vec![[BitBoard(0); 64]; 512];

    let mut target_mapping: [BitBoardMapping; 512] = [BitBoardMapping {
        from: BitBoard(0),
        to: BitBoard(0),
    }; 512];

    let mut square = 0;
    while square < 64 {
        let sq = Square::from_index(square as u8);
        let relevant_occupancy_bit_mask = mask_bishop_relevant_occupancy(sq);
        let num_relevant_occupancy_bits = relevant_occupancy_bit_mask.popcnt();

        let num = 1 << num_relevant_occupancy_bits;

        let mut target_mapping_index = 0;
        while target_mapping_index < num {
            let occupancy = generate_occupancy(target_mapping_index, relevant_occupancy_bit_mask);
            let attacks = mask_bishop_attacks_on_the_fly(sq, occupancy);

            target_mapping[target_mapping_index as usize] = BitBoardMapping {
                from: occupancy,
                to: attacks,
            };
            target_mapping_index += 1;
        }

        let magic = BISHOP_MAGIC_NUMBERS[square];

        let mut mapping_index = 0;
        while mapping_index < num {
            let mapping = target_mapping[mapping_index as usize];
            let magic_index = (mapping.from.0.wrapping_mul(magic.magic_number)) >> magic.shift;
            attacks[magic_index as usize][square] = mapping.to;
            mapping_index += 1;
        }

        square += 1;
    }

    attacks
}

pub const fn mask_bishop_relevant_occupancy(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0).0;

    let (source_rank, source_file) = ((square.to_index() / 8) as i8, (square.to_index() % 8) as i8);

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    const DIAGONAL_CONFIGS: [(i8, i8, u64); 4] = [
        // top right
        (1, 1, BitBoard::NOT_8TH_RANK.0 & BitBoard::NOT_H_FILE.0),
        // top left
        (1, -1, BitBoard::NOT_8TH_RANK.0 & BitBoard::NOT_A_FILE.0),
        // bottom left
        (-1, -1, BitBoard::NOT_1ST_RANK.0 & BitBoard::NOT_A_FILE.0),
        // bottom right
        (-1, 1, BitBoard::NOT_1ST_RANK.0 & BitBoard::NOT_H_FILE.0),
    ];

    let mut distance = 1;
    while distance <= 6 {
        let mut diagonal_index = 0;
        while diagonal_index < DIAGONAL_CONFIGS.len() {
            let (dir_rank, dir_file, mask) = DIAGONAL_CONFIGS[diagonal_index];

            let (rank, file) = (
                source_rank + distance * dir_rank,
                source_file + distance * dir_file,
            );

            if rank < 8 && rank >= 0 && file < 8 && file >= 0 {
                // within the bounds of the board
                let square_bitboard = to_square_bitboard(rank, file);
                if (square_bitboard & mask) != 0 {
                    attacks |= square_bitboard;
                }
            }

            diagonal_index += 1;
        }

        distance += 1;
    }

    BitBoard(attacks)
}

pub const fn mask_bishop_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0).0;

    let (source_rank, source_file) = ((square.to_index() / 8) as i8, (square.to_index() % 8) as i8);

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    const DIAGONAL_CONFIGS: [(i8, i8); 4] = [
        // top right
        (1, 1),
        // top left
        (1, -1),
        // bottom left
        (-1, -1),
        // bottom right
        (-1, 1),
    ];

    // same order as DIAGONAL_CONFIGS
    let mut blocked_diagonal = [false; 4];

    let mut distance = 1;
    while distance <= 7 {
        let mut diagonal_index = 0;

        while diagonal_index < DIAGONAL_CONFIGS.len() {
            if blocked_diagonal[diagonal_index] {
                // skip blocked diagonal
                diagonal_index += 1;
                continue;
            }

            let (dir_rank, dir_file) = DIAGONAL_CONFIGS[diagonal_index];

            let (rank, file) = (
                source_rank + distance * dir_rank,
                source_file + distance * dir_file,
            );

            if rank < 8 && rank >= 0 && file < 8 && file >= 0 {
                // within the bounds of the board
                let square_bitboard = to_square_bitboard(rank, file);
                attacks |= square_bitboard;
                if (blockers.0 & square_bitboard) != BitBoard(0).0 {
                    blocked_diagonal[diagonal_index] = true;
                }
            }

            diagonal_index += 1;
        }

        distance += 1;
    }

    BitBoard(attacks)
}

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::square::Square;
    use crate::tables::bishop_move::{
        mask_bishop_attacks_on_the_fly, mask_bishop_relevant_occupancy,
    };

    #[test]
    fn test_bishop_relevant_occupancy_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 9] = [F5, G6, D5, C6, B7, D3, C2, F3, G2];
        for square in SQUARES {
            expected |= square;
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
            expected |= square;
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
            expected |= square;
        }
        let attacks = mask_bishop_relevant_occupancy(A8);
        println!("attacks: {attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_bishop_relevant_occupancy_a1() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 6] = [B2, C3, D4, E5, F6, G7];
        for square in SQUARES {
            expected |= square;
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
            expected |= square;
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
            expected |= square;
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
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= B7;
        blockers |= C2;
        blockers |= F5;
        blockers |= H1;
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
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= B2;
        let attacks = mask_bishop_attacks_on_the_fly(A1, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

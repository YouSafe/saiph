use crate::bitboard::BitBoard;
use crate::square::Square;
use crate::tables::magic_number::{generate_occupancy, Magic};

#[rustfmt::skip]
pub const ROOK_MAGIC_NUMBERS: [Magic; 64] = [
    Magic { magic_number: 0xa080004000201880, shift: 52, mask: BitBoard(282578800148862)  },
    Magic { magic_number: 0x0840100040002000, shift: 53, mask: BitBoard(565157600297596)  },
    Magic { magic_number: 0x1e800c2000100080, shift: 53, mask: BitBoard(1130315200595066)  },
    Magic { magic_number: 0x1080048010000802, shift: 53, mask: BitBoard(2260630401190006)  },
    Magic { magic_number: 0x0100100800040300, shift: 53, mask: BitBoard(4521260802379886)  },
    Magic { magic_number: 0x020003100c086200, shift: 53, mask: BitBoard(9042521604759646)  },
    Magic { magic_number: 0x040016b012041308, shift: 53, mask: BitBoard(18085043209519166)  },
    Magic { magic_number: 0x420002004183002c, shift: 52, mask: BitBoard(36170086419038334)  },
    Magic { magic_number: 0x0180800080400020, shift: 53, mask: BitBoard(282578800180736)  },
    Magic { magic_number: 0x0102401004200040, shift: 54, mask: BitBoard(565157600328704)  },
    Magic { magic_number: 0x1012002010420080, shift: 54, mask: BitBoard(1130315200625152)  },
    Magic { magic_number: 0x2102002190c00a00, shift: 54, mask: BitBoard(2260630401218048)  },
    Magic { magic_number: 0x802a001009042200, shift: 54, mask: BitBoard(4521260802403840)  },
    Magic { magic_number: 0x0202000891040200, shift: 54, mask: BitBoard(9042521604775424)  },
    Magic { magic_number: 0x0010802100220080, shift: 54, mask: BitBoard(18085043209518592)  },
    Magic { magic_number: 0x0801000040810002, shift: 53, mask: BitBoard(36170086419037696)  },
    Magic { magic_number: 0x0040208000804000, shift: 53, mask: BitBoard(282578808340736)  },
    Magic { magic_number: 0x0230104000200041, shift: 54, mask: BitBoard(565157608292864)  },
    Magic { magic_number: 0x0000888020031000, shift: 54, mask: BitBoard(1130315208328192)  },
    Magic { magic_number: 0x0250008080080010, shift: 54, mask: BitBoard(2260630408398848)  },
    Magic { magic_number: 0x2202050011010800, shift: 54, mask: BitBoard(4521260808540160)  },
    Magic { magic_number: 0x0001010008040002, shift: 54, mask: BitBoard(9042521608822784)  },
    Magic { magic_number: 0x0003040002080110, shift: 54, mask: BitBoard(18085043209388032)  },
    Magic { magic_number: 0x1000020010408124, shift: 53, mask: BitBoard(36170086418907136)  },
    Magic { magic_number: 0x0010800080204008, shift: 53, mask: BitBoard(282580897300736)  },
    Magic { magic_number: 0x8060002040005000, shift: 54, mask: BitBoard(565159647117824)  },
    Magic { magic_number: 0x6001024300102001, shift: 54, mask: BitBoard(1130317180306432)  },
    Magic { magic_number: 0x0040100080080084, shift: 54, mask: BitBoard(2260632246683648)  },
    Magic { magic_number: 0x1024040080080280, shift: 54, mask: BitBoard(4521262379438080)  },
    Magic { magic_number: 0x8804020080040080, shift: 54, mask: BitBoard(9042522644946944)  },
    Magic { magic_number: 0x5042004200085144, shift: 54, mask: BitBoard(18085043175964672)  },
    Magic { magic_number: 0x2010c08200004421, shift: 53, mask: BitBoard(36170086385483776)  },
    Magic { magic_number: 0x0000400028800082, shift: 53, mask: BitBoard(283115671060736)  },
    Magic { magic_number: 0x0000882004804000, shift: 54, mask: BitBoard(565681586307584)  },
    Magic { magic_number: 0x8010002800200401, shift: 54, mask: BitBoard(1130822006735872)  },
    Magic { magic_number: 0x4000801000800804, shift: 54, mask: BitBoard(2261102847592448)  },
    Magic { magic_number: 0x0444800800800400, shift: 54, mask: BitBoard(4521664529305600)  },
    Magic { magic_number: 0x0002002004040010, shift: 54, mask: BitBoard(9042787892731904)  },
    Magic { magic_number: 0x90c8100144000802, shift: 54, mask: BitBoard(18085034619584512)  },
    Magic { magic_number: 0x1002084902000084, shift: 53, mask: BitBoard(36170077829103616)  },
    Magic { magic_number: 0x0dc0802040008000, shift: 53, mask: BitBoard(420017753620736)  },
    Magic { magic_number: 0x4010002000404001, shift: 54, mask: BitBoard(699298018886144)  },
    Magic { magic_number: 0x28200100e0450030, shift: 54, mask: BitBoard(1260057572672512)  },
    Magic { magic_number: 0x0010010010210008, shift: 54, mask: BitBoard(2381576680245248)  },
    Magic { magic_number: 0x0200080004008080, shift: 54, mask: BitBoard(4624614895390720)  },
    Magic { magic_number: 0x0e24000200048080, shift: 54, mask: BitBoard(9110691325681664)  },
    Magic { magic_number: 0x8000210802a40030, shift: 54, mask: BitBoard(18082844186263552)  },
    Magic { magic_number: 0x20a0010040820004, shift: 53, mask: BitBoard(36167887395782656)  },
    Magic { magic_number: 0x0000204080010100, shift: 53, mask: BitBoard(35466950888980736)  },
    Magic { magic_number: 0x8020200098400180, shift: 54, mask: BitBoard(34905104758997504)  },
    Magic { magic_number: 0x4000200080100880, shift: 54, mask: BitBoard(34344362452452352)  },
    Magic { magic_number: 0x9100080010008080, shift: 54, mask: BitBoard(33222877839362048)  },
    Magic { magic_number: 0x0004040080080080, shift: 54, mask: BitBoard(30979908613181440)  },
    Magic { magic_number: 0x00ba001400800280, shift: 54, mask: BitBoard(26493970160820224)  },
    Magic { magic_number: 0x7002000144084200, shift: 54, mask: BitBoard(17522093256097792)  },
    Magic { magic_number: 0x4240800100004080, shift: 53, mask: BitBoard(35607136465616896)  },
    Magic { magic_number: 0x000300800a102041, shift: 52, mask: BitBoard(9079539427579068672)  },
    Magic { magic_number: 0x000d020440208012, shift: 53, mask: BitBoard(8935706818303361536)  },
    Magic { magic_number: 0x00c04058a0010013, shift: 53, mask: BitBoard(8792156787827803136)  },
    Magic { magic_number: 0x0011001000060821, shift: 53, mask: BitBoard(8505056726876686336)  },
    Magic { magic_number: 0x0011000800020411, shift: 53, mask: BitBoard(7930856604974452736)  },
    Magic { magic_number: 0x4082001004810802, shift: 53, mask: BitBoard(6782456361169985536)  },
    Magic { magic_number: 0x0000080082011004, shift: 53, mask: BitBoard(4485655873561051136)  },
    Magic { magic_number: 0x1011cc0100205082, shift: 52, mask: BitBoard(9115426935197958144)  },
];

// TODO: document number 4096
pub fn generate_rook_attacks() -> Vec<[BitBoard; 64]> {
    // TODO: maybe flatten array
    let mut attacks = vec![[BitBoard(0); 64]; 4096];

    let mut square = 0;
    while square < 64 {
        let sq = Square::from_index(square as u8);
        let relevant_occupancy_bit_mask = mask_rook_relevant_occupancy(sq);
        let num_relevant_occupancy_bits = relevant_occupancy_bit_mask.popcnt();

        let num = 1 << num_relevant_occupancy_bits;

        let magic = ROOK_MAGIC_NUMBERS[square];

        let mut target_mapping_index = 0;
        while target_mapping_index < num {
            let occupancy = generate_occupancy(target_mapping_index, relevant_occupancy_bit_mask);
            let attack = mask_rook_attacks_on_the_fly(sq, occupancy);

            let magic_index = (occupancy.0.wrapping_mul(magic.magic_number)) >> magic.shift;
            attacks[magic_index as usize][square] = attack;

            target_mapping_index += 1;
        }
        square += 1;
    }

    attacks
}

pub const fn mask_rook_relevant_occupancy(square: Square) -> BitBoard {
    let BitBoard(mut attacks) = BitBoard(0);

    let (source_rank, source_file) = ((square.to_index() / 8) as i8, (square.to_index() % 8) as i8);

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    const LINE_CONFIGS: [(i8, i8, u64); 4] = [
        // right
        (0, 1, BitBoard::NOT_H_FILE.0),
        // up
        (1, 0, BitBoard::NOT_8TH_RANK.0),
        // left
        (0, -1, BitBoard::NOT_A_FILE.0),
        // down
        (-1, 0, BitBoard::NOT_1ST_RANK.0),
    ];

    let mut line_index = 0;
    while line_index < LINE_CONFIGS.len() {
        let mut distance = 1;
        while distance <= 6 {
            let (dir_rank, dir_file, mask) = LINE_CONFIGS[line_index];

            let (rank, file) = (
                source_rank + distance * dir_rank,
                source_file + distance * dir_file,
            );

            if rank < 8 && rank >= 0 && file < 8 && file >= 0 {
                // within the bounds of the board
                let square_bitboard = to_square_bitboard(rank, file);
                if (square_bitboard & mask) != 0 {
                    attacks |= square_bitboard;
                } else {
                    break;
                }
            } else {
                break;
            }

            distance += 1;
        }
        line_index += 1;
    }

    BitBoard(attacks)
}

pub const fn mask_rook_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard(0).0;

    let (source_rank, source_file) = ((square.to_index() / 8) as i8, (square.to_index() % 8) as i8);

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    const LINE_CONFIGS: [(i8, i8); 4] = [
        // right
        (0, 1),
        // up
        (1, 0),
        // left
        (0, -1),
        // down
        (-1, 0),
    ];

    let mut line_index = 0;
    while line_index < LINE_CONFIGS.len() {
        let mut distance = 1;
        while distance <= 7 {
            let (dir_rank, dir_file) = LINE_CONFIGS[line_index];

            let (rank, file) = (
                source_rank + distance * dir_rank,
                source_file + distance * dir_file,
            );

            if rank < 8 && rank >= 0 && file < 8 && file >= 0 {
                // within the bounds of the board
                let square_bitboard = to_square_bitboard(rank, file);
                attacks |= square_bitboard;
                if (blockers.0 & square_bitboard) != BitBoard(0).0 {
                    break;
                }
            } else {
                break;
            }
            distance += 1;
        }
        line_index += 1;
    }

    BitBoard(attacks)
}

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::square::Square;
    use crate::tables::rook_move::{
        generate_rook_attacks, mask_rook_attacks_on_the_fly, mask_rook_relevant_occupancy,
        ROOK_MAGIC_NUMBERS,
    };

    #[test]
    fn test_rook_relevant_occupancy_e4() {
        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 11] = [B4, C4, D4, E6, F4, G4, E6, E7, E3, E2, E5];
        for square in SQUARES {
            expected |= square;
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
            expected |= square;
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
            expected |= square;
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
            expected |= square;
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
            expected |= square;
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
            expected |= square;
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
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= D4;
        blockers |= E2;
        blockers |= H4;
        blockers |= E7;
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
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= A2;
        blockers |= B1;
        let attacks = mask_rook_attacks_on_the_fly(A1, blockers);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    pub fn test_rook_attack_table() {
        let rook_attacks = generate_rook_attacks();

        let mut expected = BitBoard(0);
        use Square::*;
        const SQUARES: [Square; 9] = [D4, E3, E2, F4, G4, H4, E5, E6, E7];
        for square in SQUARES {
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= D4;
        blockers |= E2;
        blockers |= H4;
        blockers |= E7;

        println!("{blockers}");

        let square = E4;

        let magic = ROOK_MAGIC_NUMBERS[square as usize];

        blockers &= mask_rook_relevant_occupancy(square);
        blockers = blockers * magic.magic_number;
        blockers = blockers >> magic.shift as i32;

        let attacks = rook_attacks[blockers.0 as usize][square as usize];
        println!("{expected}");
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

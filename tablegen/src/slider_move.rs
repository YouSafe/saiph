use crate::magics::{BISHOP_MAGICS, ROOK_MAGICS, SLIDER_ATTACK_TABLE_SIZE};
use types::bitboard::BitBoard;

pub const fn generate_slider_attacks() -> [BitBoard; SLIDER_ATTACK_TABLE_SIZE] {
    let mut attacks: [BitBoard; SLIDER_ATTACK_TABLE_SIZE] = [BitBoard(0); SLIDER_ATTACK_TABLE_SIZE];

    let mut square = 0;
    while square < 64 {
        // bishop attacks
        {
            let magic = &BISHOP_MAGICS[square];
            let index = magic.mask;
            let mut occupancy: u64 = 0;
            loop {
                let attack = mask_bishop_attacks_on_the_fly_const(square as i8, occupancy);
                let magic_index = (occupancy.wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
                attacks[magic_index as usize] = BitBoard(attack);
                occupancy = occupancy.wrapping_sub(index) & index;
                if occupancy == 0 {
                    break;
                }
            }
        }
        // rook attacks
        {
            let magic = &ROOK_MAGICS[square];
            let index = magic.mask;
            let mut occupancy: u64 = 0;
            loop {
                let attack = mask_rook_attacks_on_the_fly_const(square as i8, occupancy);
                let magic_index = (occupancy.wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
                attacks[magic_index as usize] = BitBoard(attack);
                occupancy = occupancy.wrapping_sub(index) & index;
                if occupancy == 0 {
                    break;
                }
            }
        }
        square += 1;
    }

    attacks
}

pub const fn mask_slider_one_direction<const SQUARE_CHANGE: i8>(square: i8, blockers: u64) -> u64 {
    let mut attacks = 0;
    let mut previous_square = square;
    loop {
        let current_square = previous_square + SQUARE_CHANGE;

        let file_difference = (previous_square % 8) - (current_square % 8);
        if current_square > 63 || current_square < 0 || file_difference > 2 || file_difference < -2
        {
            break;
        }

        let bitboard = 1 << current_square;
        attacks |= bitboard;
        if (blockers & bitboard) != 0 {
            break;
        }
        previous_square = current_square;
    }

    attacks
}

// only consider the middle 6 files since blockers on the edge do not affect the attacks
pub static FIRST_RANK_ATTACKS: [[u8; 64]; 8] = {
    let mut file: [[u8; 64]; 8] = [[0; 64]; 8];

    let mut square = 0;
    while square < 8 {
        let mut blockers = 0;
        while blockers < 64 {
            file[square as usize][blockers as usize] =
                (mask_slider_one_direction::<1>(square, blockers << 1)
                    | mask_slider_one_direction::<-1>(square, blockers << 1)) as u8;
            blockers += 1;
        }
        square += 1;
    }

    file
};

#[derive(Debug, Clone, Copy)]
pub struct BishopMask {
    bit: u64,
    swap: u64,
    main_diag: u64,
    anti_diag: u64,
}

pub static BISHOP_MASK: [BishopMask; 64] = {
    let mut masks = [BishopMask {
        bit: 0,
        swap: 0,
        main_diag: 0,
        anti_diag: 0,
    }; 64];

    let mut square = 0;
    while square < 64 {
        let bit = 1u64 << square;
        let file = square & 7;
        let rank = square >> 3;

        masks[square] = BishopMask {
            bit,
            swap: bit.swap_bytes(),
            main_diag: bit ^ BitBoard::DIAGS[7 + file - rank].0,
            anti_diag: bit ^ BitBoard::DIAGS[file + rank].0.swap_bytes(),
        };

        square += 1;
    }

    masks
};

pub const fn mask_slider_diagonals(square: i8, blockers: u64) -> u64 {
    let mask = BISHOP_MASK[square as usize];

    let mut diag = blockers & mask.main_diag;
    let mut rev1 = diag.swap_bytes();
    diag = diag.wrapping_sub(mask.bit);
    rev1 = rev1.wrapping_sub(mask.swap);
    diag ^= rev1.swap_bytes();
    diag &= mask.main_diag;

    let mut anti = blockers & mask.anti_diag;
    let mut rev2 = anti.swap_bytes();
    anti = anti.wrapping_sub(mask.bit);
    rev2 = rev2.wrapping_sub(mask.swap);
    anti ^= rev2.swap_bytes();
    anti &= mask.anti_diag;

    diag | anti
}

pub const fn mask_slider_vertical(square: i8, blockers: u64) -> u64 {
    let file_index = square & 7;
    let BitBoard(mask) = BitBoard::ALL_FILES[file_index as usize];
    let bit = 1u64 << square;

    let mut forward = blockers & mask;
    let mut reverse = forward.swap_bytes();
    forward = forward.wrapping_sub(bit);
    reverse = reverse.wrapping_sub(bit.swap_bytes());
    forward ^= reverse.swap_bytes();
    forward &= mask;

    forward
}

pub const fn mask_slider_horizontal(square: i8, blockers: u64) -> u64 {
    let rank_index = square >> 3;
    let BitBoard(mask) = BitBoard::ALL_RANKS[rank_index as usize];
    
    let o = blockers;
    let s = 1u64 << square;
    assert!(square < 64);

    // mask and map to first rank
    let o = (o & mask) >> (rank_index * 8);
    let s = (s & mask) >> (rank_index * 8);
    
    let a = FIRST_RANK_ATTACKS[s.trailing_zeros() as usize][((o >> 1) & 0x3F) as usize] as u64;

    // unmap from first rank
    a << (rank_index * 8)
}

const fn mask_rook_attacks_on_the_fly_const(square: i8, blockers: u64) -> u64 {
    mask_slider_horizontal(square, blockers) | mask_slider_vertical(square, blockers)
}

const fn mask_bishop_attacks_on_the_fly_const(square: i8, blockers: u64) -> u64 {
    mask_slider_diagonals(square, blockers)
}

#[cfg(test)]
mod test {
    use types::square::Square;

    use super::*;

    static SLIDER_ATTACKS: [BitBoard; SLIDER_ATTACK_TABLE_SIZE] = generate_slider_attacks();

    pub fn mask_rook_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
        BitBoard(mask_rook_attacks_on_the_fly_const(
            square.to_index() as i8,
            blockers.0,
        ))
    }

    pub fn mask_bishop_attacks_on_the_fly(square: Square, blockers: BitBoard) -> BitBoard {
        BitBoard(mask_bishop_attacks_on_the_fly_const(
            square.to_index() as i8,
            blockers.0,
        ))
    }

    pub fn get_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
        let magic = &BISHOP_MAGICS[square as usize];
        let magic_index =
            ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
        SLIDER_ATTACKS[magic_index as usize]
    }

    pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
        let magic = &ROOK_MAGICS[square as usize];
        let magic_index =
            ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
        SLIDER_ATTACKS[magic_index as usize]
    }

    #[test]
    fn test_bishop_attack_on_the_fly_e4() {
        let mut expected = BitBoard(0);
        use types::square::Square::*;
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
        use types::square::Square::*;
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

    #[test]
    fn test_rook_attacks_on_the_fly_e4() {
        let mut expected = BitBoard(0);
        use types::square::Square::*;
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
        use types::square::Square::*;
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
        let mut expected = BitBoard(0);
        use types::square::Square::*;
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

        let attacks = get_rook_attacks(E4, blockers);

        println!("{expected}");
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    pub fn test_bishop_attack_table() {
        let mut expected = BitBoard(0);
        use types::square::Square::*;
        const SQUARES: [Square; 9] = [F5, D5, C6, B7, D3, C2, F3, G2, H1];
        for square in SQUARES {
            expected |= square;
        }
        let mut blockers = BitBoard(0);
        blockers |= B7;
        blockers |= C2;
        blockers |= F5;
        blockers |= H1;

        println!("{blockers}");

        let attacks = get_bishop_attacks(E4, blockers);

        println!("{expected}");
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

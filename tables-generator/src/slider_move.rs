use crate::{magics::{BISHOP_MAGICS, ROOK_MAGICS, SLIDER_ATTACK_TABLE_SIZE}, BitBoard};

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

const fn mask_slider_one_direction<const SQUARE_CHANGE: i8>(square: i8, blockers: u64) -> u64 {
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

const fn mask_rook_attacks_on_the_fly_const(square: i8, blockers: u64) -> u64 {
    mask_slider_one_direction::<1>(square, blockers) // right
        | mask_slider_one_direction::<8>(square, blockers) // up
        | mask_slider_one_direction::<-1>(square, blockers) // left
        | mask_slider_one_direction::<-8>(square, blockers) // down
}

const fn mask_bishop_attacks_on_the_fly_const(square: i8, blockers: u64) -> u64 {
    mask_slider_one_direction::<9>(square, blockers) // top right
        | mask_slider_one_direction::<7>(square, blockers) // top left
        | mask_slider_one_direction::<-9>(square, blockers) // bottom left
        | mask_slider_one_direction::<-7>(square, blockers) // bottom right
}

#[cfg(test)]
mod test {
    use crate::magics::BISHOP_MAGICS;
    use crate::magics::ROOK_MAGICS;
    use crate::magics::SLIDER_ATTACK_TABLE_SIZE;
    use crate::BitBoard;
    use crate::Square;

    use super::generate_slider_attacks;
    use super::mask_bishop_attacks_on_the_fly_const;
    use super::mask_rook_attacks_on_the_fly_const;

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
        let magic_index = ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 9)) + magic.offset;
        SLIDER_ATTACKS[magic_index as usize]
    }
    
    pub fn get_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
        let magic = &ROOK_MAGICS[square as usize];
        let magic_index = ((blockers.0 & magic.mask).wrapping_mul(magic.magic) >> (64 - 12)) + magic.offset;
        SLIDER_ATTACKS[magic_index as usize]
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

        let attacks = get_rook_attacks(E4, blockers);

        println!("{expected}");
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    pub fn test_bishop_attack_table() {
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

        println!("{blockers}");

        let attacks = get_bishop_attacks(E4, blockers);

        println!("{expected}");
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

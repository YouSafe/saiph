use crate::BitBoard;
use crate::Square;

pub const fn generate_knight_attacks() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    let mut square = 0;
    while square < 64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_knight_attack(sq);
        square += 1;
    }

    result
}

const fn mask_knight_attack(square: Square) -> BitBoard {
    let BitBoard(mut attacks) = BitBoard(0);

    let BitBoard(bitboard) = BitBoard::from_square(square);

    // right shift, mask
    const MOVES: [(i8, BitBoard); 8] = [
        // two down left
        (17, BitBoard::NOT_H_FILE),
        // two down right
        (15, BitBoard::NOT_A_FILE),
        // two left down
        (10, BitBoard::NOT_GH_FILE),
        // two right down
        (6, BitBoard::NOT_AB_FILE),
        // two left up
        (-6, BitBoard::NOT_GH_FILE),
        // two right up
        (-10, BitBoard::NOT_AB_FILE),
        // two up right
        (-15, BitBoard::NOT_H_FILE),
        // two up left
        (-17, BitBoard::NOT_A_FILE),
    ];

    let mut moves_index = 0;
    while moves_index < MOVES.len() {
        let (shift, BitBoard(mask)) = MOVES[moves_index];

        let shifted = if shift > 0 {
            bitboard >> shift as i32
        } else {
            bitboard << shift.abs() as i32
        };

        if (shifted & mask) != BitBoard(0).0 {
            attacks |= shifted;
        }

        moves_index += 1;
    }

    BitBoard(attacks)
}

#[cfg(test)]
mod test {
    use crate::knight_move::mask_knight_attack;
    use crate::BitBoard;
    use crate::Square;

    #[test]
    fn test_knight_attack_a1() {
        let mut expected = BitBoard(0);
        expected |= Square::B3;
        expected |= Square::C2;
        let attacks = mask_knight_attack(Square::A1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_knight_attack_a8() {
        let mut expected = BitBoard(0);
        expected |= Square::B6;
        expected |= Square::C7;
        let attacks = mask_knight_attack(Square::A8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_knight_attack_h1() {
        let mut expected = BitBoard(0);
        expected |= Square::F2;
        expected |= Square::G3;
        let attacks = mask_knight_attack(Square::H1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_knight_attack_h8() {
        let mut expected = BitBoard(0);
        expected |= Square::F7;
        expected |= Square::G6;
        let attacks = mask_knight_attack(Square::H8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_knight_attack_e4() {
        let mut expected = BitBoard(0);
        expected |= Square::F6;
        expected |= Square::G5;
        expected |= Square::G3;
        expected |= Square::F2;
        expected |= Square::D2;
        expected |= Square::C3;
        expected |= Square::C5;
        expected |= Square::D6;
        let attacks = mask_knight_attack(Square::E4);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_knight_attack_h5() {
        let mut expected = BitBoard(0);
        expected |= Square::G7;
        expected |= Square::F6;
        expected |= Square::F4;
        expected |= Square::G3;
        let attacks = mask_knight_attack(Square::H5);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

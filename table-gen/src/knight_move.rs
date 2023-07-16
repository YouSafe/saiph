use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_knight_attacks() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_knight_attack(sq);
    }

    result
}

fn mask_knight_attack(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);

    let bitboard = BitBoard::from_square(square);

    const MOVES: [(fn(BitBoard) -> BitBoard, BitBoard); 8] = [
        // two down left
        (|bb: BitBoard| bb >> 17, BitBoard::NOT_H_FILE),
        // two down right
        (|bb: BitBoard| bb >> 15, BitBoard::NOT_A_FILE),
        // two left down
        (|bb: BitBoard| bb >> 10, BitBoard::NOT_GH_FILE),
        // two right down
        (|bb: BitBoard| bb >> 6, BitBoard::NOT_AB_FILE),
        // two left up
        (|bb: BitBoard| bb << 6, BitBoard::NOT_GH_FILE),
        // two right up
        (|bb: BitBoard| bb << 10, BitBoard::NOT_AB_FILE),
        // two up right
        (|bb: BitBoard| bb << 15, BitBoard::NOT_H_FILE),
        // two up left
        (|bb: BitBoard| bb << 17, BitBoard::NOT_A_FILE),
    ];

    for (shifter, mask) in MOVES {
        let shifted = shifter(bitboard);
        if (shifted & mask) != BitBoard(0) {
            attacks |= shifted;
        }
    }

    attacks
}

#[cfg(test)]
mod test {
    use crate::knight_move::mask_knight_attack;
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

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

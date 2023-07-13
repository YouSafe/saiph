use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_king_attacks() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_king_attacks(sq);
    }

    result
}

fn mask_king_attacks(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0);

    let bitboard = BitBoard::from_square(square);

    const MOVES: [(fn(BitBoard) -> BitBoard, BitBoard); 8] = [
        // bottom left
        (|bb: BitBoard| bb >> 9, BitBoard::NOT_H_FILE),
        // bottom
        (|bb: BitBoard| bb >> 8, BitBoard(!0)),
        // bottom right
        (|bb: BitBoard| bb >> 7, BitBoard::NOT_A_FILE),
        // left
        (|bb: BitBoard| bb >> 1, BitBoard::NOT_H_FILE),
        // right
        (|bb: BitBoard| bb << 1, BitBoard::NOT_A_FILE),
        // top left
        (|bb: BitBoard| bb << 7, BitBoard::NOT_H_FILE),
        // top
        (|bb: BitBoard| bb << 8, BitBoard(!0)),
        // top right
        (|bb: BitBoard| bb << 9, BitBoard::NOT_A_FILE),
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
    use crate::king_move::mask_king_attacks;
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

    #[test]
    fn test_king_attack_a1() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::B1);
        expected |= BitBoard::from_square(Square::B2);
        expected |= BitBoard::from_square(Square::A2);
        let attacks = mask_king_attacks(Square::A1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_a8() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::A7);
        expected |= BitBoard::from_square(Square::B7);
        expected |= BitBoard::from_square(Square::B8);
        let attacks = mask_king_attacks(Square::A8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h1() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::H2);
        expected |= BitBoard::from_square(Square::G2);
        expected |= BitBoard::from_square(Square::G1);
        let attacks = mask_king_attacks(Square::H1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h8() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::H7);
        expected |= BitBoard::from_square(Square::G8);
        expected |= BitBoard::from_square(Square::G7);
        let attacks = mask_king_attacks(Square::H8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_e4() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::D5);
        expected |= BitBoard::from_square(Square::E5);
        expected |= BitBoard::from_square(Square::F5);
        expected |= BitBoard::from_square(Square::D4);
        expected |= BitBoard::from_square(Square::F4);
        expected |= BitBoard::from_square(Square::D3);
        expected |= BitBoard::from_square(Square::E3);
        expected |= BitBoard::from_square(Square::F3);
        let attacks = mask_king_attacks(Square::E4);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h5() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::H6);
        expected |= BitBoard::from_square(Square::H4);
        expected |= BitBoard::from_square(Square::G6);
        expected |= BitBoard::from_square(Square::G5);
        expected |= BitBoard::from_square(Square::G4);
        let attacks = mask_king_attacks(Square::H5);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

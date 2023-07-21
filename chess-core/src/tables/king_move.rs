use crate::bitboard::BitBoard;
use crate::square::Square;

pub const fn generate_king_attacks() -> [BitBoard; 64] {
    let mut result = [BitBoard(0); 64];

    let mut square = 0;
    while square < 64 {
        let sq = Square::from_index(square as u8);
        result[square] = mask_king_attacks(sq);
        square += 1;
    }

    result
}

const fn mask_king_attacks(square: Square) -> BitBoard {
    let mut attacks = BitBoard(0).0;

    let BitBoard(bitboard) = BitBoard::from_square(square);

    // right shift, mask
    const MOVES: [(i8, BitBoard); 8] = [
        // bottom left
        (9, BitBoard::NOT_H_FILE),
        // bottom
        (8, BitBoard(!0)),
        // bottom right
        (7, BitBoard::NOT_A_FILE),
        // left
        (1, BitBoard::NOT_H_FILE),
        // right
        (-1, BitBoard::NOT_A_FILE),
        // top left
        (-7, BitBoard::NOT_H_FILE),
        // top
        (-8, BitBoard(!0)),
        // top right
        (-9, BitBoard::NOT_A_FILE),
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
    use crate::bitboard::BitBoard;
    use crate::square::Square;
    use crate::tables::king_move::mask_king_attacks;

    #[test]
    fn test_king_attack_a1() {
        let mut expected = BitBoard(0);
        expected |= Square::B1;
        expected |= Square::B2;
        expected |= Square::A2;
        let attacks = mask_king_attacks(Square::A1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_a8() {
        let mut expected = BitBoard(0);
        expected |= Square::A7;
        expected |= Square::B7;
        expected |= Square::B8;
        let attacks = mask_king_attacks(Square::A8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h1() {
        let mut expected = BitBoard(0);
        expected |= Square::H2;
        expected |= Square::G2;
        expected |= Square::G1;
        let attacks = mask_king_attacks(Square::H1);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h8() {
        let mut expected = BitBoard(0);
        expected |= Square::H7;
        expected |= Square::G8;
        expected |= Square::G7;
        let attacks = mask_king_attacks(Square::H8);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_e4() {
        let mut expected = BitBoard(0);
        expected |= Square::D5;
        expected |= Square::E5;
        expected |= Square::F5;
        expected |= Square::D4;
        expected |= Square::F4;
        expected |= Square::D3;
        expected |= Square::E3;
        expected |= Square::F3;
        let attacks = mask_king_attacks(Square::E4);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_king_attack_h5() {
        let mut expected = BitBoard(0);
        expected |= Square::H6;
        expected |= Square::H4;
        expected |= Square::G6;
        expected |= Square::G5;
        expected |= Square::G4;
        let attacks = mask_king_attacks(Square::H5);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

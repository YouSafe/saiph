use crate::{BitBoard, Color, Square};

pub fn generate_pawn_attacks() -> [[BitBoard; 64]; 2] {
    let mut result = [[BitBoard(0); 64]; 2];

    for square in 0..64 {
        let sq = Square::from_index(square as u8);
        result[Color::White as usize][square] = mask_pawn_attacks(sq, Color::White);
        result[Color::Black as usize][square] = mask_pawn_attacks(sq, Color::Black);
    }

    result
}

fn mask_pawn_attacks(square: Square, side: Color) -> BitBoard {
    let mut attacks = BitBoard(0);

    let bitboard = BitBoard::from_square(square);

    // left and right from the point of view of the white player
    const MOVES: [[(i8, BitBoard); 2]; 2] = [
        // White Moves
        [(-7, BitBoard::NOT_H_FILE), (-9, BitBoard::NOT_A_FILE)],
        // Black Moves
        [(9, BitBoard::NOT_H_FILE), (7, BitBoard::NOT_A_FILE)],
    ];

    for (shift, mask) in MOVES[side as usize] {
        let shifted = bitboard.shifted(shift);
        if (shifted & mask) != BitBoard(0) {
            attacks |= shifted;
        }
    }

    attacks
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pawn_attack_white_e4() {
        let mut expected = BitBoard(0);
        expected |= Square::D5;
        expected |= Square::F5;
        let attacks = mask_pawn_attacks(Square::E4, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_white_a1() {
        let mut expected = BitBoard(0);
        expected |= Square::B2;
        let attacks = mask_pawn_attacks(Square::A1, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_white_h1() {
        let mut expected = BitBoard(0);
        expected |= Square::G2;
        let attacks = mask_pawn_attacks(Square::H1, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_e6() {
        let mut expected = BitBoard(0);
        expected |= Square::D5;
        expected |= Square::F5;
        let attacks = mask_pawn_attacks(Square::E6, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_a8() {
        let mut expected = BitBoard(0);
        expected |= Square::B7;
        let attacks = mask_pawn_attacks(Square::A8, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_h8() {
        let mut expected = BitBoard(0);
        expected |= Square::G7;
        let attacks = mask_pawn_attacks(Square::H8, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

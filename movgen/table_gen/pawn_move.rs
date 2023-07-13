use std::fs::File;
use std::io::Write;

use core::bitboard::BitBoard;
use core::color::Color;
use core::square::Square;

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
    let (left_diagonal, right_diagonal) = match side {
        Color::White => (bitboard >> 7, bitboard >> 9),
        Color::Black => (bitboard << 9, bitboard << 7),
    };

    if (left_diagonal & BitBoard::NOT_A_FILE) != BitBoard(0) {
        attacks |= left_diagonal;
    }
    if (right_diagonal & BitBoard::NOT_H_FILE) != BitBoard(0) {
        attacks |= right_diagonal;
    }

    attacks
}

pub fn write_pawn_attacks(
    file: &mut File,
    pawn_attacks: &[[BitBoard; 64]; 2],
) -> std::io::Result<()> {
    writeln!(file, "const PAWN_ATTACKS: [[BitBoard; 64]; 2] = [")?;
    for attacks_for_color in pawn_attacks {
        writeln!(file, "\t[")?;
        for board in attacks_for_color {
            writeln!(file, "\t\tBitBoard({}), ", board.0)?;
        }
        writeln!(file, "\t],")?
    }
    writeln!(file, "];")
}

#[cfg(test)]
mod test {
    use crate::bitboard::BitBoard;
    use crate::color::Color;
    use crate::pawn_move::mask_pawn_attacks;
    use crate::square::Square;
    use crate::table_gen::pawn_move::mask_pawn_attacks;

    #[test]
    fn test_pawn_attack_white_e4() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::D5);
        expected |= BitBoard::from_square(Square::F5);
        let attacks = mask_pawn_attacks(Square::E4, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_white_a1() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::B2);
        let attacks = mask_pawn_attacks(Square::A1, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_white_h1() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::G2);
        let attacks = mask_pawn_attacks(Square::H1, Color::White);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_e6() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::D5);
        expected |= BitBoard::from_square(Square::F5);
        let attacks = mask_pawn_attacks(Square::E6, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_a8() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::B7);
        let attacks = mask_pawn_attacks(Square::A8, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }

    #[test]
    fn test_pawn_attack_black_h8() {
        let mut expected = BitBoard(0);
        expected |= BitBoard::from_square(Square::G7);
        let attacks = mask_pawn_attacks(Square::H8, Color::Black);
        println!("{attacks}");
        assert_eq!(expected, attacks);
    }
}

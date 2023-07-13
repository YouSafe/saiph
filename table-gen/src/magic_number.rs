use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_occupancy(index: u64, mut attack_mask: BitBoard) -> BitBoard {
    // TODO: create function for counting set bits in bitboard
    let bit_set_in_attack_mask = attack_mask.0.count_ones();
    let mut occupancy = BitBoard(0);

    // TODO: create masked iterator
    for i in 0..bit_set_in_attack_mask {
        // index of first set least significant bit
        let square = attack_mask.0.trailing_zeros();

        // TODO: add bitwise operations with Squares
        attack_mask.0 ^= 1 << square;

        if (index & (1 << i)) != 0 {
            occupancy |= BitBoard::from_square(Square::from_index(square as u8));
        }
    }

    occupancy
}

#[cfg(test)]
mod test {
    use crate::magic_number::generate_occupancy;
    use chess_core::bitboard::BitBoard;

    #[test]
    fn test_generate_occupancy() {
        let attack_mask = BitBoard(7);

        println!("{attack_mask}");

        for i in 0..8 {
            let occupancy = generate_occupancy(i, attack_mask);
            println!("{occupancy}");
            assert_eq!(occupancy, BitBoard(i));
        }
    }
}

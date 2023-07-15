use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_occupancy(index: u64, mut attack_mask: BitBoard) -> BitBoard {
    let bit_set_in_attack_mask = attack_mask.popcnt();
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

#[derive(Debug, Default, Clone, Copy)]
pub struct Magic {
    pub magic_number: u64,
    pub shift: u8,
    pub mask: BitBoard,
}

#[derive(Debug, Clone)]
pub struct BitBoardMapping {
    pub from: BitBoard,
    pub to: BitBoard,
}

pub fn find_magic_number(
    rng: &mut MagicNumberCandidateGenerator,
    attack_mask: BitBoard,
    target_mapping: &[BitBoardMapping],
    relevant_bits: u8,
) -> Magic {
    let mut used_attacks: Vec<Option<BitBoard>> = vec![None; target_mapping.len()];

    loop {
        let magic_number: u64 = rng.next_magic();
        if ((attack_mask * magic_number).0 & 0xFF00_0000_0000_0000).count_ones() < 6 {
            continue;
        }

        used_attacks.fill(None);

        let mut failed = false;

        for BitBoardMapping { from, to } in target_mapping.iter().cloned() {
            let magic_index = (from * magic_number).0 >> (64 - relevant_bits);

            match used_attacks[magic_index as usize] {
                None => used_attacks[magic_index as usize] = Some(to),
                Some(attack) => {
                    if attack != to {
                        failed = true;
                        break;
                    }
                }
            }
        }

        if !failed {
            return Magic {
                magic_number,
                shift: (64 - relevant_bits),
                mask: attack_mask,
            };
        }
    }
}

pub struct MagicNumberCandidateGenerator {
    state: u64,
}

impl MagicNumberCandidateGenerator {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_magic(&mut self) -> u64 {
        // this makes that the random number contains more 0 bits
        self.next() & self.next() & self.next()
    }

    fn next(&mut self) -> u64 {
        // See: https://en.wikipedia.org/wiki/Xorshift
        let mut number = self.state;
        number ^= number << 13;
        number ^= number >> 7;
        number ^= number << 17;

        self.state = number;
        number
    }
}

#[cfg(test)]
mod test {
    use crate::magic_number::{
        find_magic_number, generate_occupancy, BitBoardMapping, MagicNumberCandidateGenerator,
    };
    use crate::rook_move::{mask_rook_attacks_on_the_fly, mask_rook_relevant_occupancy};
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

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

    #[test]
    fn test_find_magic_number() {
        let square = Square::E4;
        let blockers = BitBoard(4503601909075968);
        let expected = BitBoard(4521264426258432);

        let mut random = MagicNumberCandidateGenerator::new(1804289383);
        let attack_mask = mask_rook_relevant_occupancy(square);
        let relevant_bits = attack_mask.popcnt();

        let target_mapping = (0..(1u64 << relevant_bits))
            .map(|index| {
                let occupancy = generate_occupancy(index, attack_mask);
                let attacks = mask_rook_attacks_on_the_fly(square, occupancy);

                BitBoardMapping {
                    from: occupancy,
                    to: attacks,
                }
            })
            .collect::<Vec<_>>();

        let magic = find_magic_number(&mut random, attack_mask, &target_mapping, relevant_bits);

        let mut reordered_attacks = vec![BitBoard::default(); 4096];
        for mapping in target_mapping {
            let magic_index = (mapping.from * magic.magic_number).0 >> magic.shift;
            reordered_attacks[magic_index as usize] = mapping.to;
        }

        println!("{:#x}", magic.magic_number);

        println!("blockers: {blockers}");
        println!("attack_mask: {attack_mask}");

        let key = blockers & attack_mask;
        println!("key: {key}");

        let magic_index = (key * magic.magic_number).0 >> magic.shift;
        println!("magic_index: {magic_index}");

        let actual = reordered_attacks[magic_index as usize];

        println!("expected: {expected}");
        println!("actual: {actual}");

        assert_eq!(expected, actual);
    }
}

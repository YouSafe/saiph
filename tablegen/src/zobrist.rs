use crate::{NUM_CASTLING_RIGHTS_CONFIGURATIONS, NUM_COLORS, NUM_FILES, NUM_PIECES, NUM_SQUARES};

#[repr(C)]
pub struct GeneratedKeys {
    pub piece_keys: [[[u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS],
    pub en_passant_keys: [u64; NUM_FILES],
    pub castle_keys: [u64; NUM_CASTLING_RIGHTS_CONFIGURATIONS],
    pub side_key: u64,
}

pub fn generate_keys() -> GeneratedKeys {
    let mut random_gen = RandomNumberGenerator::new(465864546584658);

    let mut piece_keys = [[[0u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS];
    for square in piece_keys.iter_mut().flatten().flatten() {
        *square = random_gen.next();
    }

    let mut en_passant_keys = [0u64; NUM_FILES];
    for item in en_passant_keys.iter_mut() {
        *item = random_gen.next();
    }

    let mut castle_keys = [0u64; NUM_CASTLING_RIGHTS_CONFIGURATIONS];
    for item in castle_keys.iter_mut() {
        *item = random_gen.next();
    }

    let side_key = random_gen.next();

    GeneratedKeys {
        piece_keys,
        en_passant_keys,
        castle_keys,
        side_key,
    }
}

pub struct RandomNumberGenerator {
    state: u64,
}

impl RandomNumberGenerator {
    pub fn new(seed: u64) -> Self {
        assert!(seed != 0);
        Self { state: seed }
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

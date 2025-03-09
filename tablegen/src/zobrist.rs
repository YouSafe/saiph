use crate::{NUM_CASTLING_CONFIGS, NUM_COLORS, NUM_FILES, NUM_PIECES, NUM_SQUARES};

#[repr(C)]
pub struct GeneratedKeys {
    piece_keys: [[[u64; 64]; 6]; 2],
    en_passant_keys: [u64; 8],
    castle_keys: [u64; 16],
    side_key: u64,
}

pub fn generate_keys() -> GeneratedKeys {
    let mut generated = GeneratedKeys {
        piece_keys: [[[0u64; 64]; 6]; 2],
        en_passant_keys: [0u64; 8],
        castle_keys: [0u64; 16],
        side_key: 0,
    };

    let mut random_gen = RandomNumberGenerator::new(465864546584658);

    for piece_type in 0..NUM_PIECES {
        for color in 0..NUM_COLORS {
            for square in 0..NUM_SQUARES {
                generated.piece_keys[color][piece_type][square] = random_gen.next();
            }
        }
    }

    for file in 0..NUM_FILES {
        generated.en_passant_keys[file] = random_gen.next();
    }

    for castle in 0..NUM_CASTLING_CONFIGS {
        generated.castle_keys[castle] = random_gen.next();
    }

    generated.side_key = random_gen.next();

    generated
}

pub struct RandomNumberGenerator {
    state: u64,
}

impl RandomNumberGenerator {
    pub fn new(seed: u64) -> Self {
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

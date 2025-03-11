use types::{
    castling_rights::{NUM_CASTLING_RIGHTS_CONFIGURATIONS, PerCastlingRightsConfig},
    color::{NUM_COLORS, PerColor},
    piece::{NUM_PIECES, PerPieceType},
    square::{NUM_FILES, NUM_SQUARES, PerFile, PerSquare},
};

#[repr(C)]
pub struct GeneratedKeys {
    pub piece_keys: PerColor<PerPieceType<PerSquare<u64>>>,
    pub en_passant_keys: PerFile<u64>,
    pub castle_keys: PerCastlingRightsConfig<u64>,
    pub side_key: u64,
}

pub fn generate_keys() -> GeneratedKeys {
    let mut random_gen = RandomNumberGenerator::new(465864546584658);

    let mut piece_keys = [[[0u64; NUM_SQUARES]; NUM_PIECES]; NUM_COLORS];
    for piece_type in piece_keys.iter_mut() {
        for color in piece_type.iter_mut() {
            for square in color.iter_mut() {
                *square = random_gen.next();
            }
        }
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
        piece_keys: PerColor::new(
            piece_keys.map(|per_piece| PerPieceType::new(per_piece.map(PerSquare::new))),
        ),
        en_passant_keys: PerFile::new(en_passant_keys),
        castle_keys: PerCastlingRightsConfig::new(castle_keys),
        side_key,
    }
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

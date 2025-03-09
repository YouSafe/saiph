use crate::types::{
    castling_rights::PerCastlingRightsConfig,
    color::PerColor,
    piece::PerPieceType,
    square::{PerFile, PerSquare},
};

#[repr(C)]
pub struct GeneratedKeys {
    piece_keys: [[[u64; 64]; 6]; 2],
    en_passant_keys: [u64; 8],
    castle_keys: [u64; 16],
    side_key: u64,
}

static KEYS: GeneratedKeys =
    unsafe { std::mem::transmute(*include_bytes!(concat!(env!("OUT_DIR"), "/zobrist"))) };

pub static PIECE_KEYS: PerColor<PerPieceType<PerSquare<u64>>> = cast(KEYS.piece_keys);

pub static CASTLE_KEYS: PerCastlingRightsConfig<u64> =
    PerCastlingRightsConfig::new(KEYS.castle_keys);

pub static EN_PASSANT_KEYS: PerFile<u64> = PerFile::new(KEYS.en_passant_keys);

pub static SIDE_KEY: u64 = KEYS.side_key;

const fn cast(data: [[[u64; 64]; 6]; 2]) -> PerColor<PerPieceType<PerSquare<u64>>> {
    // SAFETY: it is assumed that layout of the data matches that of the output type
    unsafe {
        std::mem::transmute::<[[[u64; 64]; 6]; 2], PerColor<PerPieceType<PerSquare<u64>>>>(data)
    }
}

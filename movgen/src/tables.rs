use chess_core::bitboard::BitBoard;

#[derive(Debug, Default, Clone, Copy)]
pub struct Magic {
    pub magic_number: u64,
    pub shift: u8,
}

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

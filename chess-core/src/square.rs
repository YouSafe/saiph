#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Square {
    A8=56, B8, C8, D8, E8, F8, G8, H8,
    A7=48, B7, C7, D7, E7, F7, G7, H7, 
    A6=40, B6, C6, D6, E6, F6, G6, H6, 
    A5=32, B5, C5, D5, E5, F5, G5, H5,  
    A4=24, B4, C4, D4, E4, F4, G4, H4,  
    A3=16, B3, C3, D3, E3, F3, G3, H3, 
    A2=08, B2, C2, D2, E2, F2, G2, H2,  
    A1=00, B1, C1, D1, E1, F1, G1, H1, 
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rank {
    R1, R2, R3, R4, R5, R6, R7, R8
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum File {
    A, B, C, D, E, F, G, H
}

impl Square {
    pub const fn to_index(&self) -> u8 {
        *self as u8
    }

    pub const fn from_index(index: u8) -> Square {
        assert!(index < 64);
        unsafe { std::mem::transmute::<u8, Square>(index) }
    }

    pub fn from(rank: Rank, file: File) -> Square {
        Self::from_index(rank as u8 * 8 + file as u8)
    }
}

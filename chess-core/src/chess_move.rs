use crate::piece::Piece;
use crate::promotion::Promotion;
use crate::square::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveFlag {
    Normal,
    DoublePawnPush,
    Checking,
    Capture,
    EnPassant,
    Castling,
}

// TODO: pack data into a single u32 or even better: u16
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
    pub piece: Piece,
    pub flags: MoveFlag,
}

#[cfg(test)]
mod test {
    use crate::chess_move::Move;

    #[test]
    fn test() {
        eprintln!("move size in bytes: {}", std::mem::size_of::<Move>());
    }
}

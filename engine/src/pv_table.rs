use std::{
    fmt::{self, Formatter},
};

use chess_core::chess_move::Move;

const MAX_PLY: usize = 64;
const TABLE_SIZE: usize = (MAX_PLY * (MAX_PLY + 1)) / 2;

type Ply = usize;

/// Triangular PV Table using a one-dimensional array for the backing data structure
pub struct PrincipleVariationTable {
    inner: [Option<Move>; TABLE_SIZE],
    lengths: [usize; MAX_PLY],
}

impl PrincipleVariationTable {
    pub fn new() -> Self {
        Self {
            inner: [None; TABLE_SIZE],
            lengths: [0; MAX_PLY],
        }
    }

    pub fn best_move(&self) -> Option<Move> {
        self.inner[0]
    }

    pub fn variation(&self) -> &[Option<Move>] {
        &self.inner[0..self.lengths[0]]
    }

    pub fn clear(&mut self, ply: Ply) {
        assert!(ply < MAX_PLY);
        self.lengths[ply] = 0;
    }

    pub fn update(&mut self, ply: Ply, mv: Move) {
        assert!(ply < MAX_PLY);
        self.inner[index(ply)] = Some(mv);
        self.lengths[ply] = self.lengths[ply + 1] + 1;

        self.copy_variation_from_next_ply(ply);
    }

    fn copy_variation_from_next_ply(&mut self, ply: Ply) {
        let start_index = index(ply + 1);
        let end_index = index(ply + 2).min(start_index + self.lengths[ply + 1]);

        self.inner
            .copy_within(start_index..end_index, index(ply) + 1);
    }
}

fn index(ply: Ply) -> usize {
    ply * MAX_PLY - (ply * ply - ply) / 2
}

impl fmt::Display for PrincipleVariationTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for ply in 0..MAX_PLY {
            for _ in 0..ply {
                write!(f, "     ")?;
            }
            for pv_index in 0..(MAX_PLY - ply) {
                if let Some(chess_move) = self.inner[index(ply) + pv_index] {
                    write!(f, "{} ", chess_move)?;
                } else {
                    write!(f, ".... ")?;
                }
            }

            write!(f, "{}", self.lengths[ply])?;
            writeln!(f)?
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {

    use chess_core::{chess_move::MoveFlag, piece::Piece, square::Square};

    use super::*;

    #[test]
    fn index_test() {
        let mut expected = 0;
        for ply in 0..MAX_PLY {
            assert_eq!(index(ply), expected);
            expected += MAX_PLY - ply;
        }
    }

    #[test]
    fn test_update() {
        let mut pv_table = PrincipleVariationTable::new();
        eprintln!("{}", pv_table);
        pv_table.update(
            4,
            Move {
                from: Square::A4,
                to: Square::A5,
                promotion: None,
                piece: Piece::Pawn,
                flags: MoveFlag::Normal,
            },
        );
        eprintln!("{}", pv_table);
        pv_table.update(
            3,
            Move {
                from: Square::A3,
                to: Square::A5,
                promotion: None,
                piece: Piece::Queen,
                flags: MoveFlag::Normal,
            },
        );
        eprintln!("{}", pv_table);
    }
}

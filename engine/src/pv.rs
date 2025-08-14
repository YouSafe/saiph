use arrayvec::ArrayVec;

use crate::types::chess_move::Move;

pub const MAX_PLY: usize = 64;

#[derive(Debug, Default, Clone)]
pub struct PrincipleVariation {
    inner: ArrayVec<Move, MAX_PLY>,
}

impl PrincipleVariation {
    pub fn from_root(m: Move) -> Self {
        let mut inner = ArrayVec::new();
        inner.push(m);
        Self { inner }
    }

    pub fn best_move(&self) -> Move {
        self.inner[0]
    }

    pub fn line(&self) -> &[Move] {
        &self.inner
    }

    pub fn truncate_to_root(&mut self) {
        self.inner.truncate(1);
    }

    pub fn load_from(&mut self, m: Move, child_pv: &Self) {
        self.inner.clear();
        self.inner.push(m);
        self.inner.try_extend_from_slice(&child_pv.inner).unwrap();
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

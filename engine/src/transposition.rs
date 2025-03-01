use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::board::Board;
use crate::evaluation::Evaluation;
use crate::types::chess_move::Move;

#[derive()]
pub struct TranspositionTable {
    inner: Vec<AtomicU64>,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let table_size = 0x100000 * size_mb;
        let num_entries = table_size / std::mem::size_of::<AtomicU64>();

        let inner = vec![0u64; num_entries]
            .into_iter()
            .map(AtomicU64::new)
            .collect();

        Self { inner }
    }

    pub fn store(
        &self,
        board: &Board,
        best_move: Move,
        depth: u8,
        mut value: Evaluation,
        value_type: ValueType,
        ply: u8,
    ) {
        let index = board.hash() % self.inner.len() as u64;

        // replacement scheme
        let old_entry = self.inner[index as usize].load(Ordering::Relaxed);

        // SAFETY: we statically asserted that an entry is exactly 8 bytes
        let old_entry: Entry = unsafe { std::mem::transmute(old_entry) };
        if depth < old_entry.depth {
            // don't replace when the new entry is less deeply analyzed as the old entry
            return;
        }

        if value.is_mate() {
            value = value.score_to_tt(ply);
        }

        let entry = Entry {
            hash_key: board.hash() as u16,
            best_move,
            depth,
            value,
            value_type,
        };

        let entry: u64 = unsafe { std::mem::transmute(entry) };

        self.inner[index as usize].store(entry, Ordering::Relaxed);
    }

    pub fn probe(&self, board: &Board, ply: u8) -> Option<Entry> {
        let index = board.hash() % self.inner.len() as u64;

        let entry = self.inner[index as usize].load(Ordering::Relaxed);

        // SAFETY: we statically asserted that an entry is exactly 8 bytes
        let mut entry: Entry = unsafe { std::mem::transmute(entry) };

        if entry.hash_key != board.hash() as u16 {
            return None;
        }

        if entry.value.is_mate() {
            entry.value = entry.value.tt_to_score(ply)
        }

        Some(entry)
    }

    pub fn chunk(&self, chunk_index: usize, num_chunks: usize) -> &[AtomicU64] {
        let stride = self.inner.len() / num_chunks;
        let start = stride * chunk_index;
        let end = (start + stride).min(self.inner.len());

        &self.inner[start..end]
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub hash_key: u16,
    pub best_move: Move,
    pub depth: u8,
    pub value: Evaluation,
    pub value_type: ValueType,
}

// entry has to fit into a u64
const _: () = assert!(std::mem::size_of::<Entry>() == 8);

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    /// Alpha
    Upperbound,
    /// Beta
    Lowerbound,
}

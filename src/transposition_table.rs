use chess::{Board, ChessMove};
use std::collections::HashMap;

pub struct TranspositionTable {
    hash_table: HashMap<Board, Entry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            hash_table: HashMap::with_capacity(64000),
        }
    }

    pub fn read_entry(&self, board: &Board) -> Option<&Entry> {
        self.hash_table.get(board)
    }

    pub fn add_entry(
        &mut self,
        board: Board,
        value: i64,
        value_type: ValueType,
        best_move: Option<ChessMove>,
        depth: u8,
    ) {
        let entry = Entry {
            best_move,
            value,
            value_type,
            depth,
        };

        self.hash_table.insert(board, entry);
    }

    pub fn clear(&mut self) {
        self.hash_table.clear();
    }
}

#[repr(u8)]
pub enum ValueType {
    Exact,
    Alpha,
    Beta,
}

pub struct Entry {
    pub best_move: Option<ChessMove>,
    pub value: i64,
    pub value_type: ValueType,
    pub depth: u8,
}

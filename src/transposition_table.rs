use crate::evaluation::Evaluation;
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

    pub fn read_entry(
        &self,
        board: &Board,
        depth: u8,
        alpha: Evaluation,
        beta: Evaluation,
    ) -> Option<&Entry> {
        let entry = self.hash_table.get(board);

        if let Some(entry) = entry {
            if entry.depth >= depth {
                let value = entry.value;
                if entry.value_type == ValueType::Exact {
                    return Some(entry);
                } else if entry.value_type == ValueType::Alpha && value <= alpha {
                    return Some(entry);
                } else if entry.value_type == ValueType::Beta && value <= beta {
                    return Some(entry);
                }
            }
        }
        None
    }

    pub fn add_entry(
        &mut self,
        board: Board,
        value: Evaluation,
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

#[derive(PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    Alpha,
    Beta,
}

pub struct Entry {
    pub best_move: Option<ChessMove>,
    pub value: Evaluation,
    pub value_type: ValueType,
    pub depth: u8,
}

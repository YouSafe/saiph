use crate::evaluation::Evaluation;
use crate::search::ScoringMove;
use crate::transposition_table::ValueType::{Alpha, Beta, Exact};
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
        ply_from_root: u8,
        alpha: Evaluation,
        beta: Evaluation,
    ) -> Option<ScoringMove> {
        let entry = self.hash_table.get(board);

        if let Some(entry) = entry {
            if entry.depth >= depth {
                let corrected_value = if entry.value.is_mate() {
                    entry.value.tt_to_score(ply_from_root)
                } else {
                    entry.value
                };

                if entry.value_type == Exact {
                    return Some(ScoringMove {
                        chess_move: entry.best_move,
                        evaluation: corrected_value,
                    });
                } else if entry.value_type == Alpha && corrected_value <= alpha {
                    return Some(ScoringMove {
                        chess_move: entry.best_move,
                        evaluation: alpha,
                    });
                } else if entry.value_type == Beta && corrected_value >= beta {
                    return Some(ScoringMove {
                        chess_move: entry.best_move,
                        evaluation: beta,
                    });
                };
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
        ply_searched: u8,
    ) {
        let corrected_value = if value.is_mate() {
            value.score_to_tt(ply_searched)
        } else {
            value
        };

        let entry = Entry {
            best_move,
            value: corrected_value,
            value_type,
            depth,
        };

        self.hash_table.insert(board, entry);
    }

    pub fn _clear(&mut self) {
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

#[cfg(test)]
mod test {
    use crate::transposition_table::Entry;

    #[test]
    fn entry_size() {
        eprintln!("entry size in bytes: {}", std::mem::size_of::<Entry>());
    }
}

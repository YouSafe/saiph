use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::movgen::generate_moves;

const TABLE_SIZE: usize = 0x10000 * 2;
const NUM_TABLE_ENTRIES: usize = TABLE_SIZE / std::mem::size_of::<Entry>();

#[derive(Debug)]
pub struct TranspositionTable {
    table: Vec<Option<Entry>>,
    current_age: u16,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: vec![None; NUM_TABLE_ENTRIES],
            current_age: 0,
        }
    }

    pub fn clear(&mut self) {
        self.current_age = 0;
        self.table.fill(None);
    }

    pub fn store(&mut self, board: &Board, best_move: Move, depth: u8) {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        let mut can_replace = false;

        // empty
        if let Some(old_entry) = &self.table[index as usize] {
            if old_entry.age < self.current_age || old_entry.depth < depth {
                can_replace = true;
            }
        } else {
            can_replace = true;
        }

        if !can_replace {
            return;
        }

        self.table[index as usize] = Some(Entry {
            hash_key: board.hash(),
            best_move,
            age: self.current_age,
            depth,
        });
    }

    pub fn probe(&self, board: &Board) -> Option<&Entry> {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        if let Some(Some(entry)) = self.table.get(index as usize) {
            if entry.hash_key == board.hash() {
                return Some(entry);
            }
        }
        None
    }

    pub fn pv_line(&self, board: &Board, depth: u8) -> Vec<Move> {
        let mut line: Vec<Move> = vec![];

        let mut count = 0;
        let mut maybe_entry = self.probe(board);
        let mut current_board = board.clone();
        while let Some(entry) = maybe_entry.take() {
            if count >= depth {
                break;
            }

            let moves = generate_moves(&current_board);
            if moves.contains(&entry.best_move) {
                current_board = current_board.make_move(entry.best_move);
                line.push(entry.best_move);
                maybe_entry = self.probe(&current_board);
            }
            count += 1;
        }
        line
    }

    pub fn age(&mut self) {
        self.current_age += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub hash_key: u64,
    pub best_move: Move,
    pub age: u16,
    pub depth: u8,
    // pub value: Evaluation,
    // pub value_type: ValueType,
}

#[derive(Clone, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    Alpha,
    Beta,
}

// use crate::evaluation::Evaluation;
// use crate::search::ScoringMove;
// use crate::transposition_table::ValueType::{Alpha, Beta, Exact};
// use chess_core::board::Board;
// use chess_core::chess_move::Move;
// use std::collections::HashMap;
//
// pub struct TranspositionTable {
//     hash_table: HashMap<Board, Entry>,
// }
//
// impl TranspositionTable {
//     pub fn new() -> Self {
//         TranspositionTable {
//             hash_table: HashMap::with_capacity(64000),
//         }
//     }
//
//     pub fn read_entry(
//         &self,
//         board: &Board,
//         depth: u8,
//         ply_from_root: u8,
//         alpha: Evaluation,
//         beta: Evaluation,
//     ) -> Option<ScoringMove> {
//         // TODO: implement zobrist hashing for board
//         // let entry = self.hash_table.get(board);
//
//         // if let Some(entry) = entry {
//         //     if entry.depth >= depth {
//         //         let corrected_value = if entry.value.is_mate() {
//         //             entry.value.tt_to_score(ply_from_root)
//         //         } else {
//         //             entry.value
//         //         };
//         //
//         //         if entry.value_type == Exact {
//         //             return Some(ScoringMove {
//         //                 chess_move: entry.best_move,
//         //                 evaluation: corrected_value,
//         //             });
//         //         } else if entry.value_type == Alpha && corrected_value <= alpha {
//         //             return Some(ScoringMove {
//         //                 chess_move: entry.best_move,
//         //                 evaluation: alpha,
//         //             });
//         //         } else if entry.value_type == Beta && corrected_value >= beta {
//         //             return Some(ScoringMove {
//         //                 chess_move: entry.best_move,
//         //                 evaluation: beta,
//         //             });
//         //         };
//         //     }
//         // }
//         None
//     }
//
//     pub fn add_entry(
//         &mut self,
//         board: Board,
//         value: Evaluation,
//         value_type: ValueType,
//         best_move: Option<Move>,
//         depth: u8,
//         ply_searched: u8,
//     ) {
//         let corrected_value = if value.is_mate() {
//             value.score_to_tt(ply_searched)
//         } else {
//             value
//         };
//
//         let entry = Entry {
//             best_move,
//             value: corrected_value,
//             value_type,
//             depth,
//         };
//
//         // TODO: implement zobrist hashing for board
//         // self.hash_table.insert(board, entry);
//     }
//
//     pub fn _clear(&mut self) {
//         self.hash_table.clear();
//     }
// }
//
// #[derive(PartialEq)]
// #[repr(u8)]
// pub enum ValueType {
//     Exact,
//     Alpha,
//     Beta,
// }
//
// pub struct Entry {
//     pub best_move: Option<Move>,
//     pub value: Evaluation,
//     pub value_type: ValueType,
//     pub depth: u8,
// }
//
// #[cfg(test)]
// mod test {
//     use crate::transposition_table::Entry;
//
//     #[test]
//     fn entry_size() {
//         eprintln!("entry size in bytes: {}", );
//     }
// }

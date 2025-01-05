use crate::evaluation::Evaluation;
use crate::board::Board;
use crate::chess_move::Move;

const TABLE_SIZE: usize = 0x100000 * 64;
const NUM_TABLE_ENTRIES: usize = TABLE_SIZE / std::mem::size_of::<Entry>();

#[derive(Debug)]
pub struct TranspositionTable {
    table: Vec<Option<Entry>>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            table: vec![None; NUM_TABLE_ENTRIES],
        }
    }

    pub fn clear(&mut self) {
        self.table.fill(None);
    }

    pub fn store(
        &mut self,
        board: &Board,
        best_move: Option<Move>,
        depth: u8,
        mut value: Evaluation,
        value_type: ValueType,
        ply: u8,
    ) {
        if value.is_mate() {
            value = value.score_to_tt(ply);
        }

        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        self.table[index as usize] = Some(Entry {
            hash_key: board.hash(),
            best_move,
            depth,
            value,
            value_type,
        })
    }

    pub fn probe(&self, board: &Board, ply: u8) -> Option<Entry> {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        if let Some(entry) = &self.table[index as usize] {
            if entry.hash_key == board.hash() {
                let mut value = entry.value;

                if value.is_mate() {
                    value = value.tt_to_score(ply)
                }

                return Some(Entry {
                    value,
                    ..entry.clone()
                });
            }
        }
        None
    }
}







#[derive(Debug, Clone)]
pub struct Entry {
    pub hash_key: u64,
    pub best_move: Option<Move>,
    pub depth: u8,
    pub value: Evaluation,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    /// Alpha
    Upperbound,
    /// Beta
    Lowerbound,
}

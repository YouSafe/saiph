use crate::evaluation::Evaluation;
use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::movgen::generate_moves;

const TABLE_SIZE: usize = 0x100000 * 4096;
const NUM_TABLE_ENTRIES: usize = TABLE_SIZE / std::mem::size_of::<Entry>();

#[derive(Debug)]
pub struct TranspositionTable {
    pub table: Vec<Option<Entry>>,
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

    pub fn store(
        &mut self,
        board: &Board,
        best_move: Option<Move>,
        depth: u8,
        value: Evaluation,
        value_type: ValueType,
        ply: u8,
    ) {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        let mut can_replace = false;

        if let Some(old_entry) = &self.table[index as usize] {
            if old_entry.age != self.current_age || old_entry.depth <= depth {
                can_replace = true;
            }
        } else {
            can_replace = true;
        }

        if !can_replace {
            return;
        }

        let corrected_value = if value.is_mate() {
            value.score_to_tt(ply)
        } else {
            value
        };

        let new_value = Some(Entry {
            hash_key: board.hash(),
            best_move,
            age: self.current_age,
            depth,
            value: corrected_value,
            value_type,
        });

        self.table[index as usize] = new_value;
    }

    pub fn probe(&self, board: &Board) -> Option<&Entry> {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        if let Some(entry) = &self.table[index as usize] {
            if entry.hash_key == board.hash() {
                return Some(entry);
            }
        }
        None
    }

    pub fn probe_pv(&self, board: &Board) -> Option<Move> {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        if let Some(entry) = &self.table[index as usize] {
            if entry.hash_key == board.hash() {
                return entry.best_move;
            }
        }
        None
    }

    pub fn pv_line(&self, board: &mut Board, depth: u8) -> Vec<Move> {
        let mut line: Vec<Move> = vec![];

        let mut count = 0;
        let current_board = board;
        let mut maybe_entry = self.probe_pv(current_board);
        while let Some(best_move) = maybe_entry.take() {
            if count >= depth {
                break;
            }

            // println!("{}", current_board);
            let moves = generate_moves(current_board);
            if moves.contains(&best_move) {
                current_board.apply_move(best_move);
                line.push(best_move);
                maybe_entry = self.probe_pv(current_board);
            } else {
                break;
            }
            count += 1;
        }

        for _ in 0..count {
            current_board.undo_move();
        }

        // println!("{}", current_board);

        line
    }

    pub fn age(&mut self) {
        self.current_age += 1;
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub hash_key: u64,
    pub best_move: Option<Move>,
    pub age: u16,
    pub depth: u8,
    pub value: Evaluation,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum ValueType {
    Exact,
    /// Upperbound
    Alpha,
    /// Lowerbound
    Beta,
}

use crate::evaluation::Evaluation;
use crate::search::ScoringMove;
use chess_core::board::Board;
use chess_core::chess_move::Move;
use chess_core::movgen::generate_moves;

const TABLE_SIZE: usize = 0x100000 * 64;
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

        let corrected_value = if value.is_mate() {
            value.score_to_tt(ply)
        } else {
            value
        };

        self.table[index as usize] = Some(Entry {
            hash_key: board.hash(),
            best_move,
            age: self.current_age,
            depth,
            value: corrected_value,
            value_type,
        });
    }

    pub fn probe(
        &self,
        board: &Board,
        alpha: Evaluation,
        beta: Evaluation,
        depth: u8,
        ply: u8,
    ) -> Option<ScoringMove> {
        let index = board.hash() % NUM_TABLE_ENTRIES as u64;

        if let Some(entry) = &self.table[index as usize] {
            if entry.hash_key == board.hash() && entry.depth >= depth {
                let corrected_value = if entry.value.is_mate() {
                    entry.value.tt_to_score(ply)
                } else {
                    entry.value
                };

                match entry.value_type {
                    ValueType::Exact => {
                        return Some(ScoringMove {
                            evaluation: corrected_value,
                            chess_move: entry.best_move,
                        });
                    }
                    ValueType::Alpha => {
                        if corrected_value <= alpha {
                            return Some(ScoringMove {
                                evaluation: alpha,
                                chess_move: entry.best_move,
                            });
                        }
                    }
                    ValueType::Beta => {
                        if corrected_value >= beta {
                            return Some(ScoringMove {
                                evaluation: beta,
                                chess_move: entry.best_move,
                            });
                        }
                    }
                }
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

    pub fn pv_line(&self, board: &Board, depth: u8) -> Vec<Move> {
        let mut line: Vec<Move> = vec![];

        let mut count = 0;
        let mut maybe_entry = self.probe_pv(board);
        let mut current_board = board.clone();
        while let Some(best_move) = maybe_entry.take() {
            if count >= depth {
                break;
            }

            let moves = generate_moves(&current_board);
            if moves.contains(&best_move) {
                current_board = current_board.make_move(best_move);
                line.push(best_move);
                maybe_entry = self.probe_pv(&current_board);
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
    Alpha,
    Beta,
}

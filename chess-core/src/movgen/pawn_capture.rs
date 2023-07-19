use crate::board::Board;
use crate::movgen::{CheckState, MoveList, PieceMoveGenerator};

pub struct PawnCaptureMoveGenerator;

impl PieceMoveGenerator for PawnCaptureMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {}
}

#[cfg(test)]
mod test {}

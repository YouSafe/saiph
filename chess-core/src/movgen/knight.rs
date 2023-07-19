use crate::board::Board;
use crate::movgen::{CheckState, MoveList, PieceMoveGenerator};

pub struct KnightMoveGenerator;

impl PieceMoveGenerator for KnightMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        todo!()
    }
}

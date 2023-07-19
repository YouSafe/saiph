use crate::board::Board;
use crate::movgen::{CheckState, MoveList, PieceMoveGenerator};

pub struct KingMoveGenerator;

impl PieceMoveGenerator for KingMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        todo!()
    }
}

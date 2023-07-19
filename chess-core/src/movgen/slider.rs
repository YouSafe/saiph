use crate::board::Board;
use crate::movgen::{CheckState, MoveList, PieceMoveGenerator};

pub struct SliderMoveGenerator;

impl PieceMoveGenerator for SliderMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        todo!()
    }
}

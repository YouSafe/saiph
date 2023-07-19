use crate::board::Board;
use crate::movgen::{CheckState, MoveList, PieceMoveGenerator};

pub struct EnPassantMoveGenerator;

impl PieceMoveGenerator for EnPassantMoveGenerator {
    fn generate<T: CheckState + 'static>(board: &Board, move_list: &mut MoveList) {
        todo!()
    }
}

use chess_core::piece::{Piece, NUM_PIECES};

#[rustfmt::skip]
const MVV_LVA: [[u8; NUM_PIECES]; NUM_PIECES] = [
    [15, 14, 13, 12, 11, 10], // victim Pawn
    [25, 24, 23, 22, 21, 20], // victim Knight
    [35, 34, 33, 32, 31, 30], // victim Bishop
    [45, 44, 43, 42, 41, 40], // victim Rook
    [55, 54, 53, 52, 51, 50], // victim Queen
    [ 0,  0,  0,  0,  0,  0], // victim King
];

pub(crate) fn mmv_lva(src_piece: Piece, dst_piece: Piece) -> i32 {
    MVV_LVA[dst_piece as usize][src_piece as usize] as i32
}

#[cfg(test)]
mod test {
    use crate::move_ordering::mmv_lva;
    use chess_core::piece::{Piece, ALL_PIECES};

    #[test]
    fn test() {
        let mut scores: Vec<(Piece, Piece, i32)> =
            Vec::with_capacity(ALL_PIECES.len() * ALL_PIECES.len());

        for src_piece in ALL_PIECES {
            for dst_piece in ALL_PIECES {
                scores.push((src_piece, dst_piece, mmv_lva(src_piece, dst_piece)));
            }
        }

        scores.sort_by_key(|(_, _, score)| *score);

        for (src_piece, dst_piece, score) in scores {
            println!("{:?} takes {:?}: {}", src_piece, dst_piece, score);
        }
    }
}

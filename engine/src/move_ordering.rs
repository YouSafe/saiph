use crate::evaluation::raw_piece_value;
use chess_core::piece::Piece;

/// Most Valuable Victim - Least Valuable Aggressor
pub fn mmv_lva(src_piece: Piece, dst_piece: Piece) -> i32 {
    10 * raw_piece_value(dst_piece) - raw_piece_value(src_piece)
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

use crate::BitBoard;

pub const fn generate_squares_line() -> [[BitBoard; 64]; 64] {
    let mut result = [[BitBoard(0); 64]; 64];

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    let mut from: i8 = 0;
    while from < 64 {
        let mut to: i8 = 0;
        while to < 64 {
            let from_indices @ (from_rank, from_file) = (from / 8, from % 8);
            let to_indices @ (to_rank, to_file) = (to / 8, to % 8);
            let (dir_rank, dir_file) = (
                (to_rank - from_rank).signum(),
                (to_file - from_file).signum(),
            );

            if let (0, 0) = (dir_rank, dir_file) {
                to += 1;
                continue;
            }

            const fn share_diagonal(
                (from_rank, from_file): (i8, i8),
                (to_rank, to_file): (i8, i8),
            ) -> bool {
                (from_file - to_file).abs() == (from_rank - to_rank).abs()
            }

            const fn share_line(
                (from_rank, from_file): (i8, i8),
                (to_rank, to_file): (i8, i8),
            ) -> bool {
                ((from_file == to_file) && (from_rank != to_rank))
                    || ((from_rank == to_rank) && (from_file != to_file))
            }

            if share_diagonal(from_indices, to_indices) || share_line(from_indices, to_indices) {
                let mut marching_index = 0;
                while marching_index < 8 {
                    let (march_rank, march_file) = (
                        from_rank + marching_index * dir_rank,
                        from_file + marching_index * dir_file,
                    );

                    if march_rank < 0 || march_rank > 7 || march_file < 0 || march_file > 7 {
                        break;
                    }

                    result[from as usize][to as usize] = BitBoard(
                        result[from as usize][to as usize].0
                            | to_square_bitboard(march_rank, march_file),
                    );

                    marching_index += 1;
                }
            }

            to += 1;
        }
        from += 1;
    }

    result
}

#[cfg(test)]
mod test {
    use super::generate_squares_line;

    #[test]
    fn test_generate_xray_lines() {
        let ray = generate_squares_line();

        for from in 0..64 {
            for target in 0..64 {
                let line = ray[from][target];
                println!("from: {from} to: {target} {line}");
            }
        }
    }
}

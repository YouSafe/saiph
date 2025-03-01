use crate::BitBoard;

pub const fn generate_squares_between() -> [[BitBoard; 64]; 64] {
    let mut result = [[BitBoard(0); 64]; 64];

    const fn to_square_bitboard(rank: i8, file: i8) -> u64 {
        1 << (rank * 8 + file)
    }

    const fn max(a: i8, b: i8) -> i8 {
        [a, b][(a < b) as usize]
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
                let chebyshev_dist = max((from_file - to_file).abs(), (from_rank - to_rank).abs());

                let mut marching_index = 1;
                while marching_index < chebyshev_dist {
                    let (march_rank, march_file) = (
                        from_rank + marching_index * dir_rank,
                        from_file + marching_index * dir_file,
                    );

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
    use crate::rays_between::generate_squares_between;
    use crate::BitBoard;
    use crate::Square;

    #[test]
    fn test_generate_rays_between_negative_diagonal() {
        let between = generate_squares_between();
        let ray = between[Square::A8 as usize][Square::H1 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_negative_diagonal_backwards() {
        let between = generate_squares_between();
        let ray = between[Square::H1 as usize][Square::A8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_horizontal() {
        let between = generate_squares_between();
        let ray = between[Square::A8 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(9079256848778919936));
    }

    #[test]
    fn test_generate_rays_between_vertical() {
        let between = generate_squares_between();
        let ray = between[Square::H1 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(36170086419038208));
    }

    #[test]
    fn test_generate_rays_between_positive_diagonal() {
        let between = generate_squares_between();
        let ray = between[Square::A1 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(18049651735527936));
    }
    #[test]
    fn test_generate_rays_between_neighbours() {
        let between = generate_squares_between();
        let ray = between[Square::D5 as usize][Square::E5 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(0));
    }
}

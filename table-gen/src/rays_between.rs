use chess_core::bitboard::BitBoard;
use chess_core::square::Square;

pub fn generate_rays_between() -> [[BitBoard; 64]; 64] {
    let mut result = [[BitBoard(0); 64]; 64];

    for from in 0i8..64 {
        for to in 0i8..64 {
            let (from_rank, from_file) = (from / 8, from % 8);
            let (to_rank, to_file) = (to / 8, to % 8);
            let (dir_rank, dir_file) = (
                (to_rank - from_rank).signum(),
                (to_file - from_file).signum(),
            );

            let man_dist = (from_file - to_file).abs() + (from_rank - to_rank).abs();

            if (from_file - to_file).abs() == (from_rank - to_rank).abs() {
                // shares a diagonal
                // one diagonal move is one horizontal and vertical move
                let length = man_dist / 2;

                for marching in 1..length {
                    let (march_rank, march_file) = (
                        from_rank + marching * dir_rank,
                        from_file + marching * dir_file,
                    );

                    result[from as usize][to as usize] |=
                        Square::from_index(march_rank as u8 * 8 + march_file as u8);
                }
            } else if ((from_file == to_file) && (from_rank != to_rank))
                || ((from_rank == to_rank) && (from_file != to_file))
            {
                // shares a line
                let length = man_dist;

                for marching in 1..length {
                    let (march_rank, march_file) = (
                        from_rank + marching * dir_rank,
                        from_file + marching * dir_file,
                    );

                    result[from as usize][to as usize] |=
                        Square::from_index(march_rank as u8 * 8 + march_file as u8);
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::rays_between::generate_rays_between;
    use chess_core::bitboard::BitBoard;
    use chess_core::square::Square;

    #[test]
    fn test_generate_rays_between_negative_diagonal() {
        let between = generate_rays_between();
        let ray = between[Square::A8 as usize][Square::H1 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_negative_diagonal_backwards() {
        let between = generate_rays_between();
        let ray = between[Square::H1 as usize][Square::A8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_horizontal() {
        let between = generate_rays_between();
        let ray = between[Square::A8 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(9079256848778919936));
    }

    #[test]
    fn test_generate_rays_between_vertical() {
        let between = generate_rays_between();
        let ray = between[Square::H1 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(36170086419038208));
    }

    #[test]
    fn test_generate_rays_between_positive_diagonal() {
        let between = generate_rays_between();
        let ray = between[Square::A1 as usize][Square::H8 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(18049651735527936));
    }
    #[test]
    fn test_generate_rays_between_neighbours() {
        let between = generate_rays_between();
        let ray = between[Square::D5 as usize][Square::E5 as usize];

        println!("{ray}");
        assert_eq!(ray, BitBoard(0));
    }
}

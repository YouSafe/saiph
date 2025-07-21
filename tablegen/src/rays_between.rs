use crate::BitBoard;

pub fn generate_squares_between() -> [[BitBoard; 64]; 64] {
    let mut result = [[BitBoard(0); 64]; 64];

    const DIRECTIONS: [i8; 8] = [9, 8, 7, 1, -1, -7, -8, -9];

    for from in 0..64 {
        for dir in DIRECTIONS {
            let mut attacks = 0;
            let mut prev_square = from;

            loop {
                let curr_square = prev_square + dir;
                let abs_file_diff = ((prev_square % 8) - (curr_square % 8)).abs();
                if !(0..64).contains(&curr_square) || abs_file_diff > 2 {
                    break;
                }

                result[from as usize][curr_square as usize] = BitBoard(attacks);
                attacks |= 1 << curr_square;
                prev_square = curr_square;
            }
        }
    }

    result
}

#[cfg(test)]
mod test {
    use crate::Square;

    use super::*;

    fn test_between_table(from: Square, to: Square, expected: BitBoard) {
        let between = generate_squares_between();
        let ray = between[from as usize][to as usize];

        println!("{ray}");
        assert_eq!(ray, expected);
    }

    #[test]
    fn test_generate_rays_between_negative_diagonal() {
        test_between_table(Square::A8, Square::H1, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_negative_diagonal_backwards() {
        test_between_table(Square::H1, Square::A8, BitBoard(567382630219776));
    }

    #[test]
    fn test_generate_rays_between_horizontal() {
        test_between_table(Square::A8, Square::H8, BitBoard(9079256848778919936));
    }

    #[test]
    fn test_generate_rays_between_vertical() {
        test_between_table(Square::H1, Square::H8, BitBoard(36170086419038208));
    }

    #[test]
    fn test_generate_rays_between_positive_diagonal() {
        test_between_table(Square::A1, Square::H8, BitBoard(18049651735527936));
    }

    #[test]
    fn test_generate_rays_between_neighbours() {
        test_between_table(Square::D5, Square::E5, BitBoard(0));
    }
}

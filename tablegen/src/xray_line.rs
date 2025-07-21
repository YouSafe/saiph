use crate::BitBoard;

pub fn generate_squares_line() -> [[BitBoard; 64]; 64] {
    let mut result = [[BitBoard(0); 64]; 64];

    const DIRECTIONS: [i8; 8] = [9, 8, 7, 1, -1, -7, -8, -9];

    for from in 0..64 {
        for dir in DIRECTIONS {
            let mask = BitBoard(mask_one_direction(from, dir));
            let mut squares = mask.0;
            while squares > 0 {
                let sq = squares.trailing_zeros();

                result[from as usize][sq as usize] = mask;

                // also include the from square
                result[from as usize][sq as usize] |= BitBoard(1 << from);

                squares &= squares - 1;
            }
        }
    }
    result
}

pub fn mask_one_direction(square: i8, dir: i8) -> u64 {
    let mut attacks = 0;
    let mut previous_square = square;

    loop {
        let current_square = previous_square + dir;

        let abs_file_diff = ((previous_square % 8) - (current_square % 8)).abs();
        if !(0..64).contains(&current_square) || abs_file_diff > 2 {
            break;
        }

        attacks |= 1 << current_square;
        previous_square = current_square;
    }

    attacks
}

#[cfg(test)]
mod test {
    use crate::{BitBoard, Square, xray_line::generate_squares_line};

    fn test_line_table(from: Square, to: Square, expected: BitBoard) {
        let line = generate_squares_line();
        let ray = line[from as usize][to as usize];

        println!("{ray}");
        assert_eq!(ray, expected);
    }

    #[test]
    fn test_line_same_from_to() {
        let ray = generate_squares_line();
        for from in 0..64 {
            assert_eq!(ray[from][from], BitBoard(0));
        }
    }

    #[test]
    fn test_line_up() {
        test_line_table(Square::E4, Square::E6, BitBoard(1157442765408174080));
        test_line_table(Square::E4, Square::E5, BitBoard(1157442765408174080));
        test_line_table(Square::E4, Square::E8, BitBoard(1157442765408174080));
    }
}

use crate::types::{bitboard::BitBoard, square::Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineType {
    MainDiagonal,
    AntiDiagonal,
    Vertical,
    Horizontal,
}

impl LineType {
    pub fn shared(s1: Square, s2: Square) -> Option<LineType> {
        let s1 = s1 as i8;
        let s2 = s2 as i8;

        let df = (s2 % 8) - (s1 % 8);
        let dr = (s2 / 8) - (s1 / 8);

        if dr == df {
            Some(LineType::MainDiagonal)
        } else if dr == -df {
            Some(LineType::AntiDiagonal)
        } else if dr == 0 {
            Some(LineType::Horizontal)
        } else if df == 0 {
            Some(LineType::Vertical)
        } else {
            None
        }
    }

    pub fn mask(self, anchor: Square) -> BitBoard {
        match self {
            LineType::MainDiagonal => anchor.main_diagonal(),
            LineType::AntiDiagonal => anchor.anti_diagonal(),
            LineType::Vertical => anchor.file().mask(),
            LineType::Horizontal => anchor.rank().mask(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::{line::LineType, square::Square};

    #[test]
    fn test_shared() {
        assert_eq!(
            LineType::shared(Square::A1, Square::F1),
            Some(LineType::Horizontal)
        );
        assert_eq!(
            LineType::shared(Square::G6, Square::E4),
            Some(LineType::MainDiagonal)
        );
        assert_eq!(
            LineType::shared(Square::D4, Square::D6),
            Some(LineType::Vertical)
        );
        assert_eq!(
            LineType::shared(Square::E2, Square::A6),
            Some(LineType::AntiDiagonal)
        );
        assert_eq!(LineType::shared(Square::A1, Square::F2), None);
    }
}

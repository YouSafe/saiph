use crate::types::{bitboard::BitBoard, square::Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    MainDiagonal,
    AntiDiagonal,
    Vertical,
    Horizontal,
}

impl Line {
    pub fn shared(s1: Square, s2: Square) -> Option<Line> {
        let from = s1 as i8;
        let to = s2 as i8;

        let df = (to % 8) - (from % 8);
        let dr = (to / 8) - (from / 8);

        if dr == df {
            Some(Line::MainDiagonal)
        } else if dr == -df {
            Some(Line::AntiDiagonal)
        } else if dr == 0 {
            Some(Line::Horizontal)
        } else if df == 0 {
            Some(Line::Vertical)
        } else {
            None
        }
    }

    pub fn mask(self, anchor: Square) -> BitBoard {
        match self {
            Line::MainDiagonal => anchor.main_diagonal(),
            Line::AntiDiagonal => anchor.anti_diagonal(),
            Line::Vertical => anchor.file().mask(),
            Line::Horizontal => anchor.rank().mask(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::types::{line::Line, square::Square};

    #[test]
    fn test_shared() {
        assert_eq!(Line::shared(Square::A1, Square::F1), Some(Line::Horizontal));
        assert_eq!(
            Line::shared(Square::G6, Square::E4),
            Some(Line::MainDiagonal)
        );
        assert_eq!(Line::shared(Square::D4, Square::D6), Some(Line::Vertical));
        assert_eq!(
            Line::shared(Square::E2, Square::A6),
            Some(Line::AntiDiagonal)
        );
        assert_eq!(Line::shared(Square::A1, Square::F2), None);
    }
}

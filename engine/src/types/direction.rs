use crate::types::{bitboard::BitBoard, color::Color};

#[rustfmt::skip]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N, NE, E, SE, S, SW, W, NW
}

impl Direction {
    pub const fn masked_shift(self, bb: BitBoard) -> BitBoard {
        (bb.intersect(self.mask())).shift(self.shift() as i32)
    }

    pub const fn shift(self) -> i8 {
        match self {
            Direction::N => 8,
            Direction::NE => 9,
            Direction::E => 1,
            Direction::SE => -7,
            Direction::S => -8,
            Direction::SW => -9,
            Direction::W => -1,
            Direction::NW => 7,
        }
    }

    pub const fn repeated_masked_shift<const N: u8>(self, mut bb: BitBoard) -> BitBoard {
        let mut i = N;
        while i > 0 {
            bb = self.masked_shift(bb);
            i -= 1;
        }
        bb
    }

    pub const fn horizontal_flip(self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::NE => Direction::SE,
            Direction::E => Direction::E,
            Direction::SE => Direction::NE,
            Direction::S => Direction::N,
            Direction::SW => Direction::NW,
            Direction::W => Direction::W,
            Direction::NW => Direction::SW,
        }
    }

    pub const fn flip(self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::NE => Direction::SW,
            Direction::E => Direction::W,
            Direction::SE => Direction::NW,
            Direction::S => Direction::N,
            Direction::SW => Direction::NE,
            Direction::W => Direction::E,
            Direction::NW => Direction::SE,
        }
    }

    pub const fn mask(self) -> BitBoard {
        match self {
            Direction::N | Direction::S => BitBoard::FULL,
            Direction::E | Direction::NE | Direction::SE => BitBoard::NOT_H_FILE,
            Direction::W | Direction::NW | Direction::SW => BitBoard::NOT_A_FILE,
        }
    }
}

pub enum RelativeDir {
    Forward,
    Backward,
    Left,
    Right,
    BackwardLeft,
    BackwardRight,
    ForwardLeft,
    ForwardRight,
}

impl RelativeDir {
    pub const fn to_absolute(self, color: Color) -> Direction {
        let dir = match self {
            RelativeDir::Forward => Direction::N,
            RelativeDir::Backward => Direction::S,
            RelativeDir::Left => Direction::W,
            RelativeDir::Right => Direction::E,
            RelativeDir::BackwardLeft => Direction::SW,
            RelativeDir::BackwardRight => Direction::SE,
            RelativeDir::ForwardLeft => Direction::NW,
            RelativeDir::ForwardRight => Direction::NE,
        };

        match color {
            Color::White => dir,
            Color::Black => dir.horizontal_flip(),
        }
    }

    #[inline]
    pub const fn masked_shift(self, color: Color, bb: BitBoard) -> BitBoard {
        self.to_absolute(color).masked_shift(bb)
    }
}

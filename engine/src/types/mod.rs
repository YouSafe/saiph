pub mod bitboard;
pub mod castling_rights;
pub mod chess_move;
pub mod color;
pub mod piece;
pub mod promotion;
pub mod search_limits;
pub mod square;
pub mod uci_move;
pub mod direction;

#[macro_export]
macro_rules! declare_per_type {
    ($name: ident, $index: ty, $num_elements: expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name<T> {
            inner: [T; $num_elements],
        }

        impl<T> std::ops::Index<$index> for $name<T> {
            type Output = T;

            fn index(&self, index: $index) -> &Self::Output {
                &self.inner[usize::from(index)]
            }
        }

        impl<T> std::ops::IndexMut<$index> for $name<T> {
            fn index_mut(&mut self, index: $index) -> &mut Self::Output {
                &mut self.inner[usize::from(index)]
            }
        }

        impl<T: Default> Default for $name<T> {
            fn default() -> Self {
                Self {
                    inner: std::array::from_fn(|_| Default::default()),
                }
            }
        }

        impl<T> $name<T> {
            pub const fn new(inner: [T; $num_elements]) -> Self {
                Self { inner }
            }
        }
    };
}

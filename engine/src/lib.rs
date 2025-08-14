pub mod board;
pub mod clock;
pub mod evaluation;
pub mod movegen;
pub mod moveord;
pub mod search;
pub mod threadpool;
pub mod transposition;
pub mod types;
pub mod uci;
pub mod pv;

mod zobrist;

pub trait Printer {
    fn println(s: &str);
}

pub trait ThreadSpawner {
    fn spawn<F>(f: F)
    where
        F: FnOnce() + Send + 'static;
}

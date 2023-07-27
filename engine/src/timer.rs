use std::ops::Add;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    pub end: Option<Instant>,
}

impl Timer {
    pub fn new() -> Self {
        Timer { end: None }
    }

    pub fn set_timer(&mut self, duration: Duration) {
        self.end = Some(Instant::now().add(duration));
    }

    pub fn is_already_up(&self) -> bool {
        if let Some(end) = self.end {
            end < Instant::now()
        } else {
            false
        }
    }
}

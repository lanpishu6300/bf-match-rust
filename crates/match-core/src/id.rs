use std::sync::atomic::{AtomicU64, Ordering};

/// Generates monotonically increasing numeric identifiers.
pub trait IdGenerator {
    fn next(&self) -> u64;
}

/// Thread-safe u64 counter for tests and local harnesses.
#[derive(Debug, Default)]
pub struct AtomicU64IdGenerator {
    next: AtomicU64,
}

impl AtomicU64IdGenerator {
    pub fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl IdGenerator for AtomicU64IdGenerator {
    fn next(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}

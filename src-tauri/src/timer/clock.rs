use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub trait ClockProvider: Send + Sync {
    fn now_ms(&self) -> i64;
    fn instant_now(&self) -> Instant;
}

pub struct SystemClock;

impl ClockProvider for SystemClock {
    fn now_ms(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_millis() as i64
    }
    fn instant_now(&self) -> Instant { Instant::now() }
}

#[cfg(test)]
pub struct MockClock {
    pub ms: std::sync::atomic::AtomicI64,
    pub start: Instant,
}

#[cfg(test)]
impl MockClock {
    pub fn new(initial_ms: i64) -> Self {
        Self { ms: std::sync::atomic::AtomicI64::new(initial_ms), start: Instant::now() }
    }
    pub fn advance(&self, delta_ms: i64) {
        self.ms.fetch_add(delta_ms, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
impl ClockProvider for MockClock {
    fn now_ms(&self) -> i64 { self.ms.load(std::sync::atomic::Ordering::SeqCst) }
    fn instant_now(&self) -> Instant { self.start }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_clock_returns_positive() {
        let c = SystemClock;
        assert!(c.now_ms() > 1_700_000_000_000);
    }

    #[test]
    fn mock_clock_advances() {
        let m = MockClock::new(1000);
        assert_eq!(m.now_ms(), 1000);
        m.advance(500);
        assert_eq!(m.now_ms(), 1500);
    }
}

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use tokio::time::Instant;

/// A timeout that can be bumped forward in time by calling reset.
///
/// Calling reset with a new duration will change the deadline
/// to the current time plus the new duration. It is non-mutating
/// and can be called from multiple threads.
#[derive(Debug)]
pub struct BumpTimeout(Instant, AtomicU64);

impl BumpTimeout {
    pub fn new(duration: Duration) -> Self {
        let start = Instant::now();
        let millis = duration.as_millis();
        Self(start, AtomicU64::new(millis as u64))
    }

    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.1.load(Ordering::Relaxed))
    }

    pub fn deadline(&self) -> Instant {
        self.0 + self.duration()
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }

    /// Resets the deadline to the current time plus the given duration.
    pub fn reset(&self, duration: Duration) {
        let duration = self.0.elapsed() + duration;
        self.1.store(duration.as_millis() as u64, Ordering::Relaxed);
    }

    pub fn as_instant(&self) -> Instant {
        self.0 + self.duration()
    }

    /// Waits until the deadline is reached, but if the deadline is
    /// changed while waiting, it will wait until the new deadline is reached.
    pub async fn wait(&self) {
        let mut deadline = self.as_instant();
        loop {
            tokio::time::sleep_until(deadline).await;
            let new_deadline = self.as_instant();

            if new_deadline > deadline {
                deadline = new_deadline;
            } else {
                break;
            }
        }
    }
}

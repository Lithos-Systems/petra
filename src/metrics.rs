use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct EngineMetrics {
    total_scans: AtomicU64,
    total_errors: AtomicU64,
    total_scan_time_us: AtomicU64,
}

impl EngineMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_scan(&self, duration: Duration) {
        self.total_scans.fetch_add(1, Ordering::Relaxed);
        self.total_scan_time_us
            .fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }
}

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use tracing::info;

use super::StorageMetricsSnapshot;

#[derive(Debug, Default)]
pub struct StorageMetrics {
    pub write_count: AtomicU64,
    pub query_count: AtomicU64,
    pub bytes_written: AtomicU64,
    pub bytes_read: AtomicU64,
    pub active_connections: AtomicU32,
    pub error_count: AtomicU64,
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn snapshot(&self) -> StorageMetricsSnapshot {
        StorageMetricsSnapshot {
            write_count: self.write_count.load(Ordering::Relaxed),
            query_count: self.query_count.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            bytes_read: self.bytes_read.load(Ordering::Relaxed),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
        }
    }

    pub fn report(&self) {
        let snap = self.snapshot();
        info!(
            "Storage metrics: writes={} queries={} bytes_written={} bytes_read={} connections={} errors={}",
            snap.write_count,
            snap.query_count,
            snap.bytes_written,
            snap.bytes_read,
            snap.active_connections,
            snap.error_count
        );
    }
}

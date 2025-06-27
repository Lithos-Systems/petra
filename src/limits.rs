// Add to src/limits.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_signals: usize,
    pub max_blocks: usize,
    pub max_mqtt_connections: usize,
    pub max_memory_mb: usize,
    pub max_file_size_mb: u64,
    pub scan_timeout_ms: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_signals: 10_000,
            max_blocks: 1_000,
            max_mqtt_connections: 100,
            max_memory_mb: 512,
            max_file_size_mb: 100,
            scan_timeout_ms: 1000,
        }
    }
}

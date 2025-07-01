// src/limits.rs
use serde::{Deserialize, Serialize};
use crate::error::{Result, PlcError};

/// Resource limits for PETRA runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limits {
    #[cfg(feature = "configurable-limits")]
    pub max_signals: Option<usize>,
    
    #[cfg(feature = "configurable-limits")]
    pub max_blocks: Option<usize>,
    
    #[cfg(feature = "configurable-limits")]
    pub max_scan_rate_hz: Option<u32>,
    
    #[cfg(feature = "configurable-limits")]
    pub min_scan_time_ms: Option<u64>,
    
    #[cfg(feature = "memory-limits")]
    pub max_memory_mb: Option<usize>,
    
    #[cfg(feature = "memory-limits")]
    pub max_buffer_size_mb: Option<usize>,
    
    #[cfg(feature = "rate-limits")]
    pub max_updates_per_second: Option<u32>,
    
    #[cfg(feature = "rate-limits")]
    pub max_alarms_per_minute: Option<u32>,
    
    #[cfg(feature = "connection-limits")]
    pub max_mqtt_connections: Option<u32>,
    
    #[cfg(feature = "connection-limits")]
    pub max_plc_connections: Option<u32>,
    
    #[cfg(feature = "storage-limits")]
    pub max_history_size_gb: Option<u64>,
    
    #[cfg(feature = "storage-limits")]
    pub max_log_size_mb: Option<u64>,
}

#[cfg(not(feature = "configurable-limits"))]
impl Default for Limits {
    fn default() -> Self {
        // Hard-coded production limits for minimal builds
        Self {
            #[cfg(feature = "memory-limits")]
            max_memory_mb: Some(512),
            #[cfg(feature = "memory-limits")]
            max_buffer_size_mb: Some(64),
            #[cfg(feature = "rate-limits")]
            max_updates_per_second: Some(10000),
            #[cfg(feature = "rate-limits")]
            max_alarms_per_minute: Some(100),
            #[cfg(feature = "connection-limits")]
            max_mqtt_connections: Some(10),
            #[cfg(feature = "connection-limits")]
            max_plc_connections: Some(32),
            #[cfg(feature = "storage-limits")]
            max_history_size_gb: Some(100),
            #[cfg(feature = "storage-limits")]
            max_log_size_mb: Some(1024),
        }
    }
}

#[cfg(feature = "configurable-limits")]
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_signals: Some(10000),
            max_blocks: Some(1000),
            max_scan_rate_hz: Some(1000),
            min_scan_time_ms: Some(1),
            #[cfg(feature = "memory-limits")]
            max_memory_mb: Some(512),
            #[cfg(feature = "memory-limits")]
            max_buffer_size_mb: Some(64),
            #[cfg(feature = "rate-limits")]
            max_updates_per_second: Some(10000),
            #[cfg(feature = "rate-limits")]
            max_alarms_per_minute: Some(100),
            #[cfg(feature = "connection-limits")]
            max_mqtt_connections: Some(10),
            #[cfg(feature = "connection-limits")]
            max_plc_connections: Some(32),
            #[cfg(feature = "storage-limits")]
            max_history_size_gb: Some(100),
            #[cfg(feature = "storage-limits")]
            max_log_size_mb: Some(1024),
        }
    }
}

#[cfg(feature = "configurable-limits")]
impl Limits {
    pub fn from_config(config: &LimitsConfig) -> Self {
        Self {
            max_signals: config.max_signals,
            max_blocks: config.max_blocks,
            max_scan_rate_hz: config.max_scan_rate_hz,
            min_scan_time_ms: config.min_scan_time_ms,
            #[cfg(feature = "memory-limits")]
            max_memory_mb: config.max_

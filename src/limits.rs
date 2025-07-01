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
            max_memory_mb: config.max_memory_mb,
            #[cfg(feature = "memory-limits")]
            max_buffer_size_mb: config.max_buffer_size_mb,
            #[cfg(feature = "rate-limits")]
            max_updates_per_second: config.max_updates_per_second,
            #[cfg(feature = "rate-limits")]
            max_alarms_per_minute: config.max_alarms_per_minute,
            #[cfg(feature = "connection-limits")]
            max_mqtt_connections: config.max_mqtt_connections,
            #[cfg(feature = "connection-limits")]
            max_plc_connections: config.max_plc_connections,
            #[cfg(feature = "storage-limits")]
            max_history_size_gb: config.max_history_size_gb,
            #[cfg(feature = "storage-limits")]
            max_log_size_mb: config.max_log_size_mb,
        }
   }
   
   pub fn unlimited() -> Self {
       Self {
           max_signals: None,
           max_blocks: None,
           max_scan_rate_hz: None,
           min_scan_time_ms: None,
           #[cfg(feature = "memory-limits")]
           max_memory_mb: None,
           #[cfg(feature = "memory-limits")]
           max_buffer_size_mb: None,
           #[cfg(feature = "rate-limits")]
           max_updates_per_second: None,
           #[cfg(feature = "rate-limits")]
           max_alarms_per_minute: None,
           #[cfg(feature = "connection-limits")]
           max_mqtt_connections: None,
           #[cfg(feature = "connection-limits")]
           max_plc_connections: None,
           #[cfg(feature = "storage-limits")]
           max_history_size_gb: None,
           #[cfg(feature = "storage-limits")]
           max_log_size_mb: None,
       }
   }
}

#[cfg(feature = "configurable-limits")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
   pub max_signals: Option<usize>,
   pub max_blocks: Option<usize>,
   pub max_scan_rate_hz: Option<u32>,
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

/// Enforcer for runtime limits
pub struct LimitEnforcer {
   limits: Limits,
   #[cfg(feature = "rate-limits")]
   rate_trackers: std::sync::Arc<parking_lot::RwLock<RateTrackers>>,
   #[cfg(feature = "memory-limits")]
   memory_monitor: std::sync::Arc<parking_lot::RwLock<MemoryMonitor>>,
}

#[cfg(feature = "rate-limits")]
struct RateTrackers {
   update_tracker: RateTracker,
   alarm_tracker: RateTracker,
}

#[cfg(feature = "rate-limits")]
struct RateTracker {
   window_size: std::time::Duration,
   events: std::collections::VecDeque<std::time::Instant>,
   max_events: u32,
}

#[cfg(feature = "memory-limits")]
struct MemoryMonitor {
   last_check: std::time::Instant,
   check_interval: std::time::Duration,
}

impl LimitEnforcer {
   pub fn new(limits: Limits) -> Self {
       Self {
           limits,
           #[cfg(feature = "rate-limits")]
           rate_trackers: std::sync::Arc::new(parking_lot::RwLock::new(RateTrackers {
               update_tracker: RateTracker::new(
                   std::time::Duration::from_secs(1),
                   limits.max_updates_per_second.unwrap_or(u32::MAX),
               ),
               alarm_tracker: RateTracker::new(
                   std::time::Duration::from_secs(60),
                   limits.max_alarms_per_minute.unwrap_or(u32::MAX),
               ),
           })),
           #[cfg(feature = "memory-limits")]
           memory_monitor: std::sync::Arc::new(parking_lot::RwLock::new(MemoryMonitor {
               last_check: std::time::Instant::now(),
               check_interval: std::time::Duration::from_secs(5),
           })),
       }
   }
   
   pub fn check_signal_count(&self, current: usize) -> Result<()> {
       #[cfg(feature = "configurable-limits")]
       if let Some(max) = self.limits.max_signals {
           if current > max {
               return Err(PlcError::LimitExceeded(format!(
                   "Signal count {} exceeds limit of {}",
                   current, max
               )));
           }
       }
       Ok(())
   }
   
   pub fn check_block_count(&self, current: usize) -> Result<()> {
       #[cfg(feature = "configurable-limits")]
       if let Some(max) = self.limits.max_blocks {
           if current > max {
               return Err(PlcError::LimitExceeded(format!(
                   "Block count {} exceeds limit of {}",
                   current, max
               )));
           }
       }
       Ok(())
   }
   
   pub fn check_scan_time(&self, scan_time_ms: u64) -> Result<()> {
       #[cfg(feature = "configurable-limits")]
       {
           if let Some(min) = self.limits.min_scan_time_ms {
               if scan_time_ms < min {
                   return Err(PlcError::LimitExceeded(format!(
                       "Scan time {}ms is less than minimum {}ms",
                       scan_time_ms, min
                   )));
               }
           }
           
           if let Some(max_rate) = self.limits.max_scan_rate_hz {
               let max_time_ms = 1000 / max_rate as u64;
               if scan_time_ms < max_time_ms {
                   return Err(PlcError::LimitExceeded(format!(
                       "Scan time {}ms exceeds maximum rate of {} Hz",
                       scan_time_ms, max_rate
                   )));
               }
           }
       }
       Ok(())
   }
   
   #[cfg(feature = "rate-limits")]
   pub fn check_update_rate(&self) -> Result<()> {
       let mut trackers = self.rate_trackers.write();
       trackers.update_tracker.check()
   }
   
   #[cfg(feature = "rate-limits")]
   pub fn check_alarm_rate(&self) -> Result<()> {
       let mut trackers = self.rate_trackers.write();
       trackers.alarm_tracker.check()
   }
   
   #[cfg(feature = "memory-limits")]
   pub fn check_memory_usage(&self) -> Result<()> {
       let mut monitor = self.memory_monitor.write();
       
       if monitor.last_check.elapsed() < monitor.check_interval {
           return Ok(());
       }
       
       monitor.last_check = std::time::Instant::now();
       
       if let Some(max_mb) = self.limits.max_memory_mb {
           let current_mb = get_memory_usage_mb();
           if current_mb > max_mb {
               return Err(PlcError::LimitExceeded(format!(
                   "Memory usage {}MB exceeds limit of {}MB",
                   current_mb, max_mb
               )));
           }
       }
       
       Ok(())
   }
   
   #[cfg(feature = "connection-limits")]
   pub fn check_connection_count(&self, conn_type: ConnectionType, current: u32) -> Result<()> {
       let max = match conn_type {
           ConnectionType::Mqtt => self.limits.max_mqtt_connections,
           ConnectionType::Plc => self.limits.max_plc_connections,
       };
       
       if let Some(max_val) = max {
           if current > max_val {
               return Err(PlcError::LimitExceeded(format!(
                   "{:?} connection count {} exceeds limit of {}",
                   conn_type, current, max_val
               )));
           }
       }
       
       Ok(())
   }
   
   #[cfg(feature = "storage-limits")]
   pub fn check_storage_usage(&self, usage_gb: u64) -> Result<()> {
       if let Some(max_gb) = self.limits.max_history_size_gb {
           if usage_gb > max_gb {
               return Err(PlcError::LimitExceeded(format!(
                   "Storage usage {}GB exceeds limit of {}GB",
                   usage_gb, max_gb
               )));
           }
       }
       Ok(())
   }
}

#[cfg(feature = "connection-limits")]
#[derive(Debug, Clone, Copy)]
pub enum ConnectionType {
   Mqtt,
   Plc,
}

#[cfg(feature = "rate-limits")]
impl RateTracker {
   fn new(window_size: std::time::Duration, max_events: u32) -> Self {
       Self {
           window_size,
           events: std::collections::VecDeque::new(),
           max_events,
       }
   }
   
   fn check(&mut self) -> Result<()> {
       let now = std::time::Instant::now();
       
       // Remove old events outside the window
       while let Some(&front) = self.events.front() {
           if now.duration_since(front) > self.window_size {
               self.events.pop_front();
           } else {
               break;
           }
       }
       
       // Check if we can add a new event
       if self.events.len() >= self.max_events as usize {
           Err(PlcError::LimitExceeded(format!(
               "Rate limit exceeded: {} events per {:?}",
               self.max_events, self.window_size
           )))
       } else {
           self.events.push_back(now);
           Ok(())
       }
   }
}

#[cfg(feature = "memory-limits")]
fn get_memory_usage_mb() -> usize {
   // Platform-specific memory usage detection
   #[cfg(target_os = "linux")]
   {
       if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
           for line in status.lines() {
               if line.starts_with("VmRSS:") {
                   if let Some(kb_str) = line.split_whitespace().nth(1) {
                       if let Ok(kb) = kb_str.parse::<usize>() {
                           return kb / 1024;
                       }
                   }
               }
           }
       }
   }
   
   // Fallback or other platforms
   0
}

// Statistics tracking
#[cfg(feature = "limit-statistics")]
pub struct LimitStatistics {
   pub signal_count_max: usize,
   pub block_count_max: usize,
   pub memory_usage_max_mb: usize,
   pub update_rate_max: u32,
   pub alarm_rate_max: u32,
   pub limit_violations: u64,
}

#[cfg(feature = "limit-statistics")]
impl LimitStatistics {
   pub fn new() -> Self {
       Self {
           signal_count_max: 0,
           block_count_max: 0,
           memory_usage_max_mb: 0,
           update_rate_max: 0,
           alarm_rate_max: 0,
           limit_violations: 0,
       }
   }
   
   pub fn update(&mut self, current: &CurrentUsage) {
       self.signal_count_max = self.signal_count_max.max(current.signal_count);
       self.block_count_max = self.block_count_max.max(current.block_count);
       self.memory_usage_max_mb = self.memory_usage_max_mb.max(current.memory_mb);
       self.update_rate_max = self.update_rate_max.max(current.update_rate);
       self.alarm_rate_max = self.alarm_rate_max.max(current.alarm_rate);
   }
}

#[cfg(feature = "limit-statistics")]
pub struct CurrentUsage {
   pub signal_count: usize,
   pub block_count: usize,
   pub memory_mb: usize,
   pub update_rate: u32,
   pub alarm_rate: u32,
}

#[cfg(test)]
mod tests {
   use super::*;
   
   #[test]
   fn test_default_limits() {
       let limits = Limits::default();
       
       #[cfg(feature = "configurable-limits")]
       {
           assert!(limits.max_signals.is_some());
           assert!(limits.max_blocks.is_some());
       }
   }
   
   #[cfg(feature = "configurable-limits")]
   #[test]
   fn test_unlimited_limits() {
       let limits = Limits::unlimited();
       assert!(limits.max_signals.is_none());
       assert!(limits.max_blocks.is_none());
   }
   
   #[test]
   fn test_limit_enforcer() {
       let limits = Limits::default();
       let enforcer = LimitEnforcer::new(limits);
       
       // Test signal count check
       assert!(enforcer.check_signal_count(100).is_ok());
       
       #[cfg(feature = "configurable-limits")]
       {
           assert!(enforcer.check_signal_count(20000).is_err());
       }
   }
   
   #[cfg(feature = "rate-limits")]
   #[test]
   fn test_rate_tracker() {
       let mut tracker = RateTracker::new(
           std::time::Duration::from_secs(1),
           5,
       );
       
       // Should allow 5 events
       for _ in 0..5 {
           assert!(tracker.check().is_ok());
       }
       
       // 6th event should fail
       assert!(tracker.check().is_err());
   }
}

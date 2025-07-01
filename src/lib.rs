// src/lib.rs
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! PETRA - Programmable Engine for Telemetry, Runtime, and Automation
//! 
//! A high-performance, production-ready automation engine built in Rust.

// Core modules (always available)
pub mod error;
pub mod value;
pub mod signal;
pub mod blocks;  // Changed from 'block' to 'blocks'
pub mod config;
pub mod engine;

// Optional modules
#[cfg(feature = "mqtt")]
#[cfg_attr(docsrs, doc(cfg(feature = "mqtt")))]
pub mod mqtt;

#[cfg(feature = "s7-support")]
#[cfg_attr(docsrs, doc(cfg(feature = "s7-support")))]
pub mod s7;

#[cfg(feature = "modbus-support")]
#[cfg_attr(docsrs, doc(cfg(feature = "modbus-support")))]
pub mod modbus;

#[cfg(feature = "opcua-support")]
#[cfg_attr(docsrs, doc(cfg(feature = "opcua-support")))]
pub mod opcua;

#[cfg(feature = "history")]
#[cfg_attr(docsrs, doc(cfg(feature = "history")))]
pub mod history;

#[cfg(feature = "alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "alarms")))]
pub mod alarms;

#[cfg(feature = "advanced-storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "advanced-storage")))]
pub mod storage;

#[cfg(feature = "security")]
#[cfg_attr(docsrs, doc(cfg(feature = "security")))]
pub mod security;

#[cfg(feature = "validation")]
#[cfg_attr(docsrs, doc(cfg(feature = "validation")))]
pub mod validation;

#[cfg(feature = "realtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "realtime")))]
pub mod realtime;

#[cfg(feature = "health")]
#[cfg_attr(docsrs, doc(cfg(feature = "health")))]
pub mod health;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics_server;

// Unified protocol module
#[cfg(any(
    feature = "s7-support",
    feature = "modbus-support", 
    feature = "opcua-support",
    feature = "mqtt"
))]
#[cfg_attr(docsrs, doc(cfg(any(
    feature = "s7-support",
    feature = "modbus-support",
    feature = "opcua-support",
    feature = "mqtt"
))))]
pub mod protocols;

// Re-exports for convenience
pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use engine::Engine;
pub use config::Config;
pub use blocks::{Block, create_block};  // Updated re-export

// Feature-specific re-exports
#[cfg(feature = "enhanced-monitoring")]
pub use engine::DetailedStats;

#[cfg(feature = "circuit-breaker")]
pub use blocks::BlockExecutor;  // Updated re-export

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Initialize the PETRA runtime with optional features
pub fn init() -> Result<()> {
    // Initialize logging
    #[cfg(not(test))]
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "petra=info");
    }
    
    // Initialize metrics registry
    #[cfg(feature = "metrics")]
    {
        use metrics::{describe_counter, describe_gauge, describe_histogram};
        
        describe_counter!("petra_scan_count_total", "Total number of scan cycles");
        describe_gauge!("petra_engine_running", "Engine running state (1=running, 0=stopped)");
        describe_histogram!("petra_scan_duration_us", "Scan cycle duration in microseconds");
        
        #[cfg(feature = "enhanced-monitoring")]
        {
            describe_histogram!("petra_block_execution_time_us", "Block execution time in microseconds");
            describe_counter!("petra_signal_changes_total", "Total signal value changes");
        }
    }
    
    // Initialize realtime capabilities
    #[cfg(all(feature = "realtime", target_os = "linux"))]
    {
        if let Err(e) = crate::realtime::check_realtime_capability() {
           tracing::warn!("Real-time capabilities not available: {}", e);
       }
   }
   
   Ok(())
}

/// Runtime feature detection
pub struct Features {
   pub enhanced_monitoring: bool,
   pub metrics: bool,
   pub security: bool,
   pub protocols: Protocols,
   pub storage: Storage,
}

pub struct Protocols {
   pub s7: bool,
   pub modbus: bool,
   pub opcua: bool,
   pub mqtt: bool,
}

pub struct Storage {
   pub advanced: bool,
   pub compression: bool,
   pub wal: bool,
}

impl Features {
   /// Get enabled features at runtime
   pub fn enabled() -> Self {
       Self {
           enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
           metrics: cfg!(feature = "metrics"),
           security: cfg!(feature = "security"),
           protocols: Protocols {
               s7: cfg!(feature = "s7-support"),
               modbus: cfg!(feature = "modbus-support"),
               opcua: cfg!(feature = "opcua-support"),
               mqtt: cfg!(feature = "mqtt"),
           },
           storage: Storage {
               advanced: cfg!(feature = "advanced-storage"),
               compression: cfg!(feature = "compression"),
               wal: cfg!(feature = "wal"),
           },
       }
   }
   
   /// Print enabled features (useful for diagnostics)
   pub fn print() {
       let features = Self::enabled();
       println!("PETRA Features:");
       println!("  Enhanced Monitoring: {}", features.enhanced_monitoring);
       println!("  Metrics: {}", features.metrics);
       println!("  Security: {}", features.security);
       println!("  Protocols:");
       println!("    S7: {}", features.protocols.s7);
       println!("    Modbus: {}", features.protocols.modbus);
       println!("    OPC-UA: {}", features.protocols.opcua);
       println!("    MQTT: {}", features.protocols.mqtt);
       println!("  Storage:");
       println!("    Advanced: {}", features.storage.advanced);
       println!("    Compression: {}", features.storage.compression);
       println!("    WAL: {}", features.storage.wal);
   }
}

#[cfg(test)]
mod tests {
   use super::*;
   
   #[test]
   fn test_init() {
       init().unwrap();
   }
   
   #[test]
   fn test_version() {
       assert!(!VERSION.is_empty());
   }
}

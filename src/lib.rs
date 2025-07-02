//! PETRA - Programmable Engine for Telemetry, Runtime, and Automation
//! 
//! A high-performance, production-ready automation engine built in Rust with
//! advanced industrial connectivity, alarm management, and enterprise data storage.
//!
//! # Feature Flags
//!
//! PETRA uses an extensive feature flag system to enable modular compilation.
//! See the [`features`] module for complete feature organization and validation.
//!
//! ## Quick Start Bundles
//! 
//! - **Edge Device**: `cargo build --features edge`
//! - **SCADA System**: `cargo build --features scada`  
//! - **Production Server**: `cargo build --features production`
//! - **Enterprise**: `cargo build --features enterprise`
//! - **Development**: `cargo build --features development`
//!
//! # Examples
//!
//! ```rust
//! use petra::{Config, Engine, Features};
//!
//! // Initialize PETRA runtime
//! petra::init()?;
//!
//! // Print enabled features
//! Features::print();
//!
//! // Load configuration and start engine
//! let config = Config::from_file("config.yaml")?;
//! let engine = Engine::new(config)?;
//! engine.run().await?;
//! # Ok::<(), petra::PlcError>(())
//! ```

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ============================================================================
// FEATURE ORGANIZATION MODULE
// ============================================================================

/// Feature flag organization, validation, and runtime detection
/// 
/// This module provides centralized management of PETRA's feature flags,
/// including validation, dependency checking, and runtime feature detection.
pub mod features;

// ============================================================================  
// CORE MODULES (always available)
// ============================================================================

/// Comprehensive error handling with structured error types
pub mod error;

/// Type-safe value system supporting multiple data types
pub mod value;

/// Thread-safe signal bus for inter-block communication
pub mod signal;

/// Extensible block system with built-in logic blocks
pub mod blocks;

/// Configuration management with YAML support and validation
pub mod config;

/// Real-time scan engine with jitter monitoring
pub mod engine;

// ============================================================================
// PROTOCOL MODULES (feature-gated)
// ============================================================================

/// Unified protocol support module
#[cfg(any(
    feature = "mqtt",
    feature = "s7-support", 
    feature = "modbus-support",
    feature = "opcua-support"
))]
#[cfg_attr(docsrs, doc(cfg(any(
    feature = "mqtt",
    feature = "s7-support",
    feature = "modbus-support", 
    feature = "opcua-support"
))))]
pub mod protocols {
    //! Protocol implementations for industrial automation and IoT
    
    #[cfg(feature = "mqtt")]
    #[cfg_attr(docsrs, doc(cfg(feature = "mqtt")))]
    /// MQTT protocol support for IoT and edge devices
    pub mod mqtt;
    
    #[cfg(feature = "s7-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "s7-support")))]
    /// Siemens S7 PLC communication
    pub mod s7;
    
    #[cfg(feature = "modbus-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "modbus-support")))]
    /// Modbus TCP/RTU protocol support
    pub mod modbus;
    
    #[cfg(feature = "opcua-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opcua-support")))]
    /// OPC-UA server implementation
    pub mod opcua;
}

// Individual protocol modules (for backward compatibility)
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

// ============================================================================
// STORAGE MODULES (feature-gated)
// ============================================================================

/// Storage and data persistence
#[cfg(any(feature = "history", feature = "advanced-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "history", feature = "advanced-storage"))))]
pub mod storage {
    //! Data storage and persistence implementations
    
    #[cfg(feature = "history")]
    #[cfg_attr(docsrs, doc(cfg(feature = "history")))]
    /// Parquet-based historical data logging
    pub mod history;
    
    #[cfg(feature = "advanced-storage")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced-storage")))]
    /// Enterprise storage backends (ClickHouse, S3, RocksDB)
    pub mod advanced;
    
    #[cfg(feature = "wal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "wal")))]
    /// Write-Ahead Logging for data durability
    pub mod wal;
}

#[cfg(feature = "history")]
#[cfg_attr(docsrs, doc(cfg(feature = "history")))]
pub mod history;

// ============================================================================
// SECURITY MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "security")]
#[cfg_attr(docsrs, doc(cfg(feature = "security")))]
/// Security, authentication, and authorization
pub mod security {
    //! Security features including authentication, authorization, and audit logging
    
    #[cfg(feature = "basic-auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "basic-auth")))]
    /// Basic authentication implementation
    pub mod basic_auth;
    
    #[cfg(feature = "jwt-auth")]
    #[cfg_attr(docsrs, doc(cfg(feature = "jwt-auth")))]
    /// JWT-based authentication
    pub mod jwt;
    
    #[cfg(feature = "rbac")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rbac")))]
    /// Role-based access control
    pub mod rbac;
    
    #[cfg(feature = "audit")]
    #[cfg_attr(docsrs, doc(cfg(feature = "audit")))]
    /// Security audit logging
    pub mod audit;
    
    #[cfg(feature = "signing")]
    #[cfg_attr(docsrs, doc(cfg(feature = "signing")))]
    /// Cryptographic signing for configurations
    pub mod signing;
}

// ============================================================================
// VALIDATION MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "validation")]
#[cfg_attr(docsrs, doc(cfg(feature = "validation")))]
/// Input validation and data sanitization  
pub mod validation {
    //! Comprehensive validation framework for configuration and runtime data
    
    #[cfg(feature = "regex-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "regex-validation")))]
    /// Regular expression-based validation
    pub mod regex;
    
    #[cfg(feature = "schema-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
    /// JSON schema validation
    pub mod schema;
    
    #[cfg(feature = "composite-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "composite-validation")))]
    /// Complex validation scenarios
    pub mod composite;
    
    #[cfg(feature = "cross-field-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cross-field-validation")))]
    /// Cross-field validation rules
    pub mod cross_field;
}

// ============================================================================
// ALARM & NOTIFICATION MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "alarms")))]
/// Alarm management and notification system
pub mod alarms {
    //! Advanced alarm management with multiple notification channels
    
    #[cfg(feature = "email")]
    #[cfg_attr(docsrs, doc(cfg(feature = "email")))]
    /// Email notification support
    pub mod email;
    
    #[cfg(feature = "twilio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "twilio")))]
    /// SMS and voice notification via Twilio
    pub mod twilio;
}

// ============================================================================
// WEB & HEALTH MODULES (feature-gated)
// ============================================================================

#[cfg(any(feature = "web", feature = "health"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "web", feature = "health"))))]
/// Web interface and health monitoring
pub mod web {
    //! Web interface, REST API, and health monitoring endpoints
    
    #[cfg(feature = "health")]
    #[cfg_attr(docsrs, doc(cfg(feature = "health")))]
    /// System health monitoring and diagnostics
    pub mod health;
    
    #[cfg(feature = "detailed-health")]
    #[cfg_attr(docsrs, doc(cfg(feature = "detailed-health")))]
    /// Detailed health metrics and reporting
    pub mod detailed_health;
    
    #[cfg(feature = "custom-endpoints")]
    #[cfg_attr(docsrs, doc(cfg(feature = "custom-endpoints")))]
    /// Custom web endpoints and API extensions
    pub mod custom_endpoints;
}

#[cfg(feature = "health")]
#[cfg_attr(docsrs, doc(cfg(feature = "health")))]
pub mod health;

// ============================================================================
// METRICS & MONITORING MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
/// Prometheus metrics and telemetry
pub mod metrics_server;

// ============================================================================
// DEVELOPMENT & TESTING MODULES (feature-gated)
// ============================================================================

#[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))))]
/// Development tools and testing utilities
pub mod dev {
    //! Development and testing utilities
    
    #[cfg(feature = "examples")]
    #[cfg_attr(docsrs, doc(cfg(feature = "examples")))]
    /// Example configurations and test scenarios
    pub mod examples;
    
    #[cfg(feature = "burn-in")]
    #[cfg_attr(docsrs, doc(cfg(feature = "burn-in")))]
    /// Burn-in testing and stress testing utilities
    pub mod burn_in;
    
    #[cfg(feature = "profiling")]
    #[cfg_attr(docsrs, doc(cfg(feature = "profiling")))]
    /// Performance profiling and benchmarking
    pub mod profiling;
}

#[cfg(feature = "realtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "realtime")))]
/// Real-time operating system integration
pub mod realtime;

#[cfg(feature = "gui")]
#[cfg_attr(docsrs, doc(cfg(feature = "gui")))]
/// Graphical user interface for configuration and monitoring
pub mod gui;

#[cfg(feature = "hot-swap")]
#[cfg_attr(docsrs, doc(cfg(feature = "hot-swap")))]
/// Hot configuration reloading
pub mod hot_swap;

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core re-exports (always available)
pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use engine::Engine;
pub use config::Config;
pub use blocks::{Block, create_block};
pub use features::RuntimeFeatures as Features;

// Feature-specific re-exports
#[cfg(feature = "enhanced-monitoring")]
pub use engine::DetailedStats;

#[cfg(feature = "circuit-breaker")]
pub use blocks::BlockExecutor;

#[cfg(feature = "metrics")]
pub use metrics_server::MetricsServer;

#[cfg(feature = "security")]
pub use security::SecurityManager;

#[cfg(feature = "validation")]
pub use validation::Validator;

// ============================================================================
// VERSION INFORMATION
// ============================================================================

/// PETRA version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// PETRA authors
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// Build information
pub mod build_info {
    /// Git commit hash (if available)
    pub const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
    
    /// Build timestamp
    pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
    
    /// Rust version used for compilation
    pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");
    
    /// Target triple
    pub const TARGET: &str = env!("TARGET");
    
    /// Build profile (debug/release)
    pub const PROFILE: &str = env!("PROFILE");
}

// ============================================================================
// INITIALIZATION AND RUNTIME
// ============================================================================

/// Initialize the PETRA runtime with feature detection and validation
/// 
/// This function should be called once at the start of your application to:
/// - Initialize logging (if not already configured)
/// - Set up metrics registry (if metrics feature is enabled)
/// - Configure real-time capabilities (if realtime feature is enabled)
/// - Validate feature combinations
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{init, Features};
/// 
/// // Initialize PETRA runtime
/// init()?;
/// 
/// // Print enabled features
/// Features::detect().print();
/// # Ok::<(), petra::PlcError>(())
/// ```
pub fn init() -> Result<()> {
    // Initialize logging if not already configured
    #[cfg(not(test))]
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "petra=info");
    }
    
    // Initialize tracing subscriber
    #[cfg(not(test))]
    {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer().with_target(false));
            
        if let Err(_) = subscriber.try_init() {
            // Already initialized, ignore error
        }
    }
    
    // Initialize metrics registry
    #[cfg(feature = "metrics")]
    init_metrics_registry()?;
    
    // Initialize real-time capabilities
    #[cfg(all(feature = "realtime", target_os = "linux"))]
    init_realtime_capabilities()?;
    
    // Validate runtime feature combinations
    validate_runtime_features()?;
    
    tracing::info!(
        "PETRA {} initialized with features: {}", 
        VERSION,
        Features::detect().summary()
    );
    
    Ok(())
}

#[cfg(feature = "metrics")]
fn init_metrics_registry() -> Result<()> {
    use metrics::{describe_counter, describe_gauge, describe_histogram};
    
    // Core metrics
    describe_counter!("petra_scan_count_total", "Total number of scan cycles completed");
    describe_gauge!("petra_engine_running", "Engine running state (1=running, 0=stopped)");
    describe_histogram!("petra_scan_duration_us", "Scan cycle duration in microseconds");
    describe_counter!("petra_errors_total", "Total number of errors by type");
    
    // Enhanced monitoring metrics
    #[cfg(feature = "enhanced-monitoring")]
    {
        describe_histogram!("petra_block_execution_time_us", "Block execution time in microseconds");
        describe_counter!("petra_signal_changes_total", "Total signal value changes");
        describe_gauge!("petra_memory_usage_bytes", "Memory usage in bytes");
        describe_gauge!("petra_cpu_usage_percent", "CPU usage percentage");
    }
    
    // Protocol metrics
    #[cfg(feature = "mqtt")]
    {
        describe_counter!("petra_mqtt_messages_sent_total", "Total MQTT messages sent");
        describe_counter!("petra_mqtt_messages_received_total", "Total MQTT messages received");
        describe_gauge!("petra_mqtt_connection_state", "MQTT connection state");
    }
    
    #[cfg(any(feature = "s7-support", feature = "modbus-support", feature = "opcua-support"))]
    {
        describe_counter!("petra_plc_reads_total", "Total PLC read operations");
        describe_counter!("petra_plc_writes_total", "Total PLC write operations"); 
        describe_counter!("petra_plc_errors_total", "Total PLC communication errors");
    }
    
    tracing::debug!("Metrics registry initialized");
    Ok(())
}

#[cfg(all(feature = "realtime", target_os = "linux"))]
fn init_realtime_capabilities() -> Result<()> {
    match realtime::check_realtime_capability() {
        Ok(()) => {
            tracing::info!("Real-time capabilities available");
            Ok(())
        }
        Err(e) => {
            tracing::warn!("Real-time capabilities not available: {}", e);
            // Don't fail initialization, just warn
            Ok(())
        }
    }
}

fn validate_runtime_features() -> Result<()> {
    use std::collections::HashSet;
    
    // Collect enabled features for validation
    let features = Features::detect();
    let mut enabled_features = HashSet::new();
    
    // Add enabled features to set for validation
    if features.core.standard_monitoring { enabled_features.insert("standard-monitoring"); }
    if features.core.enhanced_monitoring { enabled_features.insert("enhanced-monitoring"); }
    if features.core.optimized { enabled_features.insert("optimized"); }
    if features.core.metrics { enabled_features.insert("metrics"); }
    if features.protocols.mqtt { enabled_features.insert("mqtt"); }
    if features.protocols.s7 { enabled_features.insert("s7-support"); }
    if features.protocols.modbus { enabled_features.insert("modbus-support"); }
    if features.protocols.opcua { enabled_features.insert("opcua-support"); }
    if features.storage.history { enabled_features.insert("history"); }
    if features.storage.advanced { enabled_features.insert("advanced-storage"); }
    if features.security.security { enabled_features.insert("security"); }
    if features.security.jwt_auth { enabled_features.insert("jwt-auth"); }
    if features.security.rbac { enabled_features.insert("rbac"); }
    
    // Validate using the features module
    if let Err(e) = crate::features::dependencies::validate_features(&enabled_features) {
        return Err(PlcError::Config(format!("Feature validation failed: {}", e)));
    }
    
    Ok(())
}

/// Print comprehensive system information including features, version, and build info
pub fn print_system_info() {
    println!("PETRA System Information");
    println!("========================");
    println!("Version: {}", VERSION);
    println!("Authors: {}", AUTHORS);
    
    if let Some(git_hash) = build_info::GIT_HASH {
        println!("Git Hash: {}", git_hash);
    }
    
    println!("Build Target: {}", build_info::TARGET);
    println!("Build Profile: {}", build_info::PROFILE);
    println!("Rust Version: {}", build_info::RUSTC_VERSION);
    println!();
    
    // Print enabled features
    Features::detect().print();
}

/// Quick feature detection for common use cases
pub mod quick {
    use super::Features;
    
    /// Check if this is an edge device configuration
    pub fn is_edge_device() -> bool {
        let features = Features::detect();
        features.protocols.mqtt && 
        !features.protocols.s7 && 
        !features.protocols.modbus && 
        !features.protocols.opcua &&
        !features.storage.advanced
    }
    
    /// Check if this is a SCADA system configuration
    pub fn is_scada_system() -> bool {
        let features = Features::detect();
        (features.protocols.s7 || features.protocols.modbus || features.protocols.opcua) &&
        features.storage.history
    }
    
    /// Check if this is a production-ready configuration
    pub fn is_production_ready() -> bool {
        let features = Features::detect();
        features.core.optimized && 
        features.security.security &&
        (features.storage.history || features.storage.wal)
    }
    
    /// Check if enterprise features are enabled
    pub fn is_enterprise() -> bool {
        let features = Features::detect();
        features.security.rbac && 
        features.storage.advanced &&
        features.core.enhanced_monitoring
    }
}

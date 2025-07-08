//! # PETRA - Programmable Engine for Telemetry, Runtime, and Automation
//!
//! ## Purpose & Overview
//! 
//! PETRA is a highly modular industrial automation system built in Rust that provides:
//! 
//! - **Real-time PLC functionality** with deterministic scan cycles
//! - **Industrial protocol support** (MQTT, Modbus, S7, OPC-UA)  
//! - **Modular architecture** with 80+ feature flags
//! - **Thread-safe signal bus** for inter-component communication
//! - **Type-safe value system** with optional extended types
//! - **Comprehensive error handling** throughout the system
//!
//! ## Architecture & File Interactions
//!
//! This library acts as the main entry point and orchestrates interactions between:
//!
//! - **Core modules**: `error.rs`, `value.rs`, `signal.rs`, `config.rs`, `engine.rs`
//! - **Block system**: `blocks/` directory with logic blocks and factory
//! - **Protocol drivers**: `protocols/` directory with industrial communication
//! - **Feature modules**: Conditionally compiled based on feature flags
//! - **Storage systems**: History logging and data persistence
//! - **Security framework**: Authentication and authorization
//! - **Monitoring**: Health checks, metrics, and observability
//!
//! ## Key Design Principles
//!
//! 1. **Feature-gated compilation** - Only compile what you need
//! 2. **Thread safety** - All shared state uses lock-free data structures
//! 3. **Performance** - Zero-allocation hot paths where possible
//! 4. **Deterministic behavior** - Predictable real-time performance
//! 5. **Comprehensive error handling** - No panics in production code
//!
//! ## Examples
//!
//! ```rust
//! use petra::{init_petra, Config, Engine, Result};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Initialize PETRA with feature validation
//!     init_petra()?;
//!     
//!     // Load configuration
//!     let config = Config::from_file("config.yaml")?;
//!     
//!     // Create and start engine
//!     let mut engine = Engine::new(config)?;
//!     engine.start().await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![deny(clippy::panic)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

// ============================================================================
// CORE MODULES (Always Available)
// ============================================================================

/// Comprehensive error handling system
/// 
/// Provides the `PlcError` enum that all modules use for error reporting.
/// Includes error context, proper error chains, and integration with `anyhow`.
pub mod error;

/// Type-safe value system with optional extended types
/// 
/// Core `Value` enum supporting Bool, Integer, Float with optional
/// String, Binary, Array, Object types when extended-types feature is enabled.
pub mod value;

/// Memory pool for Value allocations
#[cfg(feature = "memory-pools")]
pub mod memory_pool;

/// Thread-safe signal bus for inter-component communication
/// 
/// Central nervous system using DashMap for lock-free concurrent access.
/// All data exchange between components flows through this signal bus.
pub mod signal;

/// Configuration loading and validation system
/// 
/// YAML-based configuration with comprehensive validation and feature-specific
/// configuration sections that are conditionally compiled.
pub mod config;

/// Real-time execution engine with deterministic scan cycles
/// 
/// Core execution engine that orchestrates block execution, signal updates,
/// and maintains deterministic timing with jitter monitoring.
pub mod engine;

/// Feature detection and validation system
/// 
/// Runtime feature detection, validation of feature dependencies,
/// and feature reporting for debugging and diagnostics.
pub mod features;
pub mod build_info;

// ============================================================================
// BLOCK SYSTEM (High Priority)
// ============================================================================

/// Extensible block system for logic processing
/// 
/// Provides the Block trait, factory functions, and built-in block
/// implementations. All logic processing flows through this system.
pub mod blocks;

// ============================================================================
// PROTOCOL MODULES (Feature-Gated)
// ============================================================================

/// Industrial and IoT protocol implementations
///
/// Protocol drivers for industrial automation and IoT communication.
/// Each protocol is feature-gated for modular compilation.
pub mod protocols;

#[cfg(feature = "mqtt")]
pub mod mqtt {
    pub mod cli;
    pub use cli::test_connection;
}

// ============================================================================
// STORAGE MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "history")]
#[cfg_attr(docsrs, doc(cfg(feature = "history")))]
/// Historical data logging and retrieval
/// 
/// High-performance time-series data storage using Apache Parquet
/// with compression and efficient querying capabilities.
pub mod history;

#[cfg(feature = "advanced-storage")]
#[cfg_attr(docsrs, doc(cfg(feature = "advanced-storage")))]
/// Advanced storage backends and management
/// 
/// Enterprise storage solutions including ClickHouse, RocksDB,
/// and cloud storage integration for scalable data persistence.
pub mod storage {
    //! Advanced storage backends and data management
    //! 
    //! Provides multiple storage options from local databases
    //! to cloud storage with automatic data lifecycle management.

    pub mod cli;
    pub use cli::{initialize_storage, backup_data, restore_data, compact_storage};

    #[cfg(feature = "clickhouse")]
    #[cfg_attr(docsrs, doc(cfg(feature = "clickhouse")))]
    /// ClickHouse backend for analytical workloads
    pub mod clickhouse;

    #[cfg(feature = "rocksdb")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rocksdb")))]
    /// RocksDB backend for high-performance local storage
    pub mod rocksdb;

    #[cfg(feature = "s3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "s3")))]
    /// AWS S3 integration for cloud storage
    pub mod s3;
}

// ============================================================================
// SECURITY MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "security")]
#[cfg_attr(docsrs, doc(cfg(feature = "security")))]
/// Comprehensive security framework
///
/// Authentication, authorization, and audit logging with support
/// for multiple authentication methods and role-based access control.
pub mod security;

// ============================================================================
// VALIDATION MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "validation")]
#[cfg_attr(docsrs, doc(cfg(feature = "validation")))]
/// Comprehensive input validation and data sanitization
/// 
/// Multiple validation strategies from simple range checks
/// to complex schema validation and custom validation rules.
pub mod validation {
    //! Data validation and quality assurance
    //! 
    //! Comprehensive input validation and data sanitization
    //! with multiple validation strategies and detailed error reporting.

    #[cfg(feature = "regex-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "regex-validation")))]
    /// Regular expression-based validation
    pub mod regex;

    #[cfg(feature = "schema-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
    /// JSON Schema validation
    pub mod schema;

    #[cfg(feature = "composite-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "composite-validation")))]
    /// Composite validation with chaining and dependencies
    pub mod composite;
}

// ============================================================================
// MONITORING MODULES (Feature-Gated)
// ============================================================================

#[cfg(any(feature = "metrics", feature = "enhanced-monitoring"))]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
/// Prometheus metrics server for observability
///
/// Production-ready metrics endpoint with comprehensive system
/// and application metrics for monitoring and alerting.
pub mod metrics;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
pub mod metrics_server;

#[cfg(feature = "health")]
#[cfg_attr(docsrs, doc(cfg(feature = "health")))]
/// System health monitoring and diagnostics
/// 
/// Health check endpoints with detailed system information,
/// dependency checking, and integration with monitoring systems.
pub mod health;

// ============================================================================
// ALARM & NOTIFICATION MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "alarms")))]
/// Comprehensive alarm management with configurable notifications
/// 
/// Enterprise-grade alarm system with severity levels, acknowledgment,
/// escalation policies, and integration with external notification systems.
pub mod alarms;

#[cfg(feature = "email")]
#[cfg_attr(docsrs, doc(cfg(feature = "email")))]
/// Email notification system with template support
/// 
/// Rich email notifications with HTML templates, attachments,
/// and integration with popular email services and SMTP servers.
pub mod email;

#[cfg(feature = "twilio")]
#[cfg_attr(docsrs, doc(cfg(feature = "twilio")))]
/// SMS and voice notification via Twilio API
/// 
/// Reliable SMS and voice alerts for critical alarms with delivery
/// confirmation and fallback options for maximum reliability.
pub mod twilio;

// ============================================================================
// WEB & API MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
/// REST API and web interface for configuration and monitoring
/// 
/// Full-featured web interface with REST API, WebSocket support
/// for real-time updates, and responsive UI for mobile devices.
pub mod web;

// ============================================================================
// DEVELOPMENT MODULES (Feature-Gated)
// ============================================================================

#[cfg(feature = "dev-tools")]
#[cfg_attr(docsrs, doc(cfg(feature = "dev-tools")))]
/// Development tools and utilities
/// 
/// Tools for testing, debugging, and development including
/// configuration validators, test data generators, and profiling utilities.
pub mod dev_tools;

#[cfg(feature = "gui")]
#[cfg_attr(docsrs, doc(cfg(feature = "gui")))]
/// Graphical user interface for configuration and monitoring
/// 
/// Desktop GUI application for visual configuration management,
/// real-time monitoring, and system administration.
pub mod gui;

// ============================================================================
// PUBLIC RE-EXPORTS
// ============================================================================

// Core types and functions available to all users
pub use error::{PlcError, Result};
pub use value::{Value, ValueType};
pub use signal::SignalBus;
pub use config::{Config, BlockConfig, SignalConfig, LintSeverity, LintResult};
pub use engine::EngineConfig;
pub use engine::Engine;
pub use features::{Features, RuntimeFeatures};

#[cfg(feature = "metrics")]
pub use metrics_server::MetricsServer;

// Re-export commonly used protocol modules for convenience
#[cfg(feature = "modbus-support")]
pub use protocols::modbus;
#[cfg(feature = "s7-support")]
pub use protocols::s7;
#[cfg(feature = "opcua-support")]
pub use protocols::opcua;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Build information structure
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: &'static str,
    pub git_commit: Option<&'static str>,
    pub build_timestamp: &'static str,
    pub rust_version: &'static str,
    pub profile: &'static str,
    pub features: &'static [&'static str],
}

/// Get build information
/// 
/// Returns detailed information about this build including
/// version, git commit, build timestamp, and enabled features.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: VERSION,
        git_commit: option_env!("PETRA_GIT_COMMIT"),
        build_timestamp: env!("PETRA_BUILD_TIMESTAMP"),
        rust_version: env!("PETRA_RUST_VERSION"),
        profile: if cfg!(debug_assertions) { "debug" } else { "release" },
        features: &[
            #[cfg(feature = "mqtt")]
            "mqtt",
            #[cfg(feature = "s7-support")]
            "s7-support",
            #[cfg(feature = "modbus-support")]
            "modbus-support",
            #[cfg(feature = "opcua-support")]
            "opcua-support",
            #[cfg(feature = "history")]
            "history",
            #[cfg(feature = "security")]
            "security",
            #[cfg(feature = "web")]
            "web",
            #[cfg(feature = "metrics")]
            "metrics",
            #[cfg(feature = "enhanced-monitoring")]
            "enhanced-monitoring",
            #[cfg(feature = "optimized")]
            "optimized",
        ],
    }
}

// ============================================================================
// INITIALIZATION FUNCTIONS
// ============================================================================

/// Initialize PETRA with comprehensive feature validation
/// 
/// This function should be called early in program startup to:
/// 
/// 1. **Validate feature dependencies** - Ensure all required features are enabled
/// 2. **Check feature compatibility** - Detect incompatible feature combinations  
/// 3. **Initialize logging** - Set up structured logging if not already configured
/// 4. **Initialize feature-specific subsystems** - Set up enabled features
/// 5. **Validate runtime environment** - Check platform compatibility
/// 
/// # Errors
/// 
/// Returns `PlcError::Config` if:
/// - Required feature dependencies are missing
/// - Incompatible features are enabled together
/// - Platform-specific features are enabled on unsupported platforms
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{init_petra, Result};
/// 
/// fn main() -> Result<()> {
///     // Initialize PETRA with feature validation
///     init_petra()?;
///     
///     // Continue with application startup...
///     Ok(())
/// }
/// ```
pub fn init_petra() -> Result<()> {
    // Initialize logging if not already configured
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "petra=info");
    }
    
    // Initialize logger if not already done
    let _ = env_logger::try_init();
    
    // Validate feature dependencies using the features module
    features::init().map_err(|errors| {
        let error_msg = format!("Feature validation failed:\n{}", errors.join("\n"));
        PlcError::Config(error_msg)
    })?;
    
    // Initialize feature-specific subsystems
    init_feature_subsystems()?;
    
    log::info!("PETRA {} initialized successfully", VERSION);
    log::debug!("Build info: {:#?}", build_info());
    
    Ok(())
}

/// Compatibility alias for the `init_petra` function
/// 
/// This function provides backward compatibility for tests and code
/// that expects a simple `init` function in the crate root.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::init;
/// 
/// fn main() -> petra::Result<()> {
///     init()?;
///     Ok(())
/// }
/// ```
pub fn init() -> Result<()> {
    init_petra()
}

/// Initialize feature-specific subsystems
/// 
/// This function initializes subsystems for enabled features that
/// require early initialization before the main engine starts.
fn init_feature_subsystems() -> Result<()> {
    // Initialize security subsystem if enabled
    #[cfg(feature = "security")]
    {
        log::debug!("Initializing security subsystem");
        // Security initialization would go here
    }
    
    // Initialize metrics subsystem if enabled
    #[cfg(feature = "metrics")]
    {
        log::debug!("Initializing metrics subsystem");
        // Metrics initialization would go here
    }
    
    // Initialize real-time subsystem if enabled (Linux only)
    #[cfg(all(feature = "realtime", target_os = "linux"))]
    {
        log::debug!("Initializing real-time subsystem");
        // Real-time initialization would go here
    }
    
    // Warn if realtime feature is enabled on non-Linux platforms
    #[cfg(all(feature = "realtime", not(target_os = "linux")))]
    {
        log::warn!("Real-time features are only supported on Linux");
    }
    
    Ok(())
}

// ============================================================================
// FEATURE DETECTION AND REPORTING
// ============================================================================

/// Print enabled features to stdout
/// 
/// Useful for debugging and verifying which features are enabled
/// in the current build. Includes feature counts by category.
pub fn print_features() {
    let features = features::current();
    features.print();
}

/// Get a summary of enabled features
/// 
/// Returns a formatted string with feature counts by category
/// suitable for logging or display.
pub fn feature_summary() -> String {
    let features = features::current();
    features.summary().to_string()
}

// ============================================================================
// CONDITIONAL IMPORTS FOR DEVELOPMENT
// ============================================================================

// Re-export additional symbols when in development mode
#[cfg(any(test, feature = "dev-tools"))]
pub mod test_utils {
    //! Test utilities and mock implementations
    //! 
    //! Only available in test builds or when dev-tools feature is enabled.
    
    pub use crate::signal::SignalBus;
    pub use crate::config::Config;
    
    /// Create a test signal bus for testing
    pub fn create_test_bus() -> SignalBus {
        SignalBus::new()
    }
    
    /// Create a minimal test configuration
    pub fn create_test_config() -> Config {
        Config::default()
    }
}

// ============================================================================
// LIBRARY METADATA
// ============================================================================

/// Library metadata for runtime inspection
pub mod meta {
    use super::BuildInfo;
    
    /// Get library version
    pub fn version() -> &'static str {
        super::VERSION
    }
    
    /// Get library description
    pub fn description() -> &'static str {
        super::DESCRIPTION
    }
    
    /// Get build information
    pub fn build_info() -> BuildInfo {
        super::build_info()
    }
    
    /// Check if a feature is enabled at compile time
    pub fn has_feature(feature: &str) -> bool {
        super::build_info().features.contains(&feature)
    }
}

// ============================================================================
// COMPATIBILITY AND LEGACY SUPPORT
// ============================================================================

// Ensure compatibility with external crates and legacy code
#[doc(hidden)]
pub mod __private {
    //! Private APIs for internal use only
    //! 
    //! These APIs may change without notice and should not be used
    //! by external code.
    
    pub use crate::features::RuntimeFeatures;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_init_petra() {
        // Should initialize without error
        assert!(init_petra().is_ok());
        
        // Should be idempotent (safe to call multiple times)
        assert!(init_petra().is_ok());
    }
    
    #[test]
    fn test_init_compatibility() {
        // Test compatibility alias
        assert!(init().is_ok());
    }
    
    #[test]
    fn test_version_info() {
        let info = build_info();
        assert!(!info.version.is_empty());
        assert!(!info.build_timestamp.is_empty());
        assert!(!info.rust_version.is_empty());
        assert!(info.profile == "debug" || info.profile == "release");
    }
    
    #[test]
    fn test_feature_detection() {
        let features = features::current();
        
        // Should detect at least the default features
        assert!(!features.is_empty());
        
        // Feature validation should pass
        assert!(features.validate().is_ok());
    }
    
    #[test]
    fn test_meta_functions() {
        assert!(!meta::version().is_empty());
        assert!(!meta::description().is_empty());
        
        let info = meta::build_info();
        assert_eq!(info.version, VERSION);
    }
    
    #[cfg(feature = "dev-tools")]
    #[test]
    fn test_dev_utils() {
        let _bus = test_utils::create_test_bus();
        let _config = test_utils::create_test_config();
    }
}

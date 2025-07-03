//! # PETRA - Programmable Engine for Telemetry, Runtime, and Automation
//!
//! ## Purpose & Overview
//! 
//! This is the main library entry point for PETRA, a highly modular industrial automation 
//! system built in Rust with over 80 feature flags. This file serves as the central hub that:
//!
//! - **Exports the public API** - All public types, traits, and functions
//! - **Manages feature gates** - Conditional compilation based on enabled features
//! - **Initializes core systems** - Runtime feature detection and validation
//! - **Provides version information** - Build metadata and versioning
//!
//! ## Architecture & Interactions
//!
//! This library integrates with:
//! - **src/main.rs** - Binary entry point that uses this library
//! - **src/engine.rs** - Core execution engine (re-exported as `Engine`)
//! - **src/signal.rs** - Signal bus system (re-exported as `SignalBus`)
//! - **src/config.rs** - Configuration management (re-exported as `Config`)
//! - **src/blocks/** - Block system framework (re-exported via `Block` trait)
//! - **All feature modules** - Conditionally compiled based on feature flags
//!
//! ## Feature System
//!
//! PETRA uses a hierarchical feature dependency system with 80+ features organized into:
//! - **Core Features**: monitoring, metrics, realtime
//! - **Protocol Features**: mqtt, s7-support, modbus-support, opcua-support
//! - **Storage Features**: history, advanced-storage, compression, wal
//! - **Security Features**: security, basic-auth, jwt-auth, rbac, audit
//! - **Type Features**: extended-types, engineering-types, quality-codes
//! - **Validation Features**: validation, regex-validation, schema-validation
//!
//! ## Performance Considerations
//!
//! - Feature gates ensure zero-cost abstractions for unused functionality
//! - Public re-exports are optimized for minimal compilation overhead
//! - Version info uses compile-time environment variables
//! - Runtime feature detection is cached for performance

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// ============================================================================
// CORE MODULES (always available)
// ============================================================================

/// Comprehensive error handling with PlcError enum and Result types
/// 
/// Central error system that all modules must use for consistent error handling.
/// Provides context-rich error messages and proper error propagation chains.
pub mod error;

/// Type-safe value system supporting Bool, Int, Float with optional extended types
/// 
/// Core value system that all data in PETRA flows through. Provides safe
/// type conversions and serialization/deserialization for all supported types.
pub mod value;

/// Thread-safe signal bus using DashMap for lock-free concurrent access
/// 
/// The central nervous system of PETRA. All inter-component communication
/// must go through the signal bus for thread safety and data consistency.
pub mod signal;

/// YAML configuration loading, validation, and feature-specific configs
/// 
/// Handles all configuration management including validation, feature detection,
/// and hot-reload support for configuration changes.
pub mod config;

/// Real-time scan engine with deterministic cycles and jitter monitoring
/// 
/// Core execution engine that runs blocks in deterministic cycles with
/// microsecond precision timing and comprehensive performance monitoring.
pub mod engine;

/// Runtime feature detection and capability management
/// 
/// Provides runtime introspection of enabled features and validates
/// feature dependencies at startup to prevent configuration errors.
pub mod features;

// ============================================================================
// BLOCK SYSTEM (always available)
// ============================================================================

/// Extensible block system with 15+ built-in block types
/// 
/// Core block framework providing the foundation for all automation logic.
/// Includes base logic blocks, timers, math operations, and data manipulation.
pub mod blocks;

// ============================================================================
// PROTOCOL MODULES (feature-gated for optimal builds)
// ============================================================================

/// Protocol implementations for industrial automation and IoT
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
    //! Protocol driver implementations for industrial automation
    //! 
    //! This module provides a unified interface for all protocol drivers
    //! with consistent error handling and connection management.
    
    #[cfg(feature = "mqtt")]
    #[cfg_attr(docsrs, doc(cfg(feature = "mqtt")))]
    /// MQTT protocol support for IoT and edge devices
    /// 
    /// High-performance MQTT client with automatic reconnection,
    /// QoS support, and integration with the signal bus.
    pub mod mqtt;
    
    #[cfg(feature = "s7-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "s7-support")))]
    /// Siemens S7 PLC communication
    /// 
    /// Native S7 protocol implementation for direct communication
    /// with Siemens PLCs without requiring additional drivers.
    pub mod s7;
    
    #[cfg(feature = "modbus-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "modbus-support")))]
    /// Modbus TCP/RTU protocol support
    /// 
    /// Complete Modbus implementation supporting both TCP and RTU
    /// variants with automatic device discovery and mapping.
    pub mod modbus;
    
    #[cfg(feature = "opcua-support")]
    #[cfg_attr(docsrs, doc(cfg(feature = "opcua-support")))]
    /// OPC-UA server implementation
    /// 
    /// Full-featured OPC-UA server for enterprise integration
    /// with security, subscriptions, and method calls.
    pub mod opcua;
}

// Individual protocol modules (for backward compatibility and direct access)
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
// STORAGE MODULES (feature-gated for lean builds)
// ============================================================================

/// Storage and data persistence with enterprise backends
#[cfg(any(feature = "history", feature = "advanced-storage"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "history", feature = "advanced-storage"))))]
pub mod storage {
    //! Data storage and persistence implementations
    //! 
    //! Provides pluggable storage backends from simple Parquet files
    //! to enterprise-grade databases with compression and WAL support.
    
    #[cfg(feature = "advanced-storage")]
    #[cfg_attr(docsrs, doc(cfg(feature = "advanced-storage")))]
    /// Enterprise storage backends (ClickHouse, S3, RocksDB)
    /// 
    /// High-performance storage options for enterprise deployments
    /// with horizontal scaling and high availability support.
    pub mod advanced;
    
    #[cfg(feature = "wal")]
    #[cfg_attr(docsrs, doc(cfg(feature = "wal")))]
    /// Write-Ahead Logging for data durability
    /// 
    /// Ensures data durability even in case of system failures
    /// with configurable flush intervals and recovery mechanisms.
    pub mod wal;
    
    #[cfg(feature = "compression")]
    #[cfg_attr(docsrs, doc(cfg(feature = "compression")))]
    /// Data compression for storage optimization
    /// 
    /// Multiple compression algorithms optimized for time-series data
    /// with configurable compression levels and real-time performance.
    pub mod compression;
}

#[cfg(feature = "history")]
#[cfg_attr(docsrs, doc(cfg(feature = "history")))]
/// Parquet-based historical data logging with time-series optimization
/// 
/// High-performance data historian using Apache Parquet format
/// for optimal compression and query performance on time-series data.
pub mod history;

// ============================================================================
// SECURITY MODULES (feature-gated for minimal attack surface)
// ============================================================================

#[cfg(feature = "security")]
#[cfg_attr(docsrs, doc(cfg(feature = "security")))]
/// Comprehensive security framework with authentication and authorization
/// 
/// Multi-layered security system supporting various authentication methods,
/// role-based access control, and comprehensive audit logging.
pub mod security;

// ============================================================================
// VALIDATION MODULES (feature-gated for performance)
// ============================================================================

#[cfg(feature = "validation")]
#[cfg_attr(docsrs, doc(cfg(feature = "validation")))]
/// Input validation and data sanitization framework
pub mod validation {
    //! Comprehensive input validation and data sanitization
    //! 
    //! Provides multiple validation strategies from simple range checks
    //! to complex schema validation and custom validation rules.
    
    #[cfg(feature = "regex-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "regex-validation")))]
    /// Regular expression-based validation
    /// 
    /// High-performance regex validation with compiled pattern caching
    /// and comprehensive Unicode support for international data.
    pub mod regex;
    
    #[cfg(feature = "schema-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "schema-validation")))]
    /// JSON Schema validation
    /// 
    /// Full JSON Schema Draft 7 implementation for complex data
    /// validation with detailed error reporting and path tracking.
    pub mod schema;
    
    #[cfg(feature = "composite-validation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "composite-validation")))]
    /// Composite validation with chaining and dependencies
    /// 
    /// Advanced validation system supporting conditional validation,
    /// cross-field dependencies, and validation rule composition.
    pub mod composite;
}

// ============================================================================
// ALARM & NOTIFICATION MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "alarms")]
#[cfg_attr(docsrs, doc(cfg(feature = "alarms")))]
/// Comprehensive alarm management with configurable notifications
/// 
/// Enterprise-grade alarm system with severity levels, acknowledgment,
/// escalation policies, and integration with external notification systems.
pub mod alarms;

#[cfg(feature = "twilio")]
#[cfg_attr(docsrs, doc(cfg(feature = "twilio")))]
/// SMS and voice notification via Twilio API
/// 
/// Reliable SMS and voice alerts for critical alarms with delivery
/// confirmation and fallback options for maximum reliability.
pub mod twilio;

#[cfg(feature = "email")]
#[cfg_attr(docsrs, doc(cfg(feature = "email")))]
/// Email notification system with template support
/// 
/// Rich email notifications with HTML templates, attachments,
/// and integration with popular email services and SMTP servers.
pub mod email;

// ============================================================================
// WEB & API MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
/// REST API and web interface for configuration and monitoring
/// 
/// Full-featured web interface with REST API, WebSocket support
/// for real-time updates, and responsive UI for mobile devices.
pub mod web;

#[cfg(feature = "metrics")]
#[cfg_attr(docsrs, doc(cfg(feature = "metrics")))]
/// Prometheus metrics server for observability
/// 
/// Production-ready metrics endpoint with comprehensive system
/// and application metrics for monitoring and alerting.
pub mod metrics_server;

#[cfg(feature = "health")]
#[cfg_attr(docsrs, doc(cfg(feature = "health")))]
/// System health monitoring and diagnostics
/// 
/// Comprehensive health checks including resource monitoring,
/// dependency checks, and automated recovery mechanisms.
pub mod health;

// ============================================================================
// TESTING & DEVELOPMENT MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "test-utils")]
#[cfg_attr(docsrs, doc(cfg(feature = "test-utils")))]
/// Testing utilities and mock implementations
pub mod test_utils {
    //! Development and testing utilities
    //! 
    //! Provides mock implementations, test fixtures, and utilities
    //! for comprehensive testing of PETRA components.
    
    #[cfg(feature = "examples")]
    #[cfg_attr(docsrs, doc(cfg(feature = "examples")))]
    /// Example configurations and test scenarios
    /// 
    /// Pre-built configurations for common automation scenarios
    /// and comprehensive examples for learning and testing.
    pub mod examples;
    
    #[cfg(feature = "burn-in")]
    #[cfg_attr(docsrs, doc(cfg(feature = "burn-in")))]
    /// Burn-in testing and stress testing utilities
    /// 
    /// Long-running stress tests for validating system stability
    /// and performance under continuous high-load conditions.
    pub mod burn_in;
    
    #[cfg(feature = "profiling")]
    #[cfg_attr(docsrs, doc(cfg(feature = "profiling")))]
    /// Performance profiling and benchmarking tools
    /// 
    /// Detailed performance analysis tools for optimizing
    /// critical paths and identifying performance bottlenecks.
    pub mod profiling;
}

// ============================================================================
// ADVANCED FEATURE MODULES (feature-gated)
// ============================================================================

#[cfg(feature = "realtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "realtime")))]
/// Real-time operating system integration (Linux-only)
/// 
/// Real-time capabilities with deterministic timing, priority inheritance,
/// and integration with RT preemption patches for microsecond precision.
pub mod realtime;

#[cfg(feature = "gui")]
#[cfg_attr(docsrs, doc(cfg(feature = "gui")))]
/// Native graphical user interface for configuration and monitoring
/// 
/// Cross-platform GUI application for visual configuration,
/// real-time monitoring, and system administration.
pub mod gui;

#[cfg(feature = "hot-swap")]
#[cfg_attr(docsrs, doc(cfg(feature = "hot-swap")))]
/// Hot configuration reloading without service interruption
/// 
/// Advanced configuration reloading that maintains system state
/// and connections while applying configuration changes.
pub mod hot_swap;

// ============================================================================
// PUBLIC API RE-EXPORTS (performance-optimized)
// ============================================================================

// Core re-exports (always available) - these are the most frequently used types
pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use engine::Engine;
pub use config::Config;
pub use blocks::{Block, create_block};
pub use features::RuntimeFeatures as Features;

// Feature-specific re-exports (conditionally compiled for zero-cost)
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

#[cfg(feature = "history")]
pub use history::DataHistorian;

#[cfg(feature = "alarms")]
pub use alarms::AlarmManager;

#[cfg(feature = "web")]
pub use web::WebServer;

#[cfg(feature = "realtime")]
pub use realtime::RealtimeScheduler;

// ============================================================================
// VERSION & BUILD INFORMATION (compile-time constants)
// ============================================================================

/// PETRA version string from Cargo.toml
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// PETRA authors from Cargo.toml
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

/// PETRA description from Cargo.toml
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// PETRA repository URL from Cargo.toml
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Build information and metadata
pub mod build_info {
    //! Compile-time build information for debugging and support
    
    /// Git commit hash (if available during build)
    pub const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
    
    /// Build timestamp in ISO 8601 format
    pub const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");
    
    /// Rust compiler version used for compilation
    pub const RUSTC_VERSION: &str = env!("RUSTC_VERSION");
    
    /// Target triple (e.g., x86_64-unknown-linux-gnu)
    pub const TARGET: &str = env!("TARGET");
    
    /// Build profile (debug or release)
    pub const PROFILE: &str = env!("PROFILE");
    
    /// Enabled features at compile time
    pub const FEATURES: &str = env!("ENABLED_FEATURES");
    
    /// Returns a formatted build information string
    /// 
    /// # Returns
    /// 
    /// A multi-line string containing all build information for debugging
    /// and support purposes.
    pub fn build_info_string() -> String {
        format!(
            "PETRA v{}\n\
             Build: {} ({})\n\
             Commit: {}\n\
             Target: {}\n\
             Rustc: {}\n\
             Features: {}",
            super::VERSION,
            BUILD_TIMESTAMP,
            PROFILE,
            GIT_HASH.unwrap_or("unknown"),
            TARGET,
            RUSTC_VERSION,
            FEATURES
        )
    }
}

// ============================================================================
// LIBRARY INITIALIZATION & FEATURE VALIDATION
// ============================================================================

/// Initialize PETRA library with feature validation
/// 
/// This function should be called once at program startup to:
/// - Validate feature dependencies
/// - Initialize global state
/// - Set up logging and monitoring
/// - Verify system compatibility
/// 
/// # Errors
/// 
/// Returns an error if:
/// - Feature dependencies are not satisfied
/// - System requirements are not met
/// - Global initialization fails
/// 
/// # Example
/// 
/// ```rust
/// use petra::init_petra;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     init_petra()?;
///     // Your application code here
///     Ok(())
/// }
/// ```
pub fn init_petra() -> Result<(), PlcError> {
    // Validate feature dependencies
    features::validate_feature_dependencies()?;
    
    // Initialize global state
    #[cfg(feature = "metrics")]
    {
        // Initialize metrics registry
        metrics_server::init_global_metrics()?;
    }
    
    #[cfg(feature = "realtime")]
    {
        // Verify real-time capabilities
        realtime::verify_realtime_support()?;
    }
    
    #[cfg(feature = "security")]
    {
        // Initialize cryptographic backends
        security::init_crypto_backends()?;
    }
    
    Ok(())
}

/// Get runtime feature information
/// 
/// Returns a `RuntimeFeatures` struct containing information about
/// which features are enabled and their capabilities.
/// 
/// # Returns
/// 
/// A `RuntimeFeatures` instance with current feature state
/// 
/// # Example
/// 
/// ```rust
/// use petra::get_runtime_features;
/// 
/// let features = get_runtime_features();
/// if features.has_realtime() {
///     println!("Real-time features are available");
/// }
/// ```
pub fn get_runtime_features() -> Features {
    Features::detect()
}

// ============================================================================
// LIBRARY-LEVEL TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!build_info::BUILD_TIMESTAMP.is_empty());
        assert!(!build_info::TARGET.is_empty());
    }
    
    #[test]
    fn test_build_info_string() {
        let info = build_info::build_info_string();
        assert!(info.contains(VERSION));
        assert!(info.contains(build_info::TARGET));
    }
    
    #[tokio::test]
    async fn test_library_initialization() {
        // Test that library can be initialized without errors
        assert!(init_petra().is_ok());
    }
    
    #[test]
    fn test_runtime_features() {
        let features = get_runtime_features();
        // Test that we can detect features without panicking
        let _has_mqtt = features.has_mqtt();
        let _has_security = features.has_security();
        let _has_realtime = features.has_realtime();
    }
    
    #[cfg(feature = "test-utils")]
    #[test]
    fn test_feature_conditional_compilation() {
        // This test only runs if test-utils feature is enabled
        assert!(true);
    }
}

//! # PETRA Comprehensive Error Handling System
//!
//! ## Purpose & Overview
//! 
//! This module provides the centralized error handling system for PETRA that:
//!
//! - **Standardizes Error Types** - All errors in PETRA flow through `PlcError`
//! - **Provides Rich Context** - Descriptive error messages with actionable information
//! - **Supports Feature Gates** - Error variants are conditionally compiled
//! - **Enables Error Recovery** - Recoverable vs non-recoverable error classification
//! - **Facilitates Debugging** - Enhanced error information in debug builds
//! - **Maintains Security** - No internal implementation details leaked in errors
//!
//! ## Architecture & Interactions
//!
//! This error module is used by ALL other modules in PETRA:
//! - **src/engine.rs** - Engine execution errors and timeouts
//! - **src/signal.rs** - Signal bus operations and type mismatches  
//! - **src/config.rs** - Configuration loading and validation errors
//! - **src/blocks/*** - Block execution and parameter errors
//! - **src/protocols/*** - Protocol communication errors
//! - **All feature modules** - Module-specific error variants
//!
//! ## Error Categorization
//!
//! Errors are categorized by severity and recoverability:
//! - **Fatal**: System cannot continue (Config, Security)
//! - **Recoverable**: Temporary failures that can be retried (Network, Protocol)
//! - **Validation**: Input data problems that can be corrected (Validation)
//! - **Runtime**: Operational issues during execution (Runtime, Signal)
//!
//! ## Performance Considerations
//!
//! - Zero-cost error variants through feature gates
//! - Efficient error propagation with `?` operator
//! - Minimal allocations for common error paths
//! - Optional enhanced error information for debugging
//! - Structured error codes for machine parsing

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use thiserror::Error;

// ============================================================================
// CORE ERROR TYPE DEFINITION
// ============================================================================

/// Central error type for all PETRA operations
/// 
/// All fallible operations in PETRA must return `Result<T, PlcError>` to ensure
/// consistent error handling throughout the system. This enum provides specific
/// error variants for different failure modes while maintaining backwards
/// compatibility through feature gates.
/// 
/// # Error Categories
/// 
/// - **Configuration Errors**: Invalid or missing configuration data
/// - **Runtime Errors**: Failures during normal operation
/// - **Signal Errors**: Signal bus and data flow problems
/// - **Protocol Errors**: Communication failures with external systems
/// - **Security Errors**: Authentication and authorization failures
/// - **Validation Errors**: Input data validation failures
/// 
/// # Example
/// 
/// ```rust
/// use petra::{PlcError, Result};
/// 
/// fn example_operation() -> Result<()> {
///     // This could fail with various error types
///     if some_condition {
///         return Err(PlcError::Config("Invalid parameter".into()));
///     }
///     Ok(())
/// }
/// ```
#[derive(Error, Debug)]
pub enum PlcError {
    // ========================================================================
    // CORE ERROR VARIANTS (always available)
    // ========================================================================
    
    /// Configuration loading, parsing, or validation error
    /// 
    /// This includes YAML parsing errors, missing required fields, invalid
    /// parameter values, and feature compatibility issues.
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Runtime operational error during normal execution
    /// 
    /// This covers failures during engine execution, signal processing,
    /// block execution, and other runtime operations.
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    /// Signal not found in the signal bus
    /// 
    /// Occurs when trying to read or write a signal that doesn't exist
    /// or hasn't been initialized yet.
    #[error("Signal '{0}' not found")]
    SignalNotFound(String),
    
    /// Type mismatch during value conversion or assignment
    /// 
    /// Happens when trying to convert between incompatible value types
    /// or assign wrong type to a signal.
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type received
        actual: String,
    },
    
    /// Generic signal bus operation error
    /// 
    /// Covers signal bus initialization, access control, and other
    /// signal-related operations not covered by specific variants.
    #[error("Signal error: {0}")]
    Signal(String),
    
    /// Block execution or configuration error
    /// 
    /// Includes block initialization failures, execution errors,
    /// invalid parameters, and missing inputs/outputs.
    #[error("Block error: {0}")]
    Block(String),
    
    /// Input validation error
    /// 
    /// Occurs when input data fails validation rules, schema validation,
    /// or constraint checking.
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Generic not found error for resources
    /// 
    /// Used for missing files, resources, or entities not covered
    /// by more specific error types.
    #[error("Not found: {0}")]
    NotFound(String),
    
    // ========================================================================
    // STANDARD LIBRARY ERROR INTEGRATIONS
    // ========================================================================
    
    /// File system I/O error
    /// 
    /// Automatically converts from `std::io::Error` using `#[from]` attribute
    /// for seamless error propagation from file operations.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    
    /// YAML parsing error
    /// 
    /// Automatically converts from `serde_yaml::Error` for configuration
    /// parsing operations.
    #[error("YAML parsing error")]
    Yaml(#[from] serde_yaml::Error),
    
    /// JSON parsing error
    /// 
    /// Automatically converts from `serde_json::Error` for JSON operations
    /// in REST APIs and data exchange.
    #[error("JSON parsing error")]
    Json(#[from] serde_json::Error),
    
    /// Network address parsing error
    /// 
    /// Automatically converts from `std::net::AddrParseError` for network
    /// configuration parsing.
    #[error("Invalid network address")]
    AddrParse(#[from] std::net::AddrParseError),
    
    // ========================================================================
    // PROTOCOL-SPECIFIC ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Protocol communication error
    /// 
    /// Generic protocol error for communication failures not covered
    /// by specific protocol variants.
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// MQTT protocol error
    /// 
    /// Covers MQTT connection failures, subscription errors, publishing
    /// failures, and protocol-level issues.
    #[cfg(feature = "mqtt")]
    #[error("MQTT error: {0}")]
    Mqtt(String),
    
    /// Siemens S7 protocol error
    /// 
    /// S7 connection failures, data type mismatches, PLC communication
    /// errors, and S7-specific protocol issues.
    #[cfg(feature = "s7-support")]
    #[error("S7 communication error: {0}")]
    S7(String),
    
    /// Modbus protocol error
    /// 
    /// Automatically converts from `tokio_modbus::Error` and covers
    /// Modbus TCP/RTU communication failures.
    #[cfg(feature = "modbus-support")]
    #[error("Modbus error")]
    Modbus(#[from] tokio_modbus::Error),
    
    /// OPC-UA protocol error
    /// 
    /// OPC-UA connection failures, subscription errors, security issues,
    /// and OPC-UA specific protocol problems.
    #[cfg(feature = "opcua-support")]
    #[error("OPC-UA error: {0}")]
    OpcUa(String),
    
    // ========================================================================
    // STORAGE & PERSISTENCE ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Data storage and persistence error
    /// 
    /// Covers database connection failures, write errors, storage backend
    /// issues, and data persistence problems.
    #[cfg(feature = "history")]
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Database operation error
    /// 
    /// Specific database errors for advanced storage backends like
    /// ClickHouse, PostgreSQL, and other enterprise databases.
    #[cfg(feature = "advanced-storage")]
    #[error("Database error: {0}")]
    Database(String),
    
    /// Data compression error
    /// 
    /// Compression and decompression failures for storage optimization
    /// and data archival operations.
    #[cfg(feature = "compression")]
    #[error("Compression error: {0}")]
    Compression(String),
    
    // ========================================================================
    // SECURITY ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Generic security error
    /// 
    /// General security-related failures not covered by specific
    /// authentication or authorization errors.
    #[cfg(feature = "security")]
    #[error("Security error: {0}")]
    Security(String),
    
    /// Authentication failure
    /// 
    /// Failed login attempts, invalid credentials, expired tokens,
    /// and other authentication-related failures.
    #[cfg(feature = "security")]
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    /// Authorization denied
    /// 
    /// Access denied due to insufficient permissions, invalid roles,
    /// or resource access restrictions.
    #[cfg(feature = "security")]
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),
    
    /// JWT token error
    /// 
    /// JWT parsing, validation, expiration, and signing errors
    /// for JWT-based authentication.
    #[cfg(feature = "jwt-auth")]
    #[error("JWT error: {0}")]
    Jwt(String),
    
    // ========================================================================
    // WEB & HTTP ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// HTTP client error
    /// 
    /// Automatically converts from `reqwest::Error` for HTTP client
    /// operations and external API communications.
    #[cfg(feature = "web")]
    #[error("HTTP error")]
    Http(#[from] reqwest::Error),
    
    /// Web server error
    /// 
    /// Web server startup failures, route errors, middleware issues,
    /// and HTTP request processing problems.
    #[cfg(feature = "web")]
    #[error("Web server error: {0}")]
    WebServer(String),
    
    /// WebSocket error
    /// 
    /// WebSocket connection failures, message parsing errors, and
    /// real-time communication issues.
    #[cfg(feature = "web")]
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    // ========================================================================
    // MONITORING & METRICS ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Metrics collection error
    /// 
    /// Prometheus metrics registration, collection failures, and
    /// metrics exporter issues.
    #[cfg(feature = "metrics")]
    #[error("Metrics error: {0}")]
    Metrics(String),
    
    /// Health monitoring error
    /// 
    /// Health check failures, monitoring system issues, and
    /// diagnostic collection problems.
    #[cfg(feature = "health")]
    #[error("Health monitoring error: {0}")]
    Health(String),
    
    // ========================================================================
    // NOTIFICATION ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Email notification error
    /// 
    /// SMTP connection failures, email sending errors, and
    /// email template processing issues.
    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(String),
    
    /// SMS/Voice notification error via Twilio
    /// 
    /// Twilio API failures, SMS sending errors, voice call issues,
    /// and notification delivery problems.
    #[cfg(feature = "twilio")]
    #[error("Twilio error: {0}")]
    Twilio(String),
    
    // ========================================================================
    // ADVANCED FEATURE ERROR VARIANTS (feature-gated)
    // ========================================================================
    
    /// Circuit breaker open state
    /// 
    /// Indicates that a circuit breaker is open and preventing
    /// operations due to previous failures.
    #[cfg(feature = "circuit-breaker")]
    #[error("Circuit breaker open - too many recent failures")]
    CircuitOpen,
    
    /// Signal quality error
    /// 
    /// Signal quality validation failures when quality codes are enabled
    /// for enhanced signal reliability monitoring.
    #[cfg(feature = "quality-codes")]
    #[error("Signal quality error: {0}")]
    SignalQuality(String),
    
    /// Real-time constraint violation
    /// 
    /// Real-time deadline misses, scheduling failures, and timing
    /// constraint violations in real-time systems.
    #[cfg(feature = "realtime")]
    #[error("Real-time constraint violation: {0}")]
    RealTimeViolation(String),
    
    /// Hot configuration reload error
    /// 
    /// Configuration hot-reload failures, state migration errors,
    /// and runtime reconfiguration issues.
    #[cfg(feature = "hot-swap")]
    #[error("Hot reload error: {0}")]
    HotReload(String),
    
    // ========================================================================
    // ENHANCED ERROR INFORMATION (feature-gated)
    // ========================================================================
    
    /// Detailed error with enhanced debugging information
    /// 
    /// Provides rich error context including location, source chain,
    /// and additional contextual information for debugging.
    #[cfg(feature = "enhanced-errors")]
    #[error("Error at {location}: {message}")]
    Detailed {
        /// Primary error message
        message: String,
        /// Source location where error occurred
        location: String,
        /// Optional source error chain
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        /// Additional contextual information
        context: HashMap<String, String>,
        /// Optional error code for machine parsing
        error_code: Option<u32>,
        /// Timestamp when error occurred
        #[cfg(feature = "enhanced-monitoring")]
        timestamp: Option<std::time::SystemTime>,
    },
}

// ============================================================================
// RESULT TYPE ALIAS
// ============================================================================

/// Standard Result type for all PETRA operations
/// 
/// This type alias ensures consistent error handling throughout PETRA.
/// All public APIs should use this Result type instead of `std::result::Result`.
/// 
/// # Example
/// 
/// ```rust
/// use petra::Result;
/// 
/// fn my_function() -> Result<String> {
///     Ok("success".to_string())
/// }
/// ```
pub type Result<T> = std::result::Result<T, PlcError>;

// ============================================================================
// ERROR CATEGORIZATION & METADATA
// ============================================================================

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - system can continue normally
    Info,
    /// Warning - potential issue but system can continue
    Warning,
    /// Error - operation failed but system remains stable
    Error,
    /// Critical - system stability may be compromised
    Critical,
    /// Fatal - system cannot continue and must shutdown
    Fatal,
}

/// Error category for grouping and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Configuration and setup errors
    Configuration,
    /// Runtime operational errors
    Runtime,
    /// Network and protocol communication errors
    Communication,
    /// Data storage and persistence errors
    Storage,
    /// Security and authentication errors
    Security,
    /// Input validation and data quality errors
    Validation,
    /// System resource and performance errors
    System,
}

/// Error recovery classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryStrategy {
    /// Error is not recoverable - system must stop or restart
    Fatal,
    /// Error can be retried immediately
    Retry,
    /// Error can be retried after a delay
    RetryWithDelay,
    /// Error requires manual intervention
    Manual,
    /// Error can be ignored and operation continued
    Continue,
}

// ============================================================================
// ERROR TRAIT IMPLEMENTATIONS
// ============================================================================

impl PlcError {
    /// Get the severity level of this error
    /// 
    /// Used for determining logging level and alerting thresholds.
    /// Higher severity errors trigger more urgent notifications.
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Config(_) => ErrorSeverity::Fatal,
            #[cfg(feature = "security")]
            Self::Security(_) => ErrorSeverity::Fatal,
            Self::Io(_) => ErrorSeverity::Critical,
            #[cfg(feature = "history")]
            Self::Storage(_) => ErrorSeverity::Critical,
            Self::Runtime(_) | Self::Protocol(_) => ErrorSeverity::Error,
            Self::Validation(_) | Self::TypeMismatch { .. } => ErrorSeverity::Warning,
            Self::SignalNotFound(_) | Self::NotFound(_) => ErrorSeverity::Info,
            
            #[cfg(feature = "circuit-breaker")]
            Self::CircuitOpen => ErrorSeverity::Warning,
            
            #[cfg(feature = "realtime")]
            Self::RealTimeViolation(_) => ErrorSeverity::Critical,
            
            _ => ErrorSeverity::Error,
        }
    }
    
    /// Get the category of this error
    /// 
    /// Used for error grouping, metrics collection, and specialized
    /// error handling based on error type.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Config(_) => ErrorCategory::Configuration,
            Self::Runtime(_) | Self::Block(_) | Self::Signal(_) => ErrorCategory::Runtime,
            Self::Protocol(_) => ErrorCategory::Communication,
            #[cfg(feature = "mqtt")]
            Self::Mqtt(_) => ErrorCategory::Communication,
            #[cfg(feature = "history")]
            Self::Storage(_) => ErrorCategory::Storage,
            #[cfg(feature = "security")]
            Self::Security(_) => ErrorCategory::Security,
            #[cfg(feature = "security")]
            Self::AuthenticationFailed(_) => ErrorCategory::Security,
            Self::Validation(_) | Self::TypeMismatch { .. } => ErrorCategory::Validation,
            Self::Io(_) => ErrorCategory::System,
            
            #[cfg(feature = "s7-support")]
            Self::S7(_) => ErrorCategory::Communication,
            
            #[cfg(feature = "modbus-support")]
            Self::Modbus(_) => ErrorCategory::Communication,
            
            #[cfg(feature = "opcua-support")]
            Self::OpcUa(_) => ErrorCategory::Communication,
            
            _ => ErrorCategory::Runtime,
        }
    }
    
    /// Get the recommended recovery strategy for this error
    /// 
    /// Determines how the system should handle this error type
    /// for automatic error recovery and resilience.
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        match self {
            Self::Config(_) => RecoveryStrategy::Fatal,
            #[cfg(feature = "security")]
            Self::Security(_) => RecoveryStrategy::Fatal,
            Self::Io(_) => RecoveryStrategy::RetryWithDelay,
            #[cfg(feature = "history")]
            Self::Storage(_) => RecoveryStrategy::RetryWithDelay,
            Self::Protocol(_) => RecoveryStrategy::Retry,
            #[cfg(feature = "mqtt")]
            Self::Mqtt(_) => RecoveryStrategy::Retry,
            Self::Validation(_) | Self::TypeMismatch { .. } => RecoveryStrategy::Manual,
            Self::SignalNotFound(_) => RecoveryStrategy::Continue,
            
            #[cfg(feature = "circuit-breaker")]
            Self::CircuitOpen => RecoveryStrategy::RetryWithDelay,
            
            #[cfg(feature = "realtime")]
            Self::RealTimeViolation(_) => RecoveryStrategy::Manual,
            
            _ => RecoveryStrategy::Retry,
        }
    }
    
    /// Check if this error indicates a permanent failure
    /// 
    /// Used to determine if operations should be retried or abandoned.
    /// Permanent failures should not be retried automatically.
    pub fn is_permanent(&self) -> bool {
        matches!(
            self.recovery_strategy(),
            RecoveryStrategy::Fatal | RecoveryStrategy::Manual
        )
    }
    
    /// Check if this error can be safely retried
    /// 
    /// Used by retry logic to determine if an operation should be
    /// attempted again after a failure.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.recovery_strategy(),
            RecoveryStrategy::Retry | RecoveryStrategy::RetryWithDelay
        )
    }
    
    /// Get error code for machine parsing and monitoring
    /// 
    /// Provides a stable numeric identifier for each error type
    /// that can be used by monitoring systems and automation.
    pub fn error_code(&self) -> u32 {
        match self {
            Self::Config(_) => 1000,
            Self::Runtime(_) => 2000,
            Self::SignalNotFound(_) => 2001,
            Self::TypeMismatch { .. } => 2002,
            Self::Signal(_) => 2003,
            Self::Block(_) => 2004,
            Self::Validation(_) => 3000,
            Self::NotFound(_) => 4000,
            Self::Io(_) => 5000,
            Self::Yaml(_) => 5001,
            Self::Json(_) => 5002,
            Self::AddrParse(_) => 5003,
            Self::Protocol(_) => 6000,
            
            #[cfg(feature = "mqtt")]
            Self::Mqtt(_) => 6001,
            
            #[cfg(feature = "s7-support")]
            Self::S7(_) => 6002,
            
            #[cfg(feature = "modbus-support")]
            Self::Modbus(_) => 6003,
            
            #[cfg(feature = "opcua-support")]
            Self::OpcUa(_) => 6004,
            
            #[cfg(feature = "history")]
            Self::Storage(_) => 7000,
            
            #[cfg(feature = "advanced-storage")]
            Self::Database(_) => 7001,
            
            #[cfg(feature = "compression")]
            Self::Compression(_) => 7002,
            
            #[cfg(feature = "security")]
            Self::Security(_) => 8000,
            
            #[cfg(feature = "security")]
            Self::AuthenticationFailed(_) => 8001,
            
            #[cfg(feature = "security")]
            Self::AuthorizationDenied(_) => 8002,
            
            #[cfg(feature = "jwt-auth")]
            Self::Jwt(_) => 8003,
            
            #[cfg(feature = "web")]
            Self::Http(_) => 9000,
            
            #[cfg(feature = "web")]
            Self::WebServer(_) => 9001,
            
            #[cfg(feature = "web")]
            Self::WebSocket(_) => 9002,
            
            #[cfg(feature = "metrics")]
            Self::Metrics(_) => 10000,
            
            #[cfg(feature = "health")]
            Self::Health(_) => 10001,
            
            #[cfg(feature = "email")]
            Self::Email(_) => 11000,
            
            #[cfg(feature = "twilio")]
            Self::Twilio(_) => 11001,
            
            #[cfg(feature = "circuit-breaker")]
            Self::CircuitOpen => 12000,
            
            #[cfg(feature = "quality-codes")]
            Self::SignalQuality(_) => 12001,
            
            #[cfg(feature = "realtime")]
            Self::RealTimeViolation(_) => 12002,
            
            #[cfg(feature = "hot-swap")]
            Self::HotReload(_) => 12003,
            
            #[cfg(feature = "enhanced-errors")]
            Self::Detailed { error_code: Some(code), .. } => *code,
            
            #[cfg(feature = "enhanced-errors")]
            Self::Detailed { .. } => 13000,
        }
    }
}

// ============================================================================
// ENHANCED ERROR BUILDER (feature-gated)
// ============================================================================

/// Builder for creating detailed errors with rich context
/// 
/// Provides a fluent interface for constructing detailed errors with
/// location information, context data, and error chaining.
#[cfg(feature = "enhanced-errors")]
pub struct DetailedErrorBuilder {
    message: String,
    location: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    context: HashMap<String, String>,
    error_code: Option<u32>,
    #[cfg(feature = "enhanced-monitoring")]
    timestamp: Option<std::time::SystemTime>,
}

#[cfg(feature = "enhanced-errors")]
impl DetailedErrorBuilder {
    /// Create a new detailed error builder with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: String::new(),
            source: None,
            context: HashMap::new(),
            error_code: None,
            #[cfg(feature = "enhanced-monitoring")]
            timestamp: Some(std::time::SystemTime::now()),
        }
    }
    
    /// Add location information (file, function, line)
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }
    
    /// Add source error for error chaining
    pub fn caused_by(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }
    
    /// Add contextual key-value information
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple context entries at once
    pub fn with_context_map(mut self, context: HashMap<String, String>) -> Self {
        self.context.extend(context);
        self
    }
    
    /// Set a specific error code
    pub fn with_error_code(mut self, code: u32) -> Self {
        self.error_code = Some(code);
        self
    }
    
    /// Build the final detailed error
    pub fn build(self) -> PlcError {
        PlcError::Detailed {
            message: self.message,
            location: self.location,
            source: self.source,
            context: self.context,
            error_code: self.error_code,
            #[cfg(feature = "enhanced-monitoring")]
            timestamp: self.timestamp,
        }
    }
}

#[cfg(feature = "enhanced-errors")]
impl PlcError {
    /// Create a detailed error builder
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use petra::PlcError;
    /// 
    /// let error = PlcError::detailed("Operation failed")
    ///     .at("engine.rs:123")
    ///     .with_context("signal", "temperature_sensor")
    ///     .with_context("block", "PID_controller")
    ///     .with_error_code(2001)
    ///     .build();
    /// ```
    pub fn detailed(message: impl Into<String>) -> DetailedErrorBuilder {
        DetailedErrorBuilder::new(message)
    }
}

// ============================================================================
// ERROR RECOVERY UTILITIES (feature-gated)
// ============================================================================

/// Trait for providing error recovery suggestions
/// 
/// Implemented by error types to provide actionable recovery information
/// for operators and automated systems.
#[cfg(feature = "error-recovery")]
pub trait ErrorRecovery {
    /// Get human-readable recovery suggestions
    fn recovery_suggestions(&self) -> Vec<String>;
    
    /// Check if error is recoverable automatically
    fn is_recoverable(&self) -> bool;
    
    /// Get recommended retry delay for retryable errors
    fn retry_delay(&self) -> Option<std::time::Duration>;
}

#[cfg(feature = "error-recovery")]
impl ErrorRecovery for PlcError {
    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::Config(msg) => vec![
                "Check YAML syntax and structure".to_string(),
                "Validate configuration against schema".to_string(),
                "Ensure all required fields are present".to_string(),
                "Check file permissions and accessibility".to_string(),
                format!("Specific error: {}", msg),
            ],
            
            Self::SignalNotFound(name) => vec![
                format!("Add signal '{}' to configuration", name),
                "Check signal name spelling and case".to_string(),
                "Ensure signal is defined before use".to_string(),
                "Verify signal initialization order".to_string(),
            ],
            
            Self::TypeMismatch { expected, actual } => vec![
                format!("Convert {} value to {} type", actual, expected),
                "Check signal type definitions in configuration".to_string(),
                "Use appropriate type conversion methods".to_string(),
                "Verify block input/output type compatibility".to_string(),
            ],
            
            Self::Protocol(msg) => vec![
                "Check network connectivity".to_string(),
                "Verify protocol configuration parameters".to_string(),
                "Ensure target device is accessible".to_string(),
                "Check firewall and security settings".to_string(),
                format!("Protocol details: {}", msg),
            ],
            
            #[cfg(feature = "mqtt")]
            Self::Mqtt(msg) => vec![
                "Verify MQTT broker is running and accessible".to_string(),
                "Check MQTT credentials and permissions".to_string(),
                "Confirm topic names and QoS settings".to_string(),
                "Test network connectivity to broker".to_string(),
                format!("MQTT error: {}", msg),
            ],
            
            #[cfg(feature = "circuit-breaker")]
            Self::CircuitOpen => vec![
                "Wait for circuit breaker cool-down period".to_string(),
                "Check system health and error logs".to_string(),
                "Address underlying issues causing failures".to_string(),
                "Consider manual circuit breaker reset if appropriate".to_string(),
            ],
            
            #[cfg(feature = "history")]
            Self::Storage(msg) => vec![
                "Check database connectivity and permissions".to_string(),
                "Verify sufficient disk space".to_string(),
                "Ensure storage backend is running".to_string(),
                "Check data directory permissions".to_string(),
                format!("Storage error: {}", msg),
            ],
            
            #[cfg(feature = "security")]
            Self::AuthenticationFailed(msg) => vec![
                "Verify username and password".to_string(),
                "Check account status and expiration".to_string(),
                "Ensure proper authentication method".to_string(),
                "Review security configuration".to_string(),
                format!("Auth details: {}", msg),
            ],
            
            Self::Validation(msg) => vec![
                "Check input data format and values".to_string(),
                "Verify data meets validation constraints".to_string(),
                "Review validation rules and requirements".to_string(),
                format!("Validation details: {}", msg),
            ],
            
            _ => vec![
                "Check system logs for more details".to_string(),
                "Verify system configuration and state".to_string(),
                "Consider restarting affected components".to_string(),
                "Contact support if issue persists".to_string(),
            ],
        }
    }
    
    fn is_recoverable(&self) -> bool {
        !matches!(
            self,
            Self::Config(_) | Self::Io(_)
            #[cfg(feature = "security")] | Self::Security(_)
        )
    }
    
    fn retry_delay(&self) -> Option<std::time::Duration> {
        match self {
            Self::Protocol(_) => Some(std::time::Duration::from_secs(5)),
            #[cfg(feature = "mqtt")]
            Self::Mqtt(_) => Some(std::time::Duration::from_secs(5)),
            #[cfg(feature = "history")]
            Self::Storage(_) => Some(std::time::Duration::from_secs(10)),
            Self::Http(_) => Some(std::time::Duration::from_secs(2)),
            
            #[cfg(feature = "circuit-breaker")]
            Self::CircuitOpen => Some(std::time::Duration::from_secs(30)),
            
            _ => None,
        }
    }
}

// ============================================================================
// CONVENIENCE MACROS
// ============================================================================

/// Create a configuration error with formatted message
/// 
/// # Example
/// 
/// ```rust
/// use petra::config_error;
/// 
/// let error = config_error!("Invalid parameter '{}': expected > 0, got {}", param_name, value);
/// ```
#[macro_export]
macro_rules! config_error {
    ($($arg:tt)*) => {
        $crate::PlcError::Config(format!($($arg)*))
    };
}

/// Create a runtime error with formatted message
/// 
/// # Example
/// 
/// ```rust
/// use petra::runtime_error;
/// 
/// let error = runtime_error!("Engine execution failed: {}", reason);
/// ```
#[macro_export]
macro_rules! runtime_error {
    ($($arg:tt)*) => {
        $crate::PlcError::Runtime(format!($($arg)*))
    };
}

/// Create a signal error with formatted message
/// 
/// # Example
/// 
/// ```rust
/// use petra::signal_error;
/// 
/// let error = signal_error!("Signal '{}' type mismatch: {}", signal_name, details);
/// ```
#[macro_export]
macro_rules! signal_error {
    ($($arg:tt)*) => {
        $crate::PlcError::Signal(format!($($arg)*))
    };
}

/// Create a validation error with formatted message
/// 
/// # Example
/// 
/// ```rust
/// use petra::validation_error;
/// 
/// let error = validation_error!("Value {} is outside valid range [{}, {}]", value, min, max);
/// ```
#[macro_export]
macro_rules! validation_error {
    ($($arg:tt)*) => {
        $crate::PlcError::Validation(format!($($arg)*))
    };
}

// ============================================================================
// COMPREHENSIVE TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = PlcError::SignalNotFound("test_signal".to_string());
        assert_eq!(err.to_string(), "Signal 'test_signal' not found");
    }
    
    #[test]
    fn test_type_mismatch() {
        let err = PlcError::TypeMismatch {
            expected: "bool".to_string(),
            actual: "int".to_string(),
        };
        assert_eq!(err.to_string(), "Type mismatch: expected bool, got int");
    }
    
    #[test]
    fn test_error_categorization() {
        let config_err = PlcError::Config("test".to_string());
        assert_eq!(config_err.category(), ErrorCategory::Configuration);
        assert_eq!(config_err.severity(), ErrorSeverity::Fatal);
        assert!(!config_err.is_retryable());
        
        let signal_err = PlcError::SignalNotFound("test".to_string());
        assert_eq!(signal_err.category(), ErrorCategory::Runtime);
        assert_eq!(signal_err.severity(), ErrorSeverity::Info);
    }
    
    #[test]
    fn test_error_codes() {
        assert_eq!(PlcError::Config("test".to_string()).error_code(), 1000);
        assert_eq!(PlcError::Runtime("test".to_string()).error_code(), 2000);
        assert_eq!(PlcError::SignalNotFound("test".to_string()).error_code(), 2001);
    }
    
    #[test]
    fn test_recovery_strategies() {
        let config_err = PlcError::Config("test".to_string());
        assert_eq!(config_err.recovery_strategy(), RecoveryStrategy::Fatal);
        assert!(config_err.is_permanent());
        
        let protocol_err = PlcError::Protocol("test".to_string());
        assert_eq!(protocol_err.recovery_strategy(), RecoveryStrategy::Retry);
        assert!(protocol_err.is_retryable());
    }
    
    #[test]
    fn test_error_macros() {
        let err = config_error!("Invalid value: {}", 42);
        match err {
            PlcError::Config(msg) => assert!(msg.contains("Invalid value: 42")),
            _ => panic!("Wrong error type"),
        }
        
        let err = signal_error!("Signal {} not found", "test");
        match err {
            PlcError::Signal(msg) => assert!(msg.contains("Signal test not found")),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[cfg(feature = "enhanced-errors")]
    #[test]
    fn test_detailed_error_builder() {
        let err = PlcError::detailed("Test error")
            .at("test.rs:123")
            .with_context("signal", "test_signal")
            .with_context("block", "test_block")
            .with_error_code(9999)
            .build();
            
        match err {
            PlcError::Detailed { message, location, context, error_code, .. } => {
                assert_eq!(message, "Test error");
                assert_eq!(location, "test.rs:123");
                assert_eq!(context.get("signal"), Some(&"test_signal".to_string()));
                assert_eq!(context.get("block"), Some(&"test_block".to_string()));
                assert_eq!(error_code, Some(9999));
            }
            _ => panic!("Wrong error type"),
        }
    }
    
    #[cfg(feature = "error-recovery")]
    #[test]
    fn test_error_recovery() {
        let err = PlcError::SignalNotFound("test".to_string());
        let suggestions = err.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(err.is_recoverable());
        
        let config_err = PlcError::Config("test".to_string());
        assert!(!config_err.is_recoverable());
    }
    
    #[test]
    fn test_error_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let plc_err: PlcError = io_err.into();
        
        match plc_err {
            PlcError::Io(ref e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
            _ => panic!("Wrong error type"),
        }
    }
    
    #[test]
    fn test_result_type() {
        fn test_function() -> Result<i32> {
            Ok(42)
        }
        
        fn test_error_function() -> Result<i32> {
            Err(PlcError::Runtime("test error".to_string()))
        }
        
        assert_eq!(test_function().unwrap(), 42);
        assert!(test_error_function().is_err());
    }
}

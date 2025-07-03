//! # PETRA Configuration System
//!
//! ## Purpose & Overview
//! 
//! This module provides the comprehensive configuration system for PETRA that serves as the central
//! configuration hub for the entire automation system. It handles:
//!
//! - **YAML Configuration Loading** - Loads and parses YAML configuration files with validation
//! - **Type-Safe Configuration** - Provides strongly-typed configuration structures  
//! - **Feature-Specific Configs** - Conditionally compiled configuration sections based on feature flags
//! - **Schema Generation** - Generates JSON schemas for IDE support and validation
//! - **Migration Support** - Handles configuration version migrations and compatibility
//! - **Hot-Reload Support** - Enables runtime configuration changes without system restart
//! - **Cross-Validation** - Ensures signal references and block connections are valid
//!
//! ## Architecture & Interactions
//!
//! The configuration system is used by all PETRA modules and serves as the source of truth:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Configuration System                         │
//! │                      (src/config.rs)                           │
//! └─────────────────┬───────────────────────────────────────────────┘
//!                   │ Config structs & validation
//!                   │
//! ┌─────────────────▼───────────────────────────────────────────────┐
//! │ Core Engine Components                                          │
//! │ • src/main.rs - CLI loads configs and passes to engine         │
//! │ • src/engine.rs - Engine uses config for initialization        │
//! │ • src/signal.rs - Signal definitions from config               │
//! └─────────────────┬───────────────────────────────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────────────────────────────┐
//! │ Feature Modules (conditionally compiled)                       │
//! │ • src/blocks/* - Block configurations define automation logic  │
//! │ • src/protocols/* - Protocol settings configured here          │
//! │ • src/mqtt.rs - MQTT broker and topic configurations          │
//! │ • src/security.rs - Authentication and authorization settings  │
//! │ • src/history.rs - Historical data storage configuration       │
//! │ • src/alarms.rs - Alarm thresholds and notification settings   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Configuration Structure
//!
//! ```yaml
//! # Core engine settings (always present)
//! scan_time_ms: 100
//! max_scan_jitter_ms: 50
//! error_recovery: true
//! 
//! # Signal definitions (always present)
//! signals:
//!   - name: "temperature"
//!     type: "float"
//!     initial: 20.0
//!     units: "°C"
//! 
//! # Block definitions (always present)
//! blocks:
//!   - name: "temp_monitor"
//!     type: "GREATER_THAN"
//!     inputs:
//!       input: "temperature"
//!     outputs:
//!       output: "high_temp_alarm"
//!     params:
//!       threshold: 80.0
//! 
//! # Optional feature configurations (feature-gated)
//! mqtt:
//!   broker: "localhost:1883"
//!   client_id: "petra"
//! ```
//!
//! ## Performance Considerations
//!
//! - **Zero-Cost Abstractions**: Feature gates ensure unused features compile to nothing
//! - **Load-Time Validation**: All validation performed once at startup, not during runtime  
//! - **Efficient Cloning**: Configuration objects designed for low-cost cloning via Arc
//! - **Schema Validation**: Comprehensive validation minimizes runtime configuration errors
//! - **Memory Layout**: Optimized struct layouts for cache-friendly access patterns

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)] // Config types naturally repeat "Config"

use crate::{
    error::{PlcError, Result},
    value::{Value, ValueType, from_yaml_value},
    Features,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

// Feature-gated imports for enhanced functionality
#[cfg(feature = "schema-validation")]
use schemars::{JsonSchema, schema_for};

#[cfg(feature = "config-migration")]
use semver::Version;

// ============================================================================
// CORE CONFIGURATION STRUCTURES
// ============================================================================

/// Main PETRA configuration structure
/// 
/// This is the root configuration object that contains all settings for a PETRA system.
/// It supports feature-gated sections that are only included when corresponding features
/// are enabled, ensuring zero-cost abstractions for unused functionality.
/// 
/// # Thread Safety
/// 
/// This structure is designed to be safely cloned and shared across threads. All
/// mutable operations should be performed through controlled mutation methods.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::Config;
/// 
/// // Load from file with validation
/// let config = Config::from_file("petra.yaml")?;
/// 
/// // Validate all configuration sections
/// config.validate()?;
/// 
/// // Check feature compatibility with runtime
/// let features = petra::get_runtime_features();
/// config.check_feature_compatibility(&features)?;
/// 
/// // Access core settings
/// println!("Scan time: {}ms", config.scan_time_ms);
/// println!("Signals: {}", config.signals.len());
/// # Ok::<(), petra::PlcError>(())
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct Config {
    // ========================================================================
    // CORE ENGINE CONFIGURATION (always present)
    // ========================================================================
    
    /// Engine scan time in milliseconds (valid range: 1-10000ms)
    /// 
    /// This controls the fundamental execution frequency of the PETRA engine.
    /// Lower values provide faster response but higher CPU usage.
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    
    /// Maximum allowed scan jitter in milliseconds
    /// 
    /// When real-time features are enabled, this defines the maximum acceptable
    /// deviation from the target scan time before warnings are generated.
    #[serde(default = "default_max_jitter")]
    pub max_scan_jitter_ms: u64,
    
    /// Enable automatic error recovery mechanisms
    /// 
    /// When enabled, the engine will attempt to recover from transient errors
    /// rather than immediately shutting down.
    #[serde(default = "default_error_recovery")]
    pub error_recovery: bool,
    
    /// Maximum consecutive errors before emergency shutdown
    /// 
    /// Safety mechanism to prevent runaway error conditions from consuming
    /// system resources indefinitely.
    #[serde(default = "default_max_consecutive_errors")]
    pub max_consecutive_errors: u64,
    
    /// Engine restart delay after fatal errors (milliseconds)
    /// 
    /// When error recovery is enabled, this is the delay before attempting
    /// to restart the engine after a fatal error condition.
    #[serde(default = "default_restart_delay_ms")]
    pub restart_delay_ms: u64,
    
    // ========================================================================
    // SIGNAL AND BLOCK DEFINITIONS (always present)
    // ========================================================================
    
    /// Signal definitions for the system
    /// 
    /// Signals are the fundamental data points that flow through PETRA.
    /// At least one signal must be defined for the system to function.
    #[serde(default)]
    pub signals: Vec<SignalConfig>,
    
    /// Block definitions for automation logic
    /// 
    /// Blocks process signals and implement control algorithms. They define
    /// the automation logic and data flow patterns in the system.
    #[serde(default)]
    pub blocks: Vec<BlockConfig>,
    
    // ========================================================================
    // PROTOCOL CONFIGURATION (conditionally present)
    // ========================================================================
    
    /// Protocol drivers configuration
    /// 
    /// Defines how PETRA communicates with external devices and systems.
    /// Different protocol drivers are enabled via feature flags.
    pub protocols: Option<ProtocolConfig>,
    
    // ========================================================================
    // FEATURE-SPECIFIC CONFIGURATIONS (conditionally compiled)
    // ========================================================================
    
    /// MQTT protocol configuration
    /// 
    /// Only included when the "mqtt" feature is enabled. Configures
    /// MQTT broker connections, topics, and message handling.
    #[cfg(feature = "mqtt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    
    /// Security and authentication configuration
    /// 
    /// Only included when the "security" feature is enabled. Configures
    /// authentication methods, authorization rules, and audit logging.
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    
    /// Historical data storage configuration
    /// 
    /// Only included when the "history" feature is enabled. Configures
    /// data persistence, retention policies, and compression settings.
    #[cfg(feature = "history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
    
    /// Alarm management configuration
    /// 
    /// Only included when the "alarms" feature is enabled. Configures
    /// alarm definitions, thresholds, and notification methods.
    #[cfg(feature = "alarms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<AlarmConfig>,
    
    /// Web server and API configuration
    /// 
    /// Only included when the "web" feature is enabled. Configures
    /// HTTP/WebSocket server settings and static file serving.
    #[cfg(feature = "web")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<WebConfig>,
    
    /// Validation rules configuration
    /// 
    /// Only included when the "validation" feature is enabled. Configures
    /// input validation rules and error handling policies.
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
    
    /// Metrics and monitoring configuration
    /// 
    /// Only included when the "metrics" feature is enabled. Configures
    /// performance monitoring and Prometheus export settings.
    #[cfg(feature = "metrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsConfig>,
    
    /// Real-time configuration
    /// 
    /// Only included when the "realtime" feature is enabled. Configures
    /// real-time scheduling, CPU affinity, and memory locking.
    #[cfg(feature = "realtime")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime: Option<RealtimeConfig>,
    
    // ========================================================================
    // METADATA AND VERSIONING
    // ========================================================================
    
    /// Configuration schema version for migration support
    #[serde(default = "default_version")]
    pub version: String,
    
    /// Human-readable configuration description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Configuration author/creator for audit trail
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Configuration creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<SystemTime>,
    
    /// Configuration last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<SystemTime>,
    
    /// Configuration tags for organization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Additional custom metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

// ============================================================================
// SIGNAL CONFIGURATION
// ============================================================================

/// Signal configuration defining a data point in the system
/// 
/// Signals are the fundamental data elements that flow through PETRA. They can represent
/// sensor readings, setpoints, status flags, computed values, or any other data values
/// in the automation system. Each signal has a strongly-typed definition that ensures
/// type safety throughout the system.
/// 
/// # Naming Convention
/// 
/// Signal names should follow the pattern: `component.subcategory.signal_name`
/// Examples: `plc1.tank1.level`, `system.heartbeat`, `zone1.temperature.sensor1`
/// 
/// # Examples
/// 
/// ```yaml
/// signals:
///   - name: "system.heartbeat"
///     type: "bool"
///     initial: false
///     description: "System heartbeat indicator"
///     category: "System"
///     
///   - name: "tank1.level"
///     type: "float"
///     initial: 50.0
///     units: "cm"
///     min_value: 0.0
///     max_value: 200.0
///     description: "Tank 1 water level"
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct SignalConfig {
    /// Unique signal name following naming convention
    /// 
    /// Must be unique within the configuration and follow the pattern
    /// `component.subcategory.signal_name` for organizational clarity.
    pub name: String,
    
    /// Signal data type (bool, integer, float, string, etc.)
    /// 
    /// Defines the type of data this signal carries. Basic types are always
    /// available, extended types require the "extended-types" feature.
    #[serde(rename = "type")]
    pub signal_type: String,
    
    /// Initial value when signal is created
    /// 
    /// Must match the signal_type. Used for signal initialization at startup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<serde_yaml::Value>,
    
    /// Human-readable description for documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Engineering units (e.g., "°C", "bar", "rpm", "m/s")
    /// 
    /// Only available with the "engineering-types" feature. Provides
    /// unit information for human interfaces and data export.
    #[cfg(feature = "engineering-types")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    
    /// Minimum valid value for range validation
    /// 
    /// Only available with the "engineering-types" feature. Values below
    /// this threshold will trigger validation warnings or errors.
    #[cfg(feature = "engineering-types")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    
    /// Maximum valid value for range validation
    /// 
    /// Only available with the "engineering-types" feature. Values above
    /// this threshold will trigger validation warnings or errors.
    #[cfg(feature = "engineering-types")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    
    /// Enable quality codes for this signal
    /// 
    /// Only available with the "quality-codes" feature. When enabled,
    /// each signal value includes quality information (good, bad, uncertain).
    #[cfg(feature = "quality-codes")]
    #[serde(default)]
    pub quality_enabled: bool,
    
    /// Enable historical logging for this signal
    /// 
    /// Only available with the "history" feature. When enabled, all
    /// changes to this signal are logged to the historical database.
    #[cfg(feature = "history")]
    #[serde(default)]
    pub log_to_history: bool,
    
    /// Logging interval in milliseconds (0 = on change only)
    /// 
    /// Only available with the "history" feature. Defines how frequently
    /// to log this signal's value. Zero means log only on value changes.
    #[cfg(feature = "history")]
    #[serde(default)]
    pub log_interval_ms: u64,
    
    /// Enable alarm monitoring for this signal
    /// 
    /// Only available with the "alarms" feature. When enabled, this
    /// signal will be monitored for alarm conditions.
    #[cfg(feature = "alarms")]
    #[serde(default)]
    pub enable_alarms: bool,
    
    /// Signal category for organization and filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    /// Data source identifier (e.g., "PLC-1", "Sensor", "Computed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    /// Expected update frequency hint in milliseconds
    /// 
    /// Provides guidance for monitoring systems about how often this
    /// signal should be updated. Used for detecting stale data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_frequency_ms: Option<u64>,
    
    /// Signal tags for organization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Validation rules for this signal
    /// 
    /// Only available with the "validation" feature. Defines specific
    /// validation rules that apply to this signal's values.
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<SignalValidationConfig>,
    
    /// Additional custom metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Signal-specific validation configuration
/// 
/// Only available with the "validation" feature. Defines how validation
/// failures should be handled for a specific signal.
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct SignalValidationConfig {
    /// Validation rules to apply to this signal
    pub rules: Vec<ValidationRule>,
    
    /// Action to take when validation fails
    #[serde(default = "default_validation_action")]
    pub on_failure: ValidationAction,
    
    /// Whether to log validation failures for debugging
    #[serde(default = "default_log_failures")]
    pub log_failures: bool,
}

// ============================================================================
// BLOCK CONFIGURATION
// ============================================================================

/// Block configuration defining automation logic elements
/// 
/// Blocks are the building blocks of automation logic in PETRA. They process signals
/// and implement control algorithms, from simple logic gates (AND, OR, NOT) to complex
/// controllers (PID, state machines). Blocks define the data flow and processing
/// patterns that implement the automation logic.
/// 
/// # Block Types
/// 
/// - **Logic Blocks**: AND, OR, NOT, XOR for boolean logic
/// - **Comparison Blocks**: GT, LT, EQ, GTE, LTE for numeric comparisons  
/// - **Math Blocks**: ADD, SUB, MUL, DIV for arithmetic operations
/// - **Timer Blocks**: ON_DELAY, OFF_DELAY, PULSE for timing functions
/// - **Control Blocks**: PID, RAMP, LIMIT for process control
/// - **Custom Blocks**: User-defined blocks for specialized logic
/// 
/// # Examples
/// 
/// ```yaml
/// blocks:
///   - name: "tank_level_alarm"
///     type: "GREATER_THAN"
///     inputs:
///       input: "tank1.level"
///     outputs:
///       output: "tank1.high_alarm"
///     params:
///       threshold: 180.0
///       
///   - name: "pump_control"
///     type: "PID"
///     inputs:
///       process_variable: "tank1.level"
///       setpoint: "tank1.setpoint"
///     outputs:
///       output: "pump1.speed"
///     params:
///       kp: 1.0
///       ki: 0.1
///       kd: 0.01
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct BlockConfig {
    /// Unique block name within the configuration
    /// 
    /// Must be unique within the configuration. Should be descriptive
    /// of the block's purpose (e.g., "tank1_level_control", "safety_interlock").
    pub name: String,
    
    /// Block type identifier (AND, OR, PID, TIMER, etc.)
    /// 
    /// Determines which block implementation to instantiate. Must match
    /// a registered block type in the block factory.
    #[serde(rename = "type")]
    pub block_type: String,
    
    /// Input signal mappings (input_name -> signal_name)
    /// 
    /// Maps block input names to signal names in the signal bus.
    /// All referenced signals must exist in the configuration.
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    /// Output signal mappings (output_name -> signal_name)
    /// 
    /// Maps block output names to signal names in the signal bus.
    /// All referenced signals must exist in the configuration.
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Block parameters for configuration
    /// 
    /// Block-specific parameters that control behavior. Parameter
    /// validation is performed by the individual block implementations.
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    
    /// Block execution priority (higher = earlier execution)
    /// 
    /// Determines execution order within a scan cycle. Higher priority
    /// blocks execute before lower priority blocks.
    #[serde(default)]
    pub priority: i32,
    
    /// Whether block is enabled (disabled blocks are skipped)
    /// 
    /// Disabled blocks are not executed but remain in the configuration
    /// for easy enabling/disabling during commissioning.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Human-readable description for documentation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Block category for organization (e.g., "Safety", "Control", "Logic")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    /// Block tags for organization and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Circuit breaker configuration for fault tolerance
    /// 
    /// Only available with the "circuit-breaker" feature. Provides
    /// automatic fault isolation when blocks repeatedly fail.
    #[cfg(feature = "circuit-breaker")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
    
    /// Enhanced monitoring for this block
    /// 
    /// Only available with the "enhanced-monitoring" feature. Enables
    /// detailed performance and state monitoring for this block.
    #[cfg(feature = "enhanced-monitoring")]
    #[serde(default)]
    pub enhanced_monitoring: bool,
    
    /// Additional custom metadata for extensibility
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Circuit breaker configuration for fault tolerance
/// 
/// Only available with the "circuit-breaker" feature. Implements the circuit
/// breaker pattern to provide automatic fault isolation and recovery.
#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    #[serde(default = "default_failure_threshold")]
    pub failure_threshold: u32,
    
    /// Time to wait before attempting recovery (milliseconds)
    #[serde(default = "default_recovery_timeout")]
    pub recovery_timeout_ms: u64,
    
    /// Maximum calls allowed in half-open state
    #[serde(default = "default_half_open_max_calls")]
    pub half_open_max_calls: u32,
}

// ============================================================================
// PROTOCOL CONFIGURATION
// ============================================================================

/// Protocol drivers configuration
/// 
/// Defines how PETRA communicates with external devices and systems. Different
/// protocol drivers are conditionally compiled based on feature flags, ensuring
/// that only required protocol code is included in the binary.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ProtocolConfig {
    /// Siemens S7 protocol configuration
    /// 
    /// Only available with the "s7-support" feature. Configures communication
    /// with Siemens S7 PLCs using the S7 communication protocol.
    #[cfg(feature = "s7-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s7: Option<S7Config>,
    
    /// Modbus protocol configuration
    /// 
    /// Only available with the "modbus-support" feature. Configures communication
    /// with Modbus devices over TCP/RTU.
    #[cfg(feature = "modbus-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modbus: Option<ModbusConfig>,
    
    /// OPC-UA protocol configuration
    /// 
    /// Only available with the "opcua-support" feature. Configures communication
    /// with OPC-UA servers for industrial data exchange.
    #[cfg(feature = "opcua-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcua: Option<OpcuaConfig>,
}

// ============================================================================
// FEATURE-SPECIFIC CONFIGURATION STRUCTURES
// ============================================================================
// 
// The following structures are conditionally compiled based on feature flags.
// This ensures zero-cost abstractions - unused features compile to nothing.

/// MQTT protocol configuration
/// 
/// Only available with the "mqtt" feature. Configures MQTT client connections,
/// topic subscriptions, and message publishing for IoT integration.
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MqttConfig {
    /// MQTT broker address (host:port format)
    pub broker: String,
    
    /// MQTT client ID (must be unique per broker)
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,
    
    /// Username for broker authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    /// Password for broker authentication  
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    
    /// Keep alive interval in seconds
    #[serde(default = "default_mqtt_keep_alive")]
    pub keep_alive: u16,
    
    /// Connection timeout in milliseconds
    #[serde(default = "default_connection_timeout")]
    pub timeout_ms: u64,
    
    /// Enable clean session mode
    #[serde(default = "default_clean_session")]
    pub clean_session: bool,
    
    /// Topic subscriptions for incoming data
    #[serde(default)]
    pub subscriptions: Vec<MqttSubscription>,
    
    /// Topic publications for outgoing data
    #[serde(default)]
    pub publications: Vec<MqttPublication>,
    
    /// TLS configuration for secure connections
    #[cfg(feature = "mqtt-tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<MqttTlsConfig>,
    
    /// Last will and testament for connection loss handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_will: Option<MqttLastWill>,
}

/// MQTT subscription configuration
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MqttSubscription {
    /// MQTT topic to subscribe to (supports wildcards)
    pub topic: String,
    
    /// Signal name to write received values to
    pub signal: String,
    
    /// Quality of Service level (0, 1, or 2)
    #[serde(default)]
    pub qos: u8,
    
    /// Expected data type of received messages
    #[serde(default = "default_mqtt_data_type")]
    pub data_type: String,
}

/// MQTT publication configuration
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MqttPublication {
    /// MQTT topic to publish to
    pub topic: String,
    
    /// Signal name to read values from
    pub signal: String,
    
    /// Quality of Service level (0, 1, or 2)
    #[serde(default)]
    pub qos: u8,
    
    /// Whether to retain messages on broker
    #[serde(default)]
    pub retain: bool,
    
    /// Publish interval in milliseconds (0 = on change only)
    #[serde(default)]
    pub interval_ms: u64,
}

/// MQTT TLS configuration
#[cfg(all(feature = "mqtt", feature = "mqtt-tls"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MqttTlsConfig {
    /// CA certificate file path
    pub ca_cert: PathBuf,
    
    /// Client certificate file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<PathBuf>,
    
    /// Client private key file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<PathBuf>,
    
    /// Whether to verify server certificate
    #[serde(default = "default_verify_server")]
    pub verify_server: bool,
}

/// MQTT last will configuration
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MqttLastWill {
    /// Topic to publish will message to
    pub topic: String,
    
    /// Will message content
    pub message: String,
    
    /// Quality of Service level (0, 1, or 2)
    #[serde(default)]
    pub qos: u8,
    
    /// Whether to retain will message
    #[serde(default)]
    pub retain: bool,
}

/// Security and authentication configuration
/// 
/// Only available with the "security" feature. Configures authentication
/// methods, authorization rules, and security audit logging.
#[cfg(feature = "security")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct SecurityConfig {
    /// Enable security features globally
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Basic authentication configuration
    #[cfg(feature = "basic-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuthConfig>,
    
    /// JWT authentication configuration
    #[cfg(feature = "jwt-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
    
    /// Role-based access control configuration
    #[cfg(feature = "rbac")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rbac: Option<RbacConfig>,
    
    /// Audit logging configuration
    #[cfg(feature = "audit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
}

/// Basic authentication configuration
#[cfg(all(feature = "security", feature = "basic-auth"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct BasicAuthConfig {
    /// User credentials (username -> password hash)
    pub users: HashMap<String, String>,
    
    /// Password hashing algorithm
    #[serde(default = "default_hash_algorithm")]
    pub hash_algorithm: String,
}

/// JWT authentication configuration
#[cfg(all(feature = "security", feature = "jwt-auth"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct JwtConfig {
    /// JWT signing secret
    pub secret: String,
    
    /// Token expiration time in seconds
    #[serde(default = "default_jwt_expiration")]
    pub expiration_secs: u64,
    
    /// Token issuer identifier
    #[serde(default = "default_jwt_issuer")]
    pub issuer: String,
    
    /// Allowed audiences
    #[serde(default)]
    pub audiences: Vec<String>,
}

/// Role-based access control configuration
#[cfg(all(feature = "security", feature = "rbac"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct RbacConfig {
    /// Role definitions (role_name -> permissions)
    pub roles: HashMap<String, Vec<String>>,
    
    /// User to role mappings
    pub users: HashMap<String, Vec<String>>,
    
    /// Default role for unauthenticated users
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_role: Option<String>,
}

/// Audit logging configuration
#[cfg(all(feature = "security", feature = "audit"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct AuditConfig {
    /// Enable audit logging
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Log file path
    pub log_file: PathBuf,
    
    /// Maximum log file size in MB
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    
    /// Number of rotated log files to keep
    #[serde(default = "default_rotate_count")]
    pub rotate_count: u32,
    
    /// Events to audit
    #[serde(default = "default_audit_events")]
    pub events: Vec<String>,
}

/// Historical data storage configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct HistoryConfig {
    /// Storage backend type (parquet, clickhouse, s3, etc.)
    #[serde(default = "default_storage_backend")]
    pub backend: String,
    
    /// Data directory for file-based backends
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    
    /// Data retention period in days (0 = unlimited)
    #[serde(default)]
    pub retention_days: u32,
    
    /// Compression algorithm (none, snappy, gzip, lz4)
    #[serde(default = "default_compression")]
    pub compression: String,
    
    /// Batch size for writes
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Flush interval in milliseconds
    #[serde(default = "default_flush_interval")]
    pub flush_interval_ms: u64,
    
    /// ClickHouse specific configuration
    #[cfg(feature = "clickhouse")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clickhouse: Option<ClickHouseConfig>,
    
    /// S3 specific configuration
    #[cfg(feature = "s3-storage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3: Option<S3Config>,
}

/// ClickHouse storage configuration
#[cfg(all(feature = "history", feature = "clickhouse"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ClickHouseConfig {
    /// ClickHouse server URL
    pub url: String,
    
    /// Database name
    pub database: String,
    
    /// Table name prefix
    #[serde(default = "default_table_prefix")]
    pub table_prefix: String,
    
    /// Connection pool size
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

/// S3 storage configuration
#[cfg(all(feature = "history", feature = "s3-storage"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct S3Config {
    /// S3 bucket name
    pub bucket: String,
    
    /// Object key prefix
    #[serde(default = "default_s3_prefix")]
    pub prefix: String,
    
    /// AWS region
    pub region: String,
    
    /// AWS access key ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    
    /// AWS secret access key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
}

/// Alarm configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct AlarmConfig {
    /// Global alarm enable/disable
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Alarm definitions
    #[serde(default)]
    pub alarms: Vec<AlarmDefinition>,
    
    /// Notification channels
    #[serde(default)]
    pub channels: Vec<NotificationChannel>,
    
    /// Default severity for alarms without explicit severity
    #[serde(default = "default_alarm_severity")]
    pub default_severity: String,
    
    /// Alarm acknowledgment timeout in seconds
    #[serde(default = "default_ack_timeout")]
    pub ack_timeout_secs: u64,
}

/// Individual alarm definition
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct AlarmDefinition {
    /// Unique alarm name
    pub name: String,
    
    /// Alarm trigger condition expression
    pub condition: String,
    
    /// Severity level (info, warning, critical, fatal)
    #[serde(default = "default_alarm_severity")]
    pub severity: String,
    
    /// Alarm message template
    pub message: String,
    
    /// Notification channels to use
    #[serde(default)]
    pub channels: Vec<String>,
    
    /// Delay before alarm triggers (milliseconds)
    #[serde(default)]
    pub delay_ms: u64,
    
    /// Auto-acknowledge timeout (seconds, 0 = manual only)
    #[serde(default)]
    pub auto_ack_secs: u64,
}

/// Notification channel configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct NotificationChannel {
    /// Channel name
    pub name: String,
    
    /// Channel type (email, sms, webhook, etc.)
    #[serde(rename = "type")]
    pub channel_type: String,
    
    /// Channel-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_yaml::Value>,
    
    /// Minimum severity to notify
    #[serde(default = "default_min_severity")]
    pub min_severity: String,
    
    /// Rate limiting (max notifications per hour)
    #[serde(default = "default_rate_limit")]
    pub rate_limit: u32,
}

/// Web server configuration
#[cfg(feature = "web")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct WebConfig {
    /// HTTP server bind address
    #[serde(default = "default_web_bind")]
    pub bind_address: String,
    
    /// HTTP server port
    #[serde(default = "default_web_port")]
    pub port: u16,
    
    /// Enable WebSocket support
    #[serde(default = "default_websocket_enabled")]
    pub websocket_enabled: bool,
    
    /// Static file directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_dir: Option<PathBuf>,
    
    /// CORS allowed origins
    #[serde(default)]
    pub cors_origins: Vec<String>,
    
    /// TLS configuration
    #[cfg(feature = "web-tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<WebTlsConfig>,
}

/// Web TLS configuration
#[cfg(all(feature = "web", feature = "web-tls"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct WebTlsConfig {
    /// Certificate file path
    pub cert_file: PathBuf,
    
    /// Private key file path
    pub key_file: PathBuf,
}

/// Validation framework configuration
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ValidationConfig {
    /// Global validation enable/disable
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Global validation rules
    #[serde(default)]
    pub rules: Vec<ValidationRule>,
    
    /// Default action on validation failure
    #[serde(default = "default_validation_action")]
    pub default_action: ValidationAction,
    
    /// Enable validation result caching
    #[serde(default)]
    pub cache_results: bool,
}

/// Metrics configuration
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct MetricsConfig {
    /// Enable metrics collection
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Prometheus exporter bind address
    #[serde(default = "default_metrics_bind")]
    pub bind_address: String,
    
    /// Prometheus exporter port
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    
    /// Metric collection interval (milliseconds)
    #[serde(default = "default_metrics_interval")]
    pub interval_ms: u64,
    
    /// Include enhanced metrics
    #[cfg(feature = "enhanced-monitoring")]
    #[serde(default)]
    pub enhanced_metrics: bool,
}

/// Real-time configuration
#[cfg(feature = "realtime")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct RealtimeConfig {
    /// Enable real-time features
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Real-time priority (1-99)
    #[serde(default = "default_rt_priority")]
    pub priority: u8,
    
    /// CPU affinity mask
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_affinity: Option<Vec<usize>>,
    
    /// Lock memory to prevent swapping
    #[serde(default = "default_lock_memory")]
    pub lock_memory: bool,
    
    /// Pre-allocate memory (MB)
    #[serde(default)]
    pub preallocate_mb: u64,
}

/// S7 protocol configuration
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct S7Config {
    /// PLC connections
    #[serde(default)]
    pub connections: Vec<S7Connection>,
    
    /// Global connection timeout (milliseconds)
    #[serde(default = "default_connection_timeout")]
    pub timeout_ms: u64,
    
    /// Retry count for failed operations
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
}

/// Individual S7 connection
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct S7Connection {
    /// Connection name
    pub name: String,
    
    /// PLC IP address
    pub ip: String,
    
    /// Rack number
    pub rack: u16,
    
    /// Slot number
    pub slot: u16,
    
    /// Connection type (PG, OP, S7_BASIC)
    #[serde(default = "default_s7_connection_type")]
    pub connection_type: String,
    
    /// Data areas to read/write
    #[serde(default)]
    pub data_areas: Vec<S7DataArea>,
}

/// S7 data area configuration
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct S7DataArea {
    /// Area type (DB, MB, IB, QB)
    pub area: String,
    
    /// DB number (for DB areas)
    #[serde(default)]
    pub db_number: u16,
    
    /// Start offset
    pub offset: u32,
    
    /// Length in bytes
    pub length: u32,
    
    /// Associated signal prefix
    pub signal_prefix: String,
}

/// Modbus protocol configuration
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ModbusConfig {
    /// Modbus connections
    #[serde(default)]
    pub connections: Vec<ModbusConnection>,
    
    /// Global timeout (milliseconds)
    #[serde(default = "default_connection_timeout")]
    pub timeout_ms: u64,
}

/// Individual Modbus connection
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ModbusConnection {
    /// Connection name
    pub name: String,
    
    /// Connection type (tcp, rtu)
    #[serde(rename = "type")]
    pub connection_type: String,
    
    /// Server address (IP:port for TCP, serial port for RTU)
    pub address: String,
    
    /// Unit/Slave ID
    pub unit_id: u8,
    
    /// Register mappings
    #[serde(default)]
    pub registers: Vec<ModbusRegister>,
}

/// Modbus register mapping
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ModbusRegister {
    /// Register type (coil, discrete, holding, input)
    #[serde(rename = "type")]
    pub register_type: String,
    
    /// Starting address
    pub address: u16,
    
    /// Number of registers
    pub count: u16,
    
    /// Associated signal name or prefix
    pub signal: String,
    
    /// Data type for interpretation
    #[serde(default = "default_modbus_data_type")]
    pub data_type: String,
}

/// OPC-UA configuration
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct OpcuaConfig {
    /// OPC-UA server endpoint
    pub endpoint: String,
    
    /// Security policy
    #[serde(default = "default_opcua_security")]
    pub security_policy: String,
    
    /// Security mode
    #[serde(default = "default_opcua_security_mode")]
    pub security_mode: String,
    
    /// User authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_auth: Option<OpcuaAuth>,
    
    /// Certificate configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificates: Option<OpcuaCertConfig>,
    
    /// Subscriptions
    #[serde(default)]
    pub subscriptions: Vec<OpcuaSubscription>,
}

/// OPC-UA authentication
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct OpcuaAuth {
    /// Authentication type (anonymous, username, certificate)
    #[serde(rename = "type")]
    pub auth_type: String,
    
    /// Username (for username auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    /// Password (for username auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// OPC-UA certificate configuration
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct OpcuaCertConfig {
    /// Client certificate path
    pub client_cert: PathBuf,
    
    /// Client private key path
    pub client_key: PathBuf,
    
    /// Trusted server certificates directory
    pub trusted_certs_dir: PathBuf,
}

/// OPC-UA subscription
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct OpcuaSubscription {
    /// Node ID to subscribe to
    pub node_id: String,
    
    /// Associated signal name
    pub signal: String,
    
    /// Sampling interval (milliseconds)
    #[serde(default = "default_sampling_interval")]
    pub sampling_interval_ms: u64,
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================
// 
// These functions provide sensible defaults for configuration fields.
// They are used by serde for deserialization when fields are missing.

// Core engine defaults
const fn default_scan_time() -> u64 { 100 }
const fn default_max_jitter() -> u64 { 50 }
const fn default_error_recovery() -> bool { true }
const fn default_max_consecutive_errors() -> u64 { 10 }
const fn default_restart_delay_ms() -> u64 { 5000 }
const fn default_enabled() -> bool { true }
fn default_version() -> String { "1.0".to_string() }

// Protocol defaults
const fn default_connection_timeout() -> u64 { 5000 }
const fn default_retry_count() -> u32 { 3 }
fn default_s7_connection_type() -> String { "PG".to_string() }
fn default_modbus_data_type() -> String { "int16".to_string() }
fn default_opcua_security() -> String { "None".to_string() }
fn default_opcua_security_mode() -> String { "None".to_string() }
const fn default_sampling_interval() -> u64 { 1000 }

// MQTT defaults
fn default_mqtt_client_id() -> String { "petra".to_string() }
const fn default_mqtt_keep_alive() -> u16 { 60 }
const fn default_clean_session() -> bool { true }
fn default_mqtt_data_type() -> String { "float".to_string() }
const fn default_verify_server() -> bool { true }

// Security defaults
fn default_hash_algorithm() -> String { "argon2".to_string() }
const fn default_jwt_expiration() -> u64 { 3600 }
fn default_jwt_issuer() -> String { "petra".to_string() }
const fn default_max_file_size() -> u64 { 100 }
const fn default_rotate_count() -> u32 { 10 }
fn default_audit_events() -> Vec<String> {
    vec![
        "login".to_string(),
        "logout".to_string(),
        "config_change".to_string(),
        "alarm_ack".to_string(),
    ]
}

// Storage defaults
fn default_storage_backend() -> String { "parquet".to_string() }
fn default_data_dir() -> PathBuf { PathBuf::from("./data") }
fn default_compression() -> String { "snappy".to_string() }
const fn default_batch_size() -> usize { 1000 }
const fn default_flush_interval() -> u64 { 5000 }
fn default_table_prefix() -> String { "petra_".to_string() }
const fn default_pool_size() -> u32 { 10 }
fn default_s3_prefix() -> String { "petra/".to_string() }

// Alarm defaults
fn default_alarm_severity() -> String { "warning".to_string() }
const fn default_ack_timeout() -> u64 { 3600 }
fn default_min_severity() -> String { "info".to_string() }
const fn default_rate_limit() -> u32 { 100 }

// Web defaults
fn default_web_bind() -> String { "0.0.0.0".to_string() }
const fn default_web_port() -> u16 { 8080 }
const fn default_websocket_enabled() -> bool { true }

// Metrics defaults
fn default_metrics_bind() -> String { "0.0.0.0".to_string() }
const fn default_metrics_port() -> u16 { 9090 }
const fn default_metrics_interval() -> u64 { 10000 }

// Real-time defaults
const fn default_rt_priority() -> u8 { 50 }
const fn default_lock_memory() -> bool { true }

// Validation defaults
#[cfg(feature = "validation")]
const fn default_validation_action() -> ValidationAction { ValidationAction::Log }
#[cfg(feature = "validation")]
const fn default_log_failures() -> bool { true }

// Circuit breaker defaults
#[cfg(feature = "circuit-breaker")]
const fn default_failure_threshold() -> u32 { 5 }
#[cfg(feature = "circuit-breaker")]
const fn default_recovery_timeout() -> u64 { 30000 }
#[cfg(feature = "circuit-breaker")]
const fn default_half_open_max_calls() -> u32 { 3 }

// ============================================================================
// VALIDATION TRAITS AND IMPLEMENTATIONS
// ============================================================================

/// Trait for configuration validation
/// 
/// All configuration types implement this trait to provide comprehensive
/// validation of their settings. Validation is performed at load time
/// to catch configuration errors early.
pub trait Validatable {
    /// Validate this configuration object
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Config` with detailed error message if validation fails.
    fn validate(&self) -> Result<()>;
}

/// Validation action enumeration
/// 
/// Only available with the "validation" feature. Defines how the system
/// should respond when validation rules are violated.
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub enum ValidationAction {
    /// Log the error and continue processing
    Log,
    /// Reject the value and use previous value
    Reject,
    /// Stop processing and raise fatal error
    Stop,
}

/// Validation rule definition
/// 
/// Only available with the "validation" feature. Defines a specific
/// validation rule that can be applied to signal values.
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "schema-validation", derive(JsonSchema))]
pub struct ValidationRule {
    /// Unique rule name for identification
    pub name: String,
    
    /// Rule type (range, regex, custom, etc.)
    pub rule_type: String,
    
    /// Rule-specific parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_yaml::Value>,
    
    /// Custom error message template
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

// ============================================================================
// CORE IMPLEMENTATION
// ============================================================================

impl Config {
    /// Load configuration from a YAML file with comprehensive validation
    /// 
    /// This method loads a configuration file, performs full validation,
    /// and returns a ready-to-use configuration object.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the YAML configuration file
    /// 
    /// # Errors
    /// 
    /// Returns errors for:
    /// - File I/O issues (file not found, permission denied, etc.)
    /// - YAML parsing errors (invalid syntax, type mismatches)
    /// - Configuration validation failures (invalid values, missing references)
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::Config;
    /// 
    /// // Load and validate configuration
    /// let config = Config::from_file("petra.yaml")?;
    /// 
    /// // Configuration is now ready for use
    /// println!("Loaded {} signals", config.signals.len());
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading configuration from: {}", path.display());
        
        // Read file content
        let content = std::fs::read_to_string(path)
            .map_err(|e| PlcError::Config(format!(
                "Failed to read config file '{}': {}", 
                path.display(), e
            )))?;
        
        // Parse YAML content
        let mut config: Config = serde_yaml::from_str(&content)
            .map_err(|e| PlcError::Config(format!(
                "Failed to parse config file '{}': {}", 
                path.display(), e
            )))?;
        
        // Set metadata
        config.modified_at = Some(SystemTime::now());
        
        // Perform comprehensive validation
        config.validate()?;
        
        debug!(
            "Configuration loaded successfully: {} signals, {} blocks", 
            config.signals.len(), 
            config.blocks.len()
        );
        
        Ok(config)
    }
    
    /// Save configuration to a YAML file
    /// 
    /// Updates the modification timestamp and writes the configuration
    /// to the specified file in YAML format.
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path where to save the configuration
    /// 
    /// # Errors
    /// 
    /// Returns errors for file I/O issues or serialization failures.
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Update modification timestamp
        self.modified_at = Some(SystemTime::now());
        
        // Serialize to YAML
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| PlcError::Config(format!(
                "Failed to serialize configuration: {}", e
            )))?;
        
        // Write to file
        std::fs::write(path, yaml)
            .map_err(|e| PlcError::Config(format!(
                "Failed to write config file '{}': {}", 
                path.display(), e
            )))?;
        
        info!("Configuration saved to: {}", path.display());
        Ok(())
    }
    
    /// Save configuration as JSON format
    /// 
    /// Useful for integration with JSON-based tools and validation systems.
    pub fn save_as_json<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PlcError::Config(format!(
                "Failed to serialize configuration as JSON: {}", e
            )))?;
        
        std::fs::write(path, json)
            .map_err(|e| PlcError::Config(format!(
                "Failed to write JSON config file '{}': {}", 
                path.display(), e
            )))?;
        
        info!("Configuration saved as JSON to: {}", path.display());
        Ok(())
    }
    
    /// Validate the entire configuration with comprehensive checks
    /// 
    /// This method performs multi-layered validation:
    /// 1. Core engine settings validation
    /// 2. Individual signal and block validation  
    /// 3. Cross-reference validation (signal references in blocks)
    /// 4. Feature-specific configuration validation
    /// 5. Consistency checks across the entire configuration
    /// 
    /// # Errors
    /// 
    /// Returns detailed error messages for any validation failures,
    /// including the specific field and reason for failure.
    pub fn validate(&self) -> Result<()> {
        // Core engine validation
        self.validate_engine_settings()?;
        
        // Signal validation
        self.validate_signals()?;
        
        // Block validation  
        self.validate_blocks()?;
        
        // Feature-specific validation
        self.validate_feature_configs()?;
        
        // Cross-validation
        self.validate_signal_references()?;
        
        debug!("Configuration validation completed successfully");
        Ok(())
    }
    
    /// Validate core engine settings
    /// 
    /// Ensures engine timing parameters are within acceptable ranges
    /// and logically consistent with each other.
    fn validate_engine_settings(&self) -> Result<()> {
        // Scan time validation
        if self.scan_time_ms == 0 {
            return Err(PlcError::Config("Scan time cannot be zero".to_string()));
        }
        
        if self.scan_time_ms > 60_000 {
            return Err(PlcError::Config(
                "Scan time too large (maximum 60 seconds)".to_string()
            ));
        }
        
        if self.scan_time_ms < 10 {
            warn!("Very fast scan time ({}ms) may cause high CPU usage", self.scan_time_ms);
        }
        
        // Jitter validation
        if self.max_scan_jitter_ms > self.scan_time_ms {
            return Err(PlcError::Config(
                "Maximum jitter cannot exceed scan time".to_string()
            ));
        }
        
        // Error handling validation
        if self.max_consecutive_errors == 0 {
            return Err(PlcError::Config(
                "Maximum consecutive errors must be greater than zero".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Validate signal definitions
    /// 
    /// Checks signal names, types, and ensures no duplicates exist.
    fn validate_signals(&self) -> Result<()> {
        if self.signals.is_empty() {
            return Err(PlcError::Config(
                "Configuration must have at least one signal".to_string()
            ));
        }
        
        let mut signal_names = HashSet::new();
        
        for signal in &self.signals {
            // Check for duplicate names
            if !signal_names.insert(&signal.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate signal name: '{}'", signal.name
                )));
            }
            
            // Validate individual signal
            signal.validate()?;
        }
        
        Ok(())
    }
    
    /// Validate block definitions
    /// 
    /// Checks block names for uniqueness and validates individual blocks.
    fn validate_blocks(&self) -> Result<()> {
        let mut block_names = HashSet::new();
        
        for block in &self.blocks {
            // Check for duplicate names
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate block name: '{}'", block.name
                )));
            }
            
            // Validate individual block
            block.validate()?;
        }
        
        Ok(())
    }
    
    /// Validate feature-specific configurations
    /// 
    /// Calls validation methods on all enabled feature configurations.
    fn validate_feature_configs(&self) -> Result<()> {
        // Protocol validation
        if let Some(protocols) = &self.protocols {
            protocols.validate()?;
        }
        
        // Feature-specific validations (conditionally compiled)
        #[cfg(feature = "mqtt")]
        if let Some(mqtt) = &self.mqtt {
            mqtt.validate()?;
        }
        
        #[cfg(feature = "security")]
        if let Some(security) = &self.security {
            security.validate()?;
        }
        
        #[cfg(feature = "history")]
        if let Some(history) = &self.history {
            history.validate()?;
        }
        
        #[cfg(feature = "alarms")]
        if let Some(alarms) = &self.alarms {
            alarms.validate()?;
        }
        
        #[cfg(feature = "web")]
        if let Some(web) = &self.web {
            web.validate()?;
        }
        
        #[cfg(feature = "validation")]
        if let Some(validation) = &self.validation {
            validation.validate()?;
        }
        
        #[cfg(feature = "metrics")]
        if let Some(metrics) = &self.metrics {
            metrics.validate()?;
        }
        
        #[cfg(feature = "realtime")]
        if let Some(realtime) = &self.realtime {
            realtime.validate()?;
        }
        
        Ok(())
    }
    
    /// Validate signal references in blocks
    /// 
    /// Ensures all signal references in block inputs/outputs point to
    /// signals that actually exist in the configuration.
    fn validate_signal_references(&self) -> Result<()> {
        let signal_names: HashSet<&String> = self.signals.iter()
            .map(|s| &s.name)
            .collect();
        
        for block in &self.blocks {
            // Validate input signal references
            for (input_name, signal_name) in &block.inputs {
                if !signal_names.contains(signal_name) {
                    return Err(PlcError::Config(format!(
                        "Block '{}' input '{}' references unknown signal '{}'",
                        block.name, input_name, signal_name
                    )));
                }
            }
            
            // Validate output signal references
            for (output_name, signal_name) in &block.outputs {
                if !signal_names.contains(signal_name) {
                    return Err(PlcError::Config(format!(
                        "Block '{}' output '{}' references unknown signal '{}'",
                        block.name, output_name, signal_name
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Check feature compatibility with runtime features
    /// 
    /// Ensures that the configuration doesn't use features that aren't
    /// compiled into the current PETRA build.
    /// 
    /// # Arguments
    /// 
    /// * `features` - Runtime feature flags from the PETRA build
    /// 
    /// # Errors
    /// 
    /// Returns errors if the configuration requires features that are
    /// not available in the current build.
    pub fn check_feature_compatibility(&self, features: &Features) -> Result<()> {
        // Check protocol features
        if let Some(protocols) = &self.protocols {
            #[cfg(feature = "s7-support")]
            if protocols.s7.is_some() && !features.has_s7() {
                return Err(PlcError::Config(
                    "Configuration uses S7 but s7-support feature is not enabled".to_string()
                ));
            }
            
            #[cfg(feature = "modbus-support")]
            if protocols.modbus.is_some() && !features.has_modbus() {
                return Err(PlcError::Config(
                    "Configuration uses Modbus but modbus-support feature is not enabled".to_string()
                ));
            }
            
            #[cfg(feature = "opcua-support")]
            if protocols.opcua.is_some() && !features.has_opcua() {
                return Err(PlcError::Config(
                    "Configuration uses OPC-UA but opcua-support feature is not enabled".to_string()
                ));
            }
        }
        
        // Check other feature compatibility
        #[cfg(feature = "mqtt")]
        if self.mqtt.is_some() && !features.has_mqtt() {
            return Err(PlcError::Config(
                "Configuration uses MQTT but mqtt feature is not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "security")]
        if self.security.is_some() && !features.has_security() {
            return Err(PlcError::Config(
                "Configuration uses security but security feature is not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "history")]
        if self.history.is_some() && !features.has_history() {
            return Err(PlcError::Config(
                "Configuration uses history but history feature is not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "alarms")]
        if self.alarms.is_some() && !features.has_alarms() {
            return Err(PlcError::Config(
                "Configuration uses alarms but alarms feature is not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "web")]
        if self.web.is_some() && !features.has_web() {
            return Err(PlcError::Config(
                "Configuration uses web but web feature is not enabled".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Generate JSON schema for configuration validation
    /// 
    /// Only available with the "schema-validation" feature. Generates
    /// a JSON schema that can be used for IDE autocomplete and validation.
    #[cfg(feature = "schema-validation")]
    pub fn json_schema() -> Result<String> {
        let schema = schema_for!(Config);
        serde_json::to_string_pretty(&schema)
            .map_err(|e| PlcError::Config(format!(
                "Failed to serialize schema: {}", e
            )))
    }
    
    /// Create an example basic configuration for testing and documentation
    /// 
    /// This creates a minimal but functional configuration that can be used
    /// as a starting point for new installations or for testing purposes.
    pub fn example_basic() -> Result<Self> {
        Ok(Config {
            // Core engine settings
            scan_time_ms: 100,
            max_scan_jitter_ms: 50,
            error_recovery: true,
            max_consecutive_errors: 10,
            restart_delay_ms: 5000,
            
            // Basic signals
            signals: vec![
                SignalConfig {
                    name: "system.heartbeat".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: Some("System heartbeat signal".to_string()),
                    category: Some("System".to_string()),
                    source: Some("Engine".to_string()),
                    tags: vec!["system".to_string(), "status".to_string()],
                    #[cfg(feature = "engineering-types")]
                    units: None,
                    #[cfg(feature = "engineering-types")]
                    min_value: None,
                    #[cfg(feature = "engineering-types")]
                    max_value: None,
                    #[cfg(feature = "quality-codes")]
                    quality_enabled: false,
                    #[cfg(feature = "history")]
                    log_to_history: false,
                    #[cfg(feature = "history")]
                    log_interval_ms: 0,
                    #[cfg(feature = "alarms")]
                    enable_alarms: false,
                    update_frequency_ms: None,
                    #[cfg(feature = "validation")]
                    validation: None,
                    metadata: HashMap::new(),
                },
                SignalConfig {
                    name: "temperature.sensor1".to_string(),
                    signal_type: "float".to_string(),
                    initial: Some(serde_yaml::Value::Number(serde_yaml::Number::from(20.0))),
                    description: Some("Temperature sensor reading".to_string()),
                    category: Some("Temperature".to_string()),
                    source: Some("Sensor".to_string()),
                    tags: vec!["temperature".to_string(), "sensor".to_string()],
                    #[cfg(feature = "engineering-types")]
                    units: Some("°C".to_string()),
                    #[cfg(feature = "engineering-types")]
                    min_value: Some(-50.0),
                    #[cfg(feature = "engineering-types")]
                    max_value: Some(150.0),
                    #[cfg(feature = "quality-codes")]
                    quality_enabled: false,
                    #[cfg(feature = "history")]
                    log_to_history: true,
                    #[cfg(feature = "history")]
                    log_interval_ms: 5000,
                    #[cfg(feature = "alarms")]
                    enable_alarms: true,
                    update_frequency_ms: Some(1000),
                    #[cfg(feature = "validation")]
                    validation: None,
                    metadata: HashMap::new(),
                },
            ],
            
            // Basic blocks
            blocks: vec![
                BlockConfig {
                    name: "heartbeat_generator".to_string(),
                    block_type: "PULSE".to_string(),
                    inputs: HashMap::new(),
                    outputs: {
                        let mut outputs = HashMap::new();
                        outputs.insert("output".to_string(), "system.heartbeat".to_string());
                        outputs
                    },
                    params: {
                        let mut params = HashMap::new();
                        params.insert(
                            "period_ms".to_string(), 
                            serde_yaml::Value::Number(serde_yaml::Number::from(1000))
                        );
                        params
                    },
                    priority: 0,
                    enabled: true,
                    description: Some("Generates system heartbeat signal".to_string()),
                    category: Some("System".to_string()),
                    tags: vec!["system".to_string(), "heartbeat".to_string()],
                    #[cfg(feature = "circuit-breaker")]
                    circuit_breaker: None,
                    #[cfg(feature = "enhanced-monitoring")]
                    enhanced_monitoring: false,
                    metadata: HashMap::new(),
                },
            ],
            
            // No protocols in basic example
            protocols: None,
            
            // Metadata
            version: "1.0".to_string(),
            description: Some("Basic PETRA configuration example".to_string()),
            author: Some("PETRA".to_string()),
            created_at: Some(SystemTime::now()),
            modified_at: Some(SystemTime::now()),
            tags: vec!["example".to_string(), "basic".to_string()],
            metadata: HashMap::new(),
            
            // Feature-specific configs disabled for basic example
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "web")]
            web: None,
            #[cfg(feature = "validation")]
            validation: None,
            #[cfg(feature = "metrics")]
            metrics: None,
            #[cfg(feature = "realtime")]
            realtime: None,
        })
    }
    
    /// Get a summary of configuration contents for logging
    /// 
    /// Returns a concise summary of the configuration suitable for logging
    /// and debugging purposes.
    pub fn summary(&self) -> String {
        format!(
            "PETRA Config v{}: {} signals, {} blocks, scan_time={}ms",
            self.version,
            self.signals.len(),
            self.blocks.len(),
            self.scan_time_ms
        )
    }
}

// ============================================================================
// INDIVIDUAL VALIDATION IMPLEMENTATIONS
// ============================================================================

impl Validatable for SignalConfig {
    fn validate(&self) -> Result<()> {
        // Name validation
        if self.name.is_empty() {
            return Err(PlcError::Config("Signal name cannot be empty".to_string()));
        }
        
        // Check naming convention compliance
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            return Err(PlcError::Config(format!(
                "Signal name '{}' contains invalid characters (only alphanumeric, underscore, and dot allowed)", 
                self.name
            )));
        }
        
        // Type validation
        match self.signal_type.to_lowercase().as_str() {
            "bool" | "int" | "integer" | "float" => {}
            #[cfg(feature = "extended-types")]
            "string" | "binary" | "timestamp" | "array" | "object" => {}
            _ => return Err(PlcError::Config(format!(
                "Unknown signal type: '{}' (valid types: bool, int, float{})",
                self.signal_type,
                #[cfg(feature = "extended-types")]
                ", string, binary, timestamp, array, object",
                #[cfg(not(feature = "extended-types"))]
                ""
            ))),
        }
        
        // Initial value type consistency check
        if let Some(initial) = &self.initial {
            let type_matches = match (&self.signal_type.to_lowercase().as_str(), initial) {
                ("bool", serde_yaml::Value::Bool(_)) => true,
                ("int" | "integer", serde_yaml::Value::Number(n)) => n.is_i64(),
                ("float", serde_yaml::Value::Number(_)) => true,
                #[cfg(feature = "extended-types")]
                ("string", serde_yaml::Value::String(_)) => true,
                _ => false,
            };
            
            if !type_matches {
                return Err(PlcError::Config(format!(
                    "Signal '{}' initial value type does not match signal type '{}'",
                    self.name, self.signal_type
                )));
            }
        }
        
        // Engineering types validation
        #[cfg(feature = "engineering-types")]
        {
            if let (Some(min), Some(max)) = (self.min_value, self.max_value) {
                if min >= max {
                    return Err(PlcError::Config(format!(
                        "Signal '{}' minimum value ({}) must be less than maximum value ({})",
                        self.name, min, max
                    )));
                }
            }
        }
        
        Ok(())
    }
}

impl Validatable for BlockConfig {
    fn validate(&self) -> Result<()> {
        // Name validation
        if self.name.is_empty() {
            return Err(PlcError::Config("Block name cannot be empty".to_string()));
        }
        
        // Type validation
        if self.block_type.is_empty() {
            return Err(PlcError::Config("Block type cannot be empty".to_string()));
        }
        
        // Check for reasonable parameter values
        if let Some(priority) = self.params.get("priority") {
            if let Some(p) = priority.as_i64() {
                if p < -1000 || p > 1000 {
                    warn!("Block '{}' has extreme priority value: {}", self.name, p);
                }
            }
        }
        
        Ok(())
    }
}

impl Validatable for ProtocolConfig {
    fn validate(&self) -> Result<()> {
        let mut protocol_count = 0;
        
        #[cfg(feature = "s7-support")]
        if let Some(s7) = &self.s7 {
            s7.validate()?;
            protocol_count += 1;
        }
        
        #[cfg(feature = "modbus-support")]
        if let Some(modbus) = &self.modbus {
            modbus.validate()?;
            protocol_count += 1;
        }
        
        #[cfg(feature = "opcua-support")]
        if let Some(opcua) = &self.opcua {
            opcua.validate()?;
            protocol_count += 1;
        }
        
        if protocol_count == 0 {
            warn!("No protocols configured - system will only use internal signals");
        }
        
        Ok(())
    }
}

#[cfg(feature = "mqtt")]
impl Validatable for MqttConfig {
    fn validate(&self) -> Result<()> {
        if self.broker.is_empty() {
            return Err(PlcError::Config("MQTT broker address cannot be empty".to_string()));
        }
        
        if self.client_id.is_empty() {
            return Err(PlcError::Config("MQTT client ID cannot be empty".to_string()));
        }
        
        if self.keep_alive == 0 {
            return Err(PlcError::Config("MQTT keep alive must be greater than 0".to_string()));
        }
        
        if self.timeout_ms == 0 {
            return Err(PlcError::Config("MQTT timeout must be greater than 0".to_string()));
        }
        
        // Validate subscriptions
        for sub in &self.subscriptions {
            if sub.topic.is_empty() {
                return Err(PlcError::Config("MQTT subscription topic cannot be empty".to_string()));
            }
            if sub.signal.is_empty() {
                return Err(PlcError::Config("MQTT subscription signal cannot be empty".to_string()));
            }
            if sub.qos > 2 {
                return Err(PlcError::Config(format!(
                    "MQTT QoS must be 0, 1, or 2 (got {})", sub.qos
                )));
            }
        }
        
        // Validate publications
        for pub_config in &self.publications {
            if pub_config.topic.is_empty() {
                return Err(PlcError::Config("MQTT publication topic cannot be empty".to_string()));
            }
            if pub_config.signal.is_empty() {
                return Err(PlcError::Config("MQTT publication signal cannot be empty".to_string()));
            }
            if pub_config.qos > 2 {
                return Err(PlcError::Config(format!(
                    "MQTT QoS must be 0, 1, or 2 (got {})", pub_config.qos
                )));
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "security")]
impl Validatable for SecurityConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        #[cfg(feature = "basic-auth")]
        if let Some(basic_auth) = &self.basic_auth {
            if basic_auth.users.is_empty() {
                return Err(PlcError::Config(
                    "Basic auth enabled but no users configured".to_string()
                ));
            }
        }
        
        #[cfg(feature = "jwt-auth")]
        if let Some(jwt) = &self.jwt {
            if jwt.secret.is_empty() {
                return Err(PlcError::Config("JWT secret cannot be empty".to_string()));
            }
            if jwt.secret.len() < 32 {
                warn!("JWT secret is shorter than recommended 32 characters");
            }
        }
        
        #[cfg(feature = "rbac")]
        if let Some(rbac) = &self.rbac {
            if rbac.roles.is_empty() {
                return Err(PlcError::Config("RBAC enabled but no roles defined".to_string()));
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "history")]
impl Validatable for HistoryConfig {
    fn validate(&self) -> Result<()> {
        if self.batch_size == 0 {
            return Err(PlcError::Config("History batch size must be greater than 0".to_string()));
        }
        
        if self.flush_interval_ms == 0 {
            return Err(PlcError::Config("History flush interval must be greater than 0".to_string()));
        }
        
        // Validate backend-specific configurations
        match self.backend.as_str() {
            "parquet" => {
                // Parquet is file-based, ensure data directory is valid
                if self.data_dir.as_os_str().is_empty() {
                    return Err(PlcError::Config("Data directory cannot be empty for parquet backend".to_string()));
                }
            }
            #[cfg(feature = "clickhouse")]
            "clickhouse" => {
                if let Some(ch) = &self.clickhouse {
                    if ch.url.is_empty() {
                        return Err(PlcError::Config("ClickHouse URL cannot be empty".to_string()));
                    }
                    if ch.database.is_empty() {
                        return Err(PlcError::Config("ClickHouse database cannot be empty".to_string()));
                    }
                } else {
                    return Err(PlcError::Config("ClickHouse backend requires clickhouse configuration".to_string()));
                }
            }
            #[cfg(feature = "s3-storage")]
            "s3" => {
                if let Some(s3) = &self.s3 {
                    if s3.bucket.is_empty() {
                        return Err(PlcError::Config("S3 bucket cannot be empty".to_string()));
                    }
                    if s3.region.is_empty() {
                        return Err(PlcError::Config("S3 region cannot be empty".to_string()));
                    }
                } else {
                    return Err(PlcError::Config("S3 backend requires s3 configuration".to_string()));
                }
            }
            _ => return Err(PlcError::Config(format!(
                "Unknown history backend: '{}'", self.backend
            ))),
        }
        
        Ok(())
    }
}

#[cfg(feature = "alarms")]
impl Validatable for AlarmConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Validate alarm definitions
        let mut alarm_names = HashSet::new();
        for alarm in &self.alarms {
            if !alarm_names.insert(&alarm.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate alarm name: '{}'", alarm.name
                )));
            }
            
            if alarm.condition.is_empty() {
                return Err(PlcError::Config(format!(
                    "Alarm '{}' has empty condition", alarm.name
                )));
            }
            
            // Validate severity
            match alarm.severity.to_lowercase().as_str() {
                "info" | "warning" | "critical" | "fatal" => {}
                _ => return Err(PlcError::Config(format!(
                    "Invalid severity '{}' for alarm '{}'", alarm.severity, alarm.name
                ))),
            }
        }
        
        // Validate channels
        let mut channel_names = HashSet::new();
        for channel in &self.channels {
            if !channel_names.insert(&channel.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate channel name: '{}'", channel.name
                )));
            }
        }
        
        // Ensure referenced channels exist
        for alarm in &self.alarms {
            for channel_ref in &alarm.channels {
                if !channel_names.contains(channel_ref) {
                    return Err(PlcError::Config(format!(
                        "Alarm '{}' references unknown channel '{}'", 
                        alarm.name, channel_ref
                    )));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "web")]
impl Validatable for WebConfig {
    fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(PlcError::Config("Web server port cannot be 0".to_string()));
        }
        
        if self.bind_address.is_empty() {
            return Err(PlcError::Config("Web server bind address cannot be empty".to_string()));
        }
        
        // Validate static directory if specified
        if let Some(static_dir) = &self.static_dir {
            if !static_dir.exists() {
                warn!("Static directory '{}' does not exist", static_dir.display());
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "metrics")]
impl Validatable for MetricsConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if self.port == 0 {
            return Err(PlcError::Config("Metrics port cannot be 0".to_string()));
        }
        
        if self.bind_address.is_empty() {
            return Err(PlcError::Config("Metrics bind address cannot be empty".to_string()));
        }
        
        if self.interval_ms == 0 {
            return Err(PlcError::Config("Metrics interval must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(feature = "realtime")]
impl Validatable for RealtimeConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        if self.priority == 0 || self.priority > 99 {
            return Err(PlcError::Config(
                "Real-time priority must be between 1 and 99".to_string()
            ));
        }
        
        if let Some(affinity) = &self.cpu_affinity {
            if affinity.is_empty() {
                return Err(PlcError::Config("CPU affinity list cannot be empty".to_string()));
            }
            
            // Check for reasonable CPU numbers
            for cpu in affinity {
                if *cpu > 256 {
                    return Err(PlcError::Config(format!(
                        "CPU {} seems unreasonably high", cpu
                    )));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "validation")]
impl Validatable for ValidationConfig {
    fn validate(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }
        
        // Validate rules
        let mut rule_names = HashSet::new();
        for rule in &self.rules {
            if !rule_names.insert(&rule.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate validation rule name: '{}'", rule.name
                )));
            }
            
            if rule.rule_type.is_empty() {
                return Err(PlcError::Config(format!(
                    "Validation rule '{}' has empty type", rule.name
                )));
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "s7-support")]
impl Validatable for S7Config {
    fn validate(&self) -> Result<()> {
        if self.connections.is_empty() {
            return Err(PlcError::Config("S7 configuration has no connections".to_string()));
        }
        
        let mut connection_names = HashSet::new();
        for conn in &self.connections {
            if !connection_names.insert(&conn.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate S7 connection name: '{}'", conn.name
                )));
            }
            
            if conn.ip.is_empty() {
                return Err(PlcError::Config(format!(
                    "S7 connection '{}' has empty IP address", conn.name
                )));
            }
            
            // Validate connection type
            match conn.connection_type.to_uppercase().as_str() {
                "PG" | "OP" | "S7_BASIC" => {}
                _ => return Err(PlcError::Config(format!(
                    "Invalid S7 connection type: '{}'", conn.connection_type
                ))),
            }
            
            // Validate data areas
            for area in &conn.data_areas {
                match area.area.to_uppercase().as_str() {
                    "DB" | "MB" | "IB" | "QB" => {}
                    _ => return Err(PlcError::Config(format!(
                        "Invalid S7 area type: '{}'", area.area
                    ))),
                }
                
                if area.length == 0 {
                    return Err(PlcError::Config(
                        "S7 data area length cannot be 0".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "modbus-support")]
impl Validatable for ModbusConfig {
    fn validate(&self) -> Result<()> {
        if self.connections.is_empty() {
            return Err(PlcError::Config("Modbus configuration has no connections".to_string()));
        }
        
        let mut connection_names = HashSet::new();
        for conn in &self.connections {
            if !connection_names.insert(&conn.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate Modbus connection name: '{}'", conn.name
                )));
            }
            
            if conn.address.is_empty() {
                return Err(PlcError::Config(format!(
                    "Modbus connection '{}' has empty address", conn.name
                )));
            }
            
            // Validate connection type
            match conn.connection_type.to_lowercase().as_str() {
                "tcp" | "rtu" => {}
                _ => return Err(PlcError::Config(format!(
                    "Invalid Modbus connection type: '{}'", conn.connection_type
                ))),
            }
            
            // Validate registers
            for reg in &conn.registers {
                match reg.register_type.to_lowercase().as_str() {
                    "coil" | "discrete" | "holding" | "input" => {}
                    _ => return Err(PlcError::Config(format!(
                        "Invalid Modbus register type: '{}'", reg.register_type
                    ))),
                }
                
                if reg.count == 0 {
                    return Err(PlcError::Config(
                        "Modbus register count cannot be 0".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(feature = "opcua-support")]
impl Validatable for OpcuaConfig {
    fn validate(&self) -> Result<()> {
        if self.endpoint.is_empty() {
            return Err(PlcError::Config("OPC-UA endpoint cannot be empty".to_string()));
        }
        
        // Validate subscriptions
        for sub in &self.subscriptions {
            if sub.node_id.is_empty() {
                return Err(PlcError::Config("OPC-UA node ID cannot be empty".to_string()));
            }
            if sub.signal.is_empty() {
                return Err(PlcError::Config("OPC-UA signal mapping cannot be empty".to_string()));
            }
            if sub.sampling_interval_ms == 0 {
                return Err(PlcError::Config("OPC-UA sampling interval must be greater than 0".to_string()));
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_config_validation() {
        let config = Config::example_basic().unwrap();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_scan_time() {
        let mut config = Config::example_basic().unwrap();
        config.scan_time_ms = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_duplicate_signal_names() {
        let mut config = Config::example_basic().unwrap();
        config.signals.push(config.signals[0].clone());
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_invalid_signal_reference() {
        let mut config = Config::example_basic().unwrap();
        config.blocks[0].outputs.insert(
            "invalid".to_string(),
            "nonexistent.signal".to_string()
        );
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_summary() {
        let config = Config::example_basic().unwrap();
        let summary = config.summary();
        assert!(summary.contains("2 signals"));
        assert!(summary.contains("1 blocks"));
        assert!(summary.contains("100ms"));
    }
}

// ============================================================================
// END OF FILE
// ============================================================================

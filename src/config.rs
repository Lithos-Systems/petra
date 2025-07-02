//! Configuration management with feature-specific organization
//!
//! This module provides YAML-based configuration with automatic feature validation,
//! schema generation, and hot-reload capabilities.

use crate::{PlcError, Result, value::ValueType};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// ============================================================================
// MAIN CONFIGURATION STRUCTURE
// ============================================================================

/// Main PETRA configuration
/// 
/// This structure contains all configuration options organized by feature groups.
/// Only sections corresponding to enabled features will be deserialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Scan cycle time in milliseconds (1-60000ms)
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u32,
    
    /// Signal definitions
    #[serde(default)]
    pub signals: Vec<SignalConfig>,
    
    /// Logic block definitions  
    #[serde(default)]
    pub blocks: Vec<BlockConfig>,
    
    // ========================================================================
    // PROTOCOL CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Protocol configurations
    #[cfg(any(
        feature = "mqtt",
        feature = "s7-support", 
        feature = "modbus-support",
        feature = "opcua-support"
    ))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<ProtocolsConfig>,
    
    /// MQTT configuration (legacy field for backward compatibility)
    #[cfg(feature = "mqtt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    
    /// S7 PLC configuration (legacy field for backward compatibility)
    #[cfg(feature = "s7-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s7: Option<S7Config>,
    
    // ========================================================================
    // STORAGE CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Storage and persistence configuration
    #[cfg(any(feature = "history", feature = "advanced-storage"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<StorageConfig>,
    
    // ========================================================================
    // SECURITY CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Security and authentication configuration
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    
    // ========================================================================
    // MONITORING & METRICS CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Monitoring configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitoring: Option<MonitoringConfig>,
    
    /// Metrics and telemetry configuration
    #[cfg(feature = "metrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricsConfig>,
    
    // ========================================================================
    // VALIDATION CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Validation rules and constraints
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
    
    // ========================================================================
    // ALARM CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Alarm management configuration
    #[cfg(feature = "alarms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<AlarmsConfig>,
    
    // ========================================================================
    // WEB & HEALTH CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Web interface configuration
    #[cfg(feature = "web")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<WebConfig>,
    
    /// Health monitoring configuration
    #[cfg(feature = "health")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<HealthConfig>,
    
    // ========================================================================
    // DEVELOPMENT CONFIGURATIONS (feature-gated)
    // ========================================================================
    
    /// Development and testing configuration
    #[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub development: Option<DevelopmentConfig>,
    
    // ========================================================================
    // METADATA & EXTENSIONS
    // ========================================================================
    
    /// Configuration metadata and custom fields
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
    
    /// Configuration signature (for security validation)
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<SignatureConfig>,
}

// ============================================================================
// CORE CONFIGURATION STRUCTURES
// ============================================================================

/// Signal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalConfig {
    /// Signal name (must be unique)
    pub name: String,
    
    /// Signal data type
    #[serde(rename = "type")]
    pub signal_type: ValueType,
    
    /// Initial value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<serde_yaml::Value>,
    
    /// Signal description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Engineering units (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    
    /// Quality code configuration (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    #[serde(default)]
    pub quality_enabled: bool,
    
    /// Validation rules for this signal
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<SignalValidationConfig>,
    
    /// Signal tags for grouping and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Custom metadata for this signal
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Logic block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockConfig {
    /// Block name (must be unique)
    pub name: String,
    
    /// Block type (e.g., "AND", "OR", "TON", "PID")
    #[serde(rename = "type")]
    pub block_type: String,
    
    /// Input signal mappings
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    /// Output signal mappings
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Block parameters
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    
    /// Block description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Block tags for grouping and filtering
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Error handling configuration
    #[cfg(feature = "enhanced-errors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handling: Option<ErrorHandlingConfig>,
    
    /// Circuit breaker configuration
    #[cfg(feature = "circuit-breaker")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

// ============================================================================
// PROTOCOL CONFIGURATIONS
// ============================================================================

/// Unified protocols configuration
#[cfg(any(
    feature = "mqtt",
    feature = "s7-support",
    feature = "modbus-support", 
    feature = "opcua-support"
))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ProtocolsConfig {
    #[cfg(feature = "mqtt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    
    #[cfg(feature = "s7-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s7: Option<S7Config>,
    
    #[cfg(feature = "modbus-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modbus: Option<ModbusConfig>,
    
    #[cfg(feature = "opcua-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcua: Option<OpcUaConfig>,
}

#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttConfig {
    pub broker_url: String,
    pub client_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    
    #[serde(default)]
    pub keep_alive: u64,
    
    #[serde(default)]
    pub subscriptions: Vec<MqttSubscription>,
    
    #[serde(default)]
    pub publications: Vec<MqttPublication>,
    
    /// MQTT persistence configuration
    #[cfg(feature = "mqtt-persistence")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence: Option<MqttPersistenceConfig>,
    
    /// TLS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttSubscription {
    pub topic: String,
    pub qos: u8,
    pub signal: String,
}

#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttPublication {
    pub topic: String,
    pub qos: u8,
    pub signal: String,
    
    #[serde(default)]
    pub retain: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_ms: Option<u64>,
}

#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S7Config {
    pub ip: String,
    pub rack: u32,
    pub slot: u32,
    
    #[serde(default = "default_s7_port")]
    pub port: u16,
    
    #[serde(default)]
    pub connections: Vec<S7Connection>,
}

#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S7Connection {
    pub name: String,
    pub area: String,  // "DB", "M", "I", "Q"
    pub db_number: Option<u32>,
    pub start: u32,
    pub size: u32,
    pub signal: String,
}

#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ModbusConfig {
    pub connection_type: String, // "tcp" or "rtu"
    pub address: String,         // IP:port for TCP, device path for RTU
    pub slave_id: u8,
    
    #[serde(default)]
    pub registers: Vec<ModbusRegister>,
}

#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ModbusRegister {
    pub name: String,
    pub register_type: String, // "coil", "discrete", "input", "holding"
    pub address: u16,
    pub count: u16,
    pub signal: String,
}

#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OpcUaConfig {
    pub endpoint: String,
    
    #[serde(default = "default_opcua_port")]
    pub port: u16,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_policy: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_mode: Option<String>,
    
    #[serde(default)]
    pub nodes: Vec<OpcUaNode>,
}

#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OpcUaNode {
    pub name: String,
    pub node_id: String,
    pub signal: String,
    
    #[serde(default)]
    pub sampling_interval_ms: u64,
}

// ============================================================================
// STORAGE CONFIGURATIONS
// ============================================================================

#[cfg(any(feature = "history", feature = "advanced-storage"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct StorageConfig {
    /// Basic historical storage configuration
    #[cfg(feature = "history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
    
    /// Advanced storage backends
    #[cfg(feature = "advanced-storage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced: Option<AdvancedStorageConfig>,
    
    /// Compression configuration
    #[cfg(feature = "compression")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionConfig>,
    
    /// Write-Ahead Log configuration
    #[cfg(feature = "wal")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wal: Option<WalConfig>,
}

#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HistoryConfig {
    pub enabled: bool,
    
    #[serde(default = "default_history_path")]
    pub path: PathBuf,
    
    #[serde(default = "default_history_interval")]
    pub interval_ms: u64,
    
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    
    #[serde(default)]
    pub signals: Vec<String>, // Signals to log, empty = all
}

// ============================================================================
// MONITORING CONFIGURATIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MonitoringConfig {
    /// Monitoring level
    #[serde(default = "default_monitoring_level")]
    pub level: MonitoringLevel,
    
    /// Jitter monitoring threshold in microseconds
    #[serde(default = "default_jitter_threshold")]
    pub jitter_threshold_us: u64,
    
    /// Performance statistics collection
    #[serde(default)]
    pub collect_stats: bool,
    
    /// Enhanced monitoring options
    #[cfg(feature = "enhanced-monitoring")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhanced: Option<EnhancedMonitoringConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum MonitoringLevel {
    None,
    Basic,
    Standard,
    Enhanced,
}

#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EnhancedMonitoringConfig {
    pub block_timing: bool,
    pub memory_tracking: bool,
    pub signal_change_tracking: bool,
    pub detailed_stats: bool,
    
    #[serde(default = "default_stats_interval")]
    pub stats_interval_ms: u64,
}

#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MetricsConfig {
    pub enabled: bool,
    
    #[serde(default = "default_metrics_port")]
    pub port: u16,
    
    #[serde(default = "default_metrics_path")]
    pub path: String,
    
    #[serde(default)]
    pub custom_metrics: Vec<CustomMetricConfig>,
}

#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CustomMetricConfig {
    pub name: String,
    pub metric_type: String, // "counter", "gauge", "histogram"
    pub description: String,
    pub signal: Option<String>,
}

// ============================================================================
// SECURITY CONFIGURATIONS
// ============================================================================

#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SecurityConfig {
    pub enabled: bool,
    
    /// Authentication configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AuthenticationConfig>,
    
    /// TLS/SSL configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    
    /// Role-based access control
    #[cfg(feature = "rbac")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rbac: Option<RbacConfig>,
    
    /// Audit logging configuration
    #[cfg(feature = "audit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
    
    /// Configuration signing
    #[cfg(feature = "signing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing: Option<SigningConfig>,
}

#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AuthenticationConfig {
    /// Authentication method
    pub method: AuthMethod,
    
    /// Session timeout in seconds
    #[serde(default = "default_session_timeout")]
    pub session_timeout_sec: u64,
    
    /// Basic auth configuration
    #[cfg(feature = "basic-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic: Option<BasicAuthConfig>,
    
    /// JWT configuration
    #[cfg(feature = "jwt-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<JwtConfig>,
}

#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum AuthMethod {
    #[cfg(feature = "basic-auth")]
    Basic,
    #[cfg(feature = "jwt-auth")]
    Jwt,
    None,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BasicAuthConfig {
    pub users: Vec<UserConfig>,
}

#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct UserConfig {
    pub username: String,
    pub password_hash: String,
    
    #[cfg(feature = "rbac")]
    #[serde(default)]
    pub roles: Vec<String>,
}

#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct JwtConfig {
    pub secret: String,
    
    #[serde(default = "default_jwt_expiry")]
    pub expiry_hours: u64,
    
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct TlsConfig {
    pub enabled: bool,
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_path: Option<PathBuf>,
    
    #[serde(default)]
    pub verify_client: bool,
}

// ============================================================================
// VALIDATION CONFIGURATIONS
// ============================================================================

#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ValidationConfig {
    pub enabled: bool,
    
    #[serde(default)]
    pub global_rules: Vec<ValidationRule>,
    
    #[serde(default)]
    pub signal_rules: HashMap<String, Vec<ValidationRule>>,
    
    /// Validation behavior on failure
    #[serde(default = "default_validation_behavior")]
    pub on_failure: ValidationBehavior,
}

#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalValidationConfig {
    #[serde(default)]
    pub rules: Vec<ValidationRule>,
    
    #[serde(default)]
    pub required: bool,
}

#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ValidationRule {
    pub rule_type: String, // "range", "regex", "schema", "custom"
    pub params: HashMap<String, serde_yaml::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum ValidationBehavior {
    Reject,  // Reject invalid values
    Warn,    // Log warning but accept
    Clamp,   // Clamp to valid range
    Default, // Use default value
}

// ============================================================================
// ALARM CONFIGURATIONS
// ============================================================================

#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmsConfig {
    pub enabled: bool,
    
    #[serde(default)]
    pub alarm_definitions: Vec<AlarmDefinition>,
    
    /// Notification channels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications: Option<NotificationConfig>,
    
    /// Alarm escalation rules
    #[serde(default)]
    pub escalation: Vec<EscalationRule>,
}

#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmDefinition {
    pub name: String,
    pub signal: String,
    pub condition: String,     // "high", "low", "change", "timeout"
    pub threshold: Option<f64>,
    pub priority: AlarmPriority,
    
    #[serde(default)]
    pub enabled: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub enum AlarmPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct NotificationConfig {
    #[cfg(feature = "email")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailConfig>,
    
    #[cfg(feature = "twilio")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twilio: Option<TwilioConfig>,
}

#[cfg(feature = "email")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    
    #[serde(default)]
    pub recipients: Vec<String>,
    
    #[serde(default)]
    pub use_tls: bool,
}

#[cfg(feature = "twilio")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    
    #[serde(default)]
    pub sms_recipients: Vec<String>,
    
    #[serde(default)]
    pub voice_recipients: Vec<String>,
}

// ============================================================================
// WEB & HEALTH CONFIGURATIONS
// ============================================================================

#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WebConfig {
    pub enabled: bool,
    
    #[serde(default = "default_web_port")]
    pub port: u16,
    
    #[serde(default = "default_web_bind")]
    pub bind_address: String,
    
    /// Static file serving
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_files: Option<StaticFilesConfig>,
    
    /// API configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiConfig>,
    
    /// CORS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,
}

#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct StaticFilesConfig {
    pub enabled: bool,
    pub path: PathBuf,
    pub route: String,
}

#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ApiConfig {
    pub enabled: bool,
    pub base_path: String,
    
    #[serde(default)]
    pub rate_limiting: bool,
    
    #[serde(default = "default_rate_limit")]
    pub requests_per_minute: u32,
}

#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CorsConfig {
    pub enabled: bool,
    
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    
    #[serde(default)]
    pub allowed_methods: Vec<String>,
    
    #[serde(default)]
    pub allowed_headers: Vec<String>,
}

#[cfg(feature = "health")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HealthConfig {
    pub enabled: bool,
    
    #[serde(default = "default_health_interval")]
    pub check_interval_ms: u64,
    
    #[serde(default)]
    pub checks: Vec<HealthCheckConfig>,
    
    /// Health history retention
    #[cfg(feature = "health-history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HealthHistoryConfig>,
}

#[cfg(feature = "health")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HealthCheckConfig {
    pub name: String,
    pub check_type: String, // "memory", "cpu", "disk", "signal", "custom"
    pub params: HashMap<String, serde_yaml::Value>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

// ============================================================================
// DEVELOPMENT CONFIGURATIONS
// ============================================================================

#[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DevelopmentConfig {
    #[cfg(feature = "examples")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<ExamplesConfig>,
    
    #[cfg(feature = "burn-in")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burn_in: Option<BurnInConfig>,
    
    #[cfg(feature = "profiling")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiling: Option<ProfilingConfig>,
    
    #[cfg(feature = "hot-swap")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hot_reload: Option<HotReloadConfig>,
}

// ============================================================================
// ADDITIONAL SUPPORTING STRUCTURES
// ============================================================================

#[cfg(feature = "enhanced-errors")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ErrorHandlingConfig {
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub fallback_value: Option<serde_yaml::Value>,
    pub escalate_on_failure: bool,
}

#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CircuitBreakerConfig {
    pub max_failures: u32,
    pub reset_timeout_ms: u64,
    pub half_open_max_calls: u32,
}

#[cfg(feature = "security")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignatureConfig {
    pub algorithm: String,
    pub public_key: String,
    pub signature: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub verify_enabled: bool,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)?;
        
        #[cfg(feature = "security")]
        {
            // Check for embedded signature
            if contents.starts_with("---BEGIN SIGNATURE---") {
                return Self::from_signed_file(path);
            }
        }
        
        let config: Config = serde_yaml::from_str(&contents)
            .map_err(|e| PlcError::Config(format!("YAML parse error: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml_str = serde_yaml::to_string(self)?;
        std::fs::write(&path, yaml_str)?;
        Ok(())
    }
    
    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        // Core validation
        self.validate_core()?;
        
        // Feature-specific validation
        #[cfg(any(
            feature = "mqtt",
            feature = "s7-support",
            feature = "modbus-support", 
            feature = "opcua-support"
        ))]
        self.validate_protocols()?;
        
        #[cfg(any(feature = "history", feature = "advanced-storage"))]
        self.validate_storage()?;
        
        #[cfg(feature = "security")]
        self.validate_security()?;
        
        #[cfg(feature = "validation")]
        self.validate_validation_config()?;
        
        #[cfg(feature = "alarms")]
        self.validate_alarms()?;
        
        #[cfg(feature = "web")]
        self.validate_web()?;
        
        Ok(())
    }
    
    fn validate_core(&self) -> Result<()> {
        // Validate scan time
        if self.scan_time_ms == 0 || self.scan_time_ms > 60000 {
            return Err(PlcError::Config(
                "Scan time must be between 1 and 60000 milliseconds".into()
            ));
        }
        
        // Validate signal names are unique
        let mut signal_names = std::collections::HashSet::new();
        for signal in &self.signals {
            if !signal_names.insert(&signal.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate signal name: {}", signal.name
                )));
            }
            signal.validate()?;
        }
        
        // Validate block names are unique
        let mut block_names = std::collections::HashSet::new();
        for block in &self.blocks {
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate block name: {}", block.name
                )));
            }
            block.validate(&signal_names)?;
        }
        
        Ok(())
    }
    
    #[cfg(feature = "json-schema")]
    /// Generate JSON schema for the configuration
    pub fn json_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Config)
    }
    
    #[cfg(feature = "security")]
    /// Load a signed configuration file
    pub fn from_signed_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        use crate::security::verify_signature;
        
        let contents = std::fs::read_to_string(&path)?;
        
        // Parse signed configuration format
        let parts: Vec<&str> = contents.split("---BEGIN CONFIG---").collect();
        if parts.len() != 2 {
            return Err(PlcError::Config("Invalid signed config format".into()));
        }
        
        let sig_section = parts[0]
            .trim_start_matches("---BEGIN SIGNATURE---")
            .trim_end_matches('\n')
            .trim();
        
        let config_section = parts[1]
            .trim_end_matches("---END CONFIG---")
            .trim();
        
        // Parse and verify signature
        let signature: SignatureConfig = serde_yaml::from_str(sig_section)?;
        if signature.verify_enabled {
            verify_signature(config_section.as_bytes(), &signature)?;
        }
        
        // Parse configuration
        let mut config: Config = serde_yaml::from_str(config_section)?;
        config.signature = Some(signature);
        
        config.validate()?;
        Ok(config)
    }
    
    /// Get feature summary for this configuration
    pub fn feature_summary(&self) -> String {
        let mut features = Vec::new();
        
        #[cfg(feature = "mqtt")]
        if self.mqtt.is_some() || 
           (self.protocols.as_ref().map_or(false, |p| p.mqtt.is_some())) {
            features.push("MQTT");
        }
        
        #[cfg(feature = "s7-support")]
        if self.s7.is_some() || 
           (self.protocols.as_ref().map_or(false, |p| p.s7.is_some())) {
            features.push("S7");
        }
        
        #[cfg(any(feature = "history", feature = "advanced-storage"))]
        if self.storage.is_some() {
            features.push("Storage");
        }
        
        #[cfg(feature = "security")]
        if self.security.as_ref().map_or(false, |s| s.enabled) {
            features.push("Security");
        }
        
        #[cfg(feature = "alarms")]
        if self.alarms.as_ref().map_or(false, |a| a.enabled) {
            features.push("Alarms");
        }
        
        #[cfg(feature = "web")]
        if self.web.as_ref().map_or(false, |w| w.enabled) {
            features.push("Web");
        }
        
        if features.is_empty() {
            "Basic Configuration".to_string()
        } else {
            features.join(" + ")
        }
    }
}

impl SignalConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(PlcError::Config("Signal name cannot be empty".into()));
        }
        
        // Validate signal name format (alphanumeric + underscore)
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlcError::Config(format!(
                "Invalid signal name '{}': only alphanumeric characters and underscores allowed",
                self.name
            )));
        }
        
        Ok(())
    }
}

impl BlockConfig {
    fn validate(&self, signal_names: &std::collections::HashSet<&String>) -> Result<()> {
        if self.name.is_empty() {
            return Err(PlcError::Config("Block name cannot be empty".into()));
        }
        
        if self.block_type.is_empty() {
            return Err(PlcError::Config(format!(
                "Block '{}' must have a type", self.name
            )));
        }
        
        // Validate that all referenced signals exist
        for (input_name, signal_name) in &self.inputs {
            if !signal_names.contains(signal_name) {
                return Err(PlcError::Config(format!(
                    "Block '{}' input '{}' references unknown signal '{}'",
                    self.name, input_name, signal_name
                )));
            }
        }
        
        for (output_name, signal_name) in &self.outputs {
            if !signal_names.contains(signal_name) {
                return Err(PlcError::Config(format!(
                    "Block '{}' output '{}' references unknown signal '{}'", 
                    self.name, output_name, signal_name
                )));
            }
        }
        
        Ok(())
    }
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================

fn default_scan_time() -> u32 { 100 }
fn default_monitoring_level() -> MonitoringLevel { MonitoringLevel::Standard }
fn default_jitter_threshold() -> u64 { 1000 }
fn default_stats_interval() -> u64 { 5000 }

#[cfg(feature = "mqtt")]
fn default_mqtt_port() -> u16 { 1883 }

#[cfg(feature = "s7-support")]
fn default_s7_port() -> u16 { 102 }

#[cfg(feature = "opcua-support")]
fn default_opcua_port() -> u16 { 4840 }

#[cfg(feature = "history")]
fn default_history_path() -> PathBuf { PathBuf::from("data/history") }

#[cfg(feature = "history")]
fn default_history_interval() -> u64 { 1000 }

#[cfg(feature = "history")]
fn default_retention_days() -> u32 { 30 }

#[cfg(feature = "metrics")]
fn default_metrics_port() -> u16 { 9090 }

#[cfg(feature = "metrics")]
fn default_metrics_path() -> String { "/metrics".to_string() }

#[cfg(feature = "security")]
fn default_session_timeout() -> u64 { 3600 }

#[cfg(feature = "jwt-auth")]
fn default_jwt_expiry() -> u64 { 24 }

#[cfg(feature = "validation")]
fn default_validation_behavior() -> ValidationBehavior { ValidationBehavior::Warn }

#[cfg(feature = "web")]
fn default_web_port() -> u16 { 8080 }

#[cfg(feature = "web")]
fn default_web_bind() -> String { "0.0.0.0".to_string() }

#[cfg(feature = "web")]
fn default_rate_limit() -> u32 { 60 }

#[cfg(feature = "health")]
fn default_health_interval() -> u64 { 5000 }

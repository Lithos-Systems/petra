//! Configuration management for PETRA
//! 
//! This module provides comprehensive configuration support with:
//! - YAML/JSON/TOML parsing
//! - Feature-specific configuration sections
//! - Validation and defaults
//! - Hot-reload support

use crate::error::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

/// Main configuration structure for PETRA
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct Config {
    /// General system configuration
    #[serde(default)]
    pub system: SystemConfig,
    
    /// Engine configuration
    #[serde(default)]
    pub engine: EngineConfig,
    
    /// Block configurations
    #[serde(default)]
    pub blocks: Vec<BlockConfig>,
    
    /// Signal mappings
    #[serde(default)]
    pub signals: HashMap<String, SignalConfig>,
    
    /// Storage configuration
    #[cfg(feature = "history")]
    #[serde(default)]
    pub storage: StorageConfig,
    
    /// Protocol configurations
    #[serde(default)]
    pub protocols: ProtocolsConfig,
    
    /// Security configuration
    #[cfg(feature = "security")]
    #[serde(default)]
    pub security: SecurityConfig,
    
    /// Alarm configuration
    #[cfg(feature = "alarms")]
    #[serde(default)]
    pub alarms: AlarmsConfig,
    
    /// Web interface configuration
    #[cfg(feature = "web")]
    #[serde(default)]
    pub web: WebConfig,
    
    /// Metrics configuration
    #[cfg(feature = "metrics")]
    #[serde(default)]
    pub metrics: MetricsConfig,
}

/// System-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SystemConfig {
    /// System name
    #[serde(default = "default_system_name")]
    pub name: String,
    
    /// System description
    #[serde(default)]
    pub description: String,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Enable hot-reload
    #[cfg(feature = "hot-swap")]
    #[serde(default)]
    pub hot_reload: bool,
    
    /// Working directory
    #[serde(default = "default_work_dir")]
    pub work_dir: PathBuf,
    
    /// Time zone
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

/// Engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EngineConfig {
    /// Scan time in milliseconds
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    
    /// Maximum jitter percentage
    #[serde(default = "default_max_jitter")]
    pub max_jitter_percent: f64,
    
    /// Enable performance monitoring
    #[serde(default = "default_true")]
    pub performance_monitoring: bool,
    
    /// Thread pool size (None = auto)
    #[serde(default)]
    pub thread_pool_size: Option<usize>,
    
    /// Priority levels
    #[serde(default)]
    pub priorities: PriorityConfig,
}

/// Priority configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct PriorityConfig {
    /// Enable priority scheduling
    #[serde(default)]
    pub enabled: bool,
    
    /// High priority scan divisor
    #[serde(default = "default_high_priority")]
    pub high: u32,
    
    /// Medium priority scan divisor
    #[serde(default = "default_medium_priority")]
    pub medium: u32,
    
    /// Low priority scan divisor
    #[serde(default = "default_low_priority")]
    pub low: u32,
}

/// Individual block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockConfig {
    /// Block name (unique identifier)
    pub name: String,
    
    /// Block type (AND, OR, TIMER_ON, etc.)
    pub block_type: String,
    
    /// Block description
    #[serde(default)]
    pub description: String,
    
    /// Input signal names
    #[serde(default)]
    pub inputs: Vec<String>,
    
    /// Output signal names
    #[serde(default)]
    pub outputs: Vec<String>,
    
    /// Block-specific parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    
    /// Execution priority
    #[serde(default)]
    pub priority: Priority,
    
    /// Enable/disable block
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Execution priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

/// Signal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalConfig {
    /// Signal description
    #[serde(default)]
    pub description: String,
    
    /// Initial value
    #[serde(default)]
    pub initial_value: serde_json::Value,
    
    /// Persist signal value
    #[serde(default)]
    pub persistent: bool,
    
    /// Signal metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Storage configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct StorageConfig {
    /// History storage backend
    #[serde(default)]
    pub backend: StorageBackend,
    
    /// Base path for file storage
    #[serde(default = "default_storage_path")]
    pub path: PathBuf,
    
    /// Retention policy
    #[serde(default)]
    pub retention: RetentionPolicy,
    
    /// Compression settings
    #[cfg(feature = "compression")]
    #[serde(default)]
    pub compression: CompressionConfig,
    
    /// Write-ahead logging
    #[cfg(feature = "wal")]
    pub wal: Option<WalConfig>,
    
    /// ClickHouse configuration
    #[cfg(feature = "clickhouse")]
    pub clickhouse: Option<ClickHouseConfig>,
    
    /// S3 configuration
    #[cfg(feature = "s3")]
    pub s3: Option<S3Config>,
}

/// Storage backend types
#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum StorageBackend {
    Parquet,
    #[cfg(feature = "clickhouse")]
    ClickHouse,
    #[cfg(feature = "s3")]
    S3,
}

#[cfg(feature = "history")]
impl Default for StorageBackend {
    fn default() -> Self {
        StorageBackend::Parquet
    }
}

/// Data retention policy
#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct RetentionPolicy {
    /// Days to retain high-resolution data
    #[serde(default = "default_retention_days")]
    pub days: u32,
    
    /// Enable downsampling
    #[serde(default)]
    pub downsampling: bool,
    
    /// Downsampling intervals
    #[serde(default)]
    pub intervals: Vec<DownsampleInterval>,
}

#[cfg(feature = "history")]
impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            days: default_retention_days(),
            downsampling: false,
            intervals: vec![],
        }
    }
}

/// Downsampling interval configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct DownsampleInterval {
    /// After this many days
    pub after_days: u32,
    
    /// Downsample to this resolution (seconds)
    pub resolution_seconds: u32,
}

/// Compression configuration
#[cfg(feature = "compression")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CompressionConfig {
    /// Compression algorithm
    #[serde(default)]
    pub algorithm: CompressionAlgorithm,
    
    /// Compression level (1-9)
    #[serde(default = "default_compression_level")]
    pub level: u32,
}

#[cfg(feature = "compression")]
impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::default(),
            level: default_compression_level(),
        }
    }
}

/// Compression algorithms
#[cfg(feature = "compression")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum CompressionAlgorithm {
    Zstd,
    Lz4,
    None,
}

#[cfg(feature = "compression")]
impl Default for CompressionAlgorithm {
    fn default() -> Self {
        CompressionAlgorithm::Zstd
    }
}

/// Write-Ahead Logging configuration
#[cfg(feature = "wal")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WalConfig {
    /// Enable WAL
    #[serde(default)]
    pub enabled: bool,
    
    /// WAL directory path
    #[serde(default = "default_wal_path")]
    pub path: PathBuf,
    
    /// Maximum WAL file size in bytes (default: 100MB)
    #[serde(default = "default_wal_max_size")]
    pub max_file_size: u64,
    
    /// Number of WAL files to keep
    #[serde(default = "default_wal_retention")]
    pub retention_count: usize,
    
    /// Sync mode for durability
    #[serde(default)]
    pub sync_mode: WalSyncMode,
    
    /// Compression for WAL files
    #[serde(default)]
    pub compression: Option<CompressionType>,
}

/// WAL sync modes
#[cfg(feature = "wal")]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum WalSyncMode {
    /// Sync after every write (slowest, most durable)
    Always,
    
    /// Sync periodically (default)
    #[default]
    Periodic,
    
    /// Never sync (fastest, least durable)
    Never,
}

/// Compression types for WAL
#[cfg(feature = "wal")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CompressionType {
    Zstd,
    Lz4,
    None,
}

/// Protocol configurations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ProtocolsConfig {
    /// MQTT configuration
    #[cfg(feature = "mqtt")]
    pub mqtt: Option<MqttConfig>,
    
    /// Modbus configuration
    #[cfg(feature = "modbus-support")]
    pub modbus: Option<ModbusConfig>,
    
    /// S7 configuration
    #[cfg(feature = "s7-support")]
    pub s7: Option<S7Config>,
    
    /// OPC-UA configuration
    #[cfg(feature = "opcua-support")]
    pub opcua: Option<OpcUaConfig>,
}

/// MQTT protocol configuration
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttConfig {
    /// MQTT broker URL
    pub broker: String,
    
    /// Client ID
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,
    
    /// Username
    #[serde(default)]
    pub username: Option<String>,
    
    /// Password
    #[serde(default)]
    pub password: Option<String>,
    
    /// Keep alive interval (seconds)
    #[serde(default = "default_mqtt_keepalive")]
    pub keepalive_secs: u16,
    
    /// Clean session
    #[serde(default = "default_true")]
    pub clean_session: bool,
    
    /// QoS level
    #[serde(default)]
    pub qos: MqttQos,
    
    /// Topic prefix
    #[serde(default = "default_mqtt_topic_prefix")]
    pub topic_prefix: String,
    
    /// TLS configuration
    #[serde(default)]
    pub tls: Option<MqttTlsConfig>,
}

/// MQTT QoS levels
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[repr(u8)]
pub enum MqttQos {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

#[cfg(feature = "mqtt")]
impl Default for MqttQos {
    fn default() -> Self {
        MqttQos::AtLeastOnce
    }
}

/// MQTT TLS configuration
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttTlsConfig {
    /// CA certificate path
    pub ca_cert: Option<PathBuf>,
    
    /// Client certificate path
    pub client_cert: Option<PathBuf>,
    
    /// Client key path
    pub client_key: Option<PathBuf>,
}

/// Security configuration
#[cfg(feature = "security")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SecurityConfig {
    /// Enable security features
    #[serde(default)]
    pub enabled: bool,
    
    /// Basic authentication
    #[cfg(feature = "basic-auth")]
    pub basic_auth: Option<BasicAuthConfig>,
    
    /// JWT authentication
    #[cfg(feature = "jwt-auth")]
    pub jwt: Option<JwtConfig>,
    
    /// RBAC configuration
    #[cfg(feature = "rbac")]
    pub rbac: Option<RbacConfig>,
    
    /// Audit logging
    #[cfg(feature = "audit")]
    pub audit: Option<AuditConfig>,
}

/// Basic authentication configuration
#[cfg(all(feature = "security", feature = "basic-auth"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BasicAuthConfig {
    /// Authentication realm
    #[serde(default = "default_auth_realm")]
    pub realm: String,
    
    /// Enable basic auth
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// User database path
    #[serde(default)]
    pub users_file: Option<PathBuf>,
}

/// Alarm configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmsConfig {
    /// Enable alarm system
    #[serde(default)]
    pub enabled: bool,
    
    /// Alarm definitions
    #[serde(default)]
    pub alarms: Vec<AlarmDefinition>,
    
    /// Notification settings
    #[serde(default)]
    pub notifications: NotificationConfig,
}

/// Individual alarm definition
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmDefinition {
    /// Alarm ID
    pub id: String,
    
    /// Description
    pub description: String,
    
    /// Trigger condition (signal name)
    pub trigger: String,
    
    /// Severity level
    #[serde(default)]
    pub severity: AlarmSeverity,
    
    /// Auto-acknowledge
    #[serde(default)]
    pub auto_ack: bool,
}

/// Alarm severity levels
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum AlarmSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[cfg(feature = "alarms")]
impl Default for AlarmSeverity {
    fn default() -> Self {
        AlarmSeverity::Warning
    }
}

/// Notification configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct NotificationConfig {
    /// Email notifications
    #[cfg(feature = "email")]
    pub email: Option<EmailNotificationConfig>,
    
    /// SMS notifications
    #[cfg(feature = "twilio")]
    pub sms: Option<SmsNotificationConfig>,
}

/// Web server configuration
#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WebConfig {
    /// Listen address
    #[serde(default = "default_web_listen")]
    pub listen: String,
    
    /// Enable web interface
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// CORS settings
    #[serde(default)]
    pub cors: CorsConfig,
    
    /// TLS configuration
    #[serde(default)]
    pub tls: Option<WebTlsConfig>,
}

#[cfg(feature = "web")]
impl Default for WebConfig {
    fn default() -> Self {
        Self {
            listen: default_web_listen(),
            enabled: true,
            cors: CorsConfig::default(),
            tls: None,
        }
    }
}

/// CORS configuration
#[cfg(feature = "web")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CorsConfig {
    /// Allow all origins
    #[serde(default)]
    pub allow_all: bool,
    
    /// Allowed origins
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

// ============================================================================
// CONFIGURATION IMPLEMENTATION
// ============================================================================

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| PlcError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config = match path.extension().and_then(|s| s.to_str()) {
            Some("yaml") | Some("yml") => serde_yaml::from_str(&content)
                .map_err(|e| PlcError::Config(format!("YAML parse error: {}", e)))?,
            Some("json") => serde_json::from_str(&content)
                .map_err(|e| PlcError::Config(format!("JSON parse error: {}", e)))?,
            Some("toml") => toml::from_str(&content)
                .map_err(|e| PlcError::Config(format!("TOML parse error: {}", e)))?,
            _ => return Err(PlcError::Config("Unsupported config format".to_string())),
        };
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate engine settings
        if self.engine.scan_time_ms == 0 {
            return Err(PlcError::Config("Scan time must be greater than 0".to_string()));
        }
        
        if self.engine.max_jitter_percent > 100.0 || self.engine.max_jitter_percent < 0.0 {
            return Err(PlcError::Config("Max jitter must be between 0 and 100".to_string()));
        }
        
        // Validate blocks
        let mut block_names = std::collections::HashSet::new();
        for block in &self.blocks {
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!("Duplicate block name: {}", block.name)));
            }
        }
        
        // Validate signals referenced by blocks exist
        for block in &self.blocks {
            for input in &block.inputs {
                if !self.signals.contains_key(input) && !block_names.contains(&input) {
                    return Err(PlcError::Config(format!(
                        "Block '{}' references unknown input signal: {}", 
                        block.name, input
                    )));
                }
            }
        }
        
        Ok(())
    }
    
    /// Merge with another configuration (other takes precedence)
    pub fn merge(mut self, other: Config) -> Self {
        self.system = other.system;
        self.engine = other.engine;
        self.blocks = other.blocks;
        self.signals = other.signals;
        
        #[cfg(feature = "history")]
        { self.storage = other.storage; }
        
        self.protocols = other.protocols;
        
        #[cfg(feature = "security")]
        { self.security = other.security; }
        
        #[cfg(feature = "alarms")]
        { self.alarms = other.alarms; }
        
        #[cfg(feature = "web")]
        { self.web = other.web; }
        
        #[cfg(feature = "metrics")]
        { self.metrics = other.metrics; }
        
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            engine: EngineConfig::default(),
            blocks: vec![],
            signals: HashMap::new(),
            #[cfg(feature = "history")]
            storage: StorageConfig::default(),
            protocols: ProtocolsConfig::default(),
            #[cfg(feature = "security")]
            security: SecurityConfig::default(),
            #[cfg(feature = "alarms")]
            alarms: AlarmsConfig::default(),
            #[cfg(feature = "web")]
            web: WebConfig::default(),
            #[cfg(feature = "metrics")]
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: default_system_name(),
            description: String::new(),
            log_level: default_log_level(),
            #[cfg(feature = "hot-swap")]
            hot_reload: false,
            work_dir: default_work_dir(),
            timezone: default_timezone(),
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            scan_time_ms: default_scan_time(),
            max_jitter_percent: default_max_jitter(),
            performance_monitoring: true,
            thread_pool_size: None,
            priorities: PriorityConfig::default(),
        }
    }
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            high: default_high_priority(),
            medium: default_medium_priority(),
            low: default_low_priority(),
        }
    }
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================

fn default_system_name() -> String {
    "PETRA System".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_work_dir() -> PathBuf {
    PathBuf::from("./data")
}

fn default_timezone() -> String {
    "UTC".to_string()
}

fn default_scan_time() -> u64 {
    100 // 100ms
}

fn default_max_jitter() -> f64 {
    10.0 // 10%
}

fn default_high_priority() -> u32 {
    1 // Every scan
}

fn default_medium_priority() -> u32 {
    10 // Every 10th scan
}

fn default_low_priority() -> u32 {
    100 // Every 100th scan
}

fn default_true() -> bool {
    true
}

#[cfg(feature = "history")]
fn default_storage_path() -> PathBuf {
    PathBuf::from("./data/history")
}

#[cfg(feature = "history")]
fn default_retention_days() -> u32 {
    30
}

#[cfg(feature = "compression")]
fn default_compression_level() -> u32 {
    3
}

#[cfg(feature = "wal")]
fn default_wal_path() -> PathBuf {
    PathBuf::from("./data/wal")
}

#[cfg(feature = "wal")]
fn default_wal_max_size() -> u64 {
    100 * 1024 * 1024 // 100MB
}

#[cfg(feature = "wal")]
fn default_wal_retention() -> usize {
    10
}

#[cfg(feature = "mqtt")]
fn default_mqtt_client_id() -> String {
    format!("petra_{}", std::process::id())
}

#[cfg(feature = "mqtt")]
fn default_mqtt_keepalive() -> u16 {
    60
}

#[cfg(feature = "mqtt")]
fn default_mqtt_topic_prefix() -> String {
    "petra".to_string()
}

#[cfg(all(feature = "security", feature = "basic-auth"))]
fn default_auth_realm() -> String {
    "PETRA".to_string()
}

#[cfg(feature = "web")]
fn default_web_listen() -> String {
    "0.0.0.0:8080".to_string()
}

// Placeholder types for features not shown
#[cfg(feature = "metrics")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MetricsConfig {
    pub enabled: bool,
    pub listen: String,
}

#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ModbusConfig {
    pub mode: String,
    pub address: String,
}

#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S7Config {
    pub ip: String,
    pub rack: u16,
    pub slot: u16,
}

#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OpcUaConfig {
    pub endpoint: String,
    pub namespace: u16,
}

#[cfg(feature = "clickhouse")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ClickHouseConfig {
    pub url: String,
    pub database: String,
}

#[cfg(feature = "s3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
}

#[cfg(all(feature = "security", feature = "jwt-auth"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_hours: u32,
}

#[cfg(all(feature = "security", feature = "rbac"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct RbacConfig {
    pub roles_file: PathBuf,
}

#[cfg(all(feature = "security", feature = "audit"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AuditConfig {
    pub log_file: PathBuf,
    pub max_size_mb: u64,
}

#[cfg(all(feature = "alarms", feature = "email"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EmailNotificationConfig {
    pub smtp_server: String,
    pub from: String,
    pub to: Vec<String>,
}

#[cfg(all(feature = "alarms", feature = "twilio"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SmsNotificationConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from: String,
    pub to: Vec<String>,
}

#[cfg(feature = "web")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WebTlsConfig {
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
}

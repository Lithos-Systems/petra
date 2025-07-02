// src/config.rs - Updated configuration structures with burn-in support

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// ============================================================================
// MAIN CONFIGURATION
// ============================================================================

/// Main PETRA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct Config {
    /// Engine configuration
    #[serde(default)]
    pub engine: EngineConfig,
    
    /// Signal definitions
    pub signals: Vec<SignalConfig>,
    
    /// Block definitions
    pub blocks: Vec<BlockConfig>,
    
    /// MQTT configuration
    #[serde(default)]
    pub mqtt: Option<MqttConfig>,
    
    /// S7 configuration
    #[serde(default)]
    pub s7: Option<S7Config>,
    
    /// Alarm configuration
    #[serde(default)]
    pub alarms: Option<AlarmConfig>,
    
    /// Security configuration
    #[serde(default)]
    pub security: Option<SecurityConfig>,
    
    #[cfg(feature = "web")]
    /// Twilio configuration
    #[serde(default)]
    pub twilio: Option<TwilioConfig>,
    
    #[cfg(feature = "history")]
    /// History configuration
    #[serde(default)]
    pub history: Option<HistoryConfig>,
    
    #[cfg(feature = "advanced-storage")]
    /// Storage configuration
    #[serde(default)]
    pub storage: Option<StorageConfig>,
    
    #[cfg(feature = "opcua-support")]
    /// OPC UA configuration
    #[serde(default)]
    pub opcua: Option<OpcUaConfig>,
    
    #[cfg(feature = "modbus-support")]
    /// Modbus configuration
    #[serde(default)]
    pub modbus: Option<ModbusConfig>,
}

// ============================================================================
// ENGINE CONFIGURATION
// ============================================================================

/// Engine runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EngineConfig {
    /// Target scan time in milliseconds
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
    
    /// Burn-in configuration
    #[serde(default)]
    pub burn_in: BurnInConfig,
    
    /// Hot-swap configuration
    #[serde(default)]
    pub hot_swap: HotSwapConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            scan_time_ms: default_scan_time(),
            max_jitter_percent: default_max_jitter(),
            performance_monitoring: default_true(),
            thread_pool_size: None,
            priorities: PriorityConfig::default(),
            burn_in: BurnInConfig::default(),
            hot_swap: HotSwapConfig::default(),
        }
    }
}

/// Burn-in configuration for performance optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BurnInConfig {
    /// Enable burn-in optimization
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Number of cycles before burn-in
    #[serde(default = "default_burn_in_cycles")]
    pub cycles: u64,
    
    /// Lock signal paths after burn-in
    #[serde(default = "default_true")]
    pub lock_paths: bool,
    
    /// Optimize memory layout after burn-in
    #[serde(default = "default_true")]
    pub optimize_memory: bool,
}

impl Default for BurnInConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            cycles: default_burn_in_cycles(),
            lock_paths: default_true(),
            optimize_memory: default_true(),
        }
    }
}

/// Hot-swap configuration for runtime changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HotSwapConfig {
    /// Enable hot-swap functionality
    #[serde(default)]
    pub enabled: bool,
    
    /// Allow block hot-swap
    #[serde(default)]
    pub blocks: bool,
    
    /// Allow signal hot-swap
    #[serde(default)]
    pub signals: bool,
    
    /// Require burn-in reset after hot-swap
    #[serde(default = "default_true")]
    pub reset_burn_in: bool,
}

impl Default for HotSwapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            blocks: false,
            signals: false,
            reset_burn_in: default_true(),
        }
    }
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
// BLOCK CONFIGURATION
// ============================================================================

/// Individual block configuration with flexible I/O mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockConfig {
    /// Block name (unique identifier)
    pub name: String,
    
    /// Block type (AND, OR, TIMER_ON, etc.)
    pub block_type: String,
    
    /// Block description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Input signal mappings (port_name -> signal_name)
    /// During burn-in, these can be optimized to direct references
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    /// Output signal mappings (port_name -> signal_name)
    /// During burn-in, these can be optimized to direct references
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Block-specific parameters
    #[serde(default)]
    pub parameters: HashMap<String, serde_yaml::Value>,
    
    /// Execution priority
    #[serde(default)]
    pub priority: Priority,
    
    /// Enable/disable block
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Hot-swap configuration
    #[serde(default)]
    pub hot_swap: BlockHotSwapConfig,
}

/// Block-specific hot-swap configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockHotSwapConfig {
    /// Allow this block to be hot-swapped
    #[serde(default)]
    pub allow: bool,
    
    /// Preserve state during hot-swap
    #[serde(default)]
    pub preserve_state: bool,
    
    /// Custom state mapping for hot-swap
    #[serde(default)]
    pub state_mapping: HashMap<String, String>,
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

// ============================================================================
// SIGNAL CONFIGURATION
// ============================================================================

/// Signal configuration with burn-in optimization support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalConfig {
    /// Signal name (unique identifier)
    pub name: String,
    
    /// Signal type (bool, int, float, etc.)
    pub signal_type: String,
    
    /// Initial value
    #[serde(default)]
    pub initial: Option<serde_yaml::Value>,
    
    /// Signal description
    #[serde(default)]
    pub description: Option<String>,
    
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Engineering units
    #[cfg(feature = "engineering-types")]
    #[serde(default)]
    pub units: Option<String>,
    
    /// Enable quality tracking
    #[cfg(feature = "quality-codes")]
    #[serde(default)]
    pub quality_enabled: bool,
    
    /// Validation rules
    #[cfg(feature = "validation")]
    #[serde(default)]
    pub validation: Option<ValidationConfig>,
    
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
    
    /// Burn-in optimization hints
    #[serde(default)]
    pub optimization: SignalOptimizationConfig,
}

/// Signal optimization configuration for burn-in
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalOptimizationConfig {
    /// Pin in memory during burn-in
    #[serde(default)]
    pub pin_memory: bool,
    
    /// Cache line alignment hint
    #[serde(default)]
    pub cache_align: bool,
    
    /// Access frequency hint (0-100)
    #[serde(default)]
    pub access_frequency: u8,
}

// ============================================================================
// STORAGE CONFIGURATION
// ============================================================================

#[cfg(feature = "advanced-storage")]
/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct StorageConfig {
    /// Storage path
    pub path: PathBuf,
    
    /// Write-ahead log settings
    #[serde(default)]
    pub wal: WalConfig,
    
    /// Compression settings
    #[serde(default)]
    pub compression: CompressionConfig,
    
    /// Cache settings
    #[serde(default)]
    pub cache: CacheConfig,
}

#[cfg(feature = "advanced-storage")]
/// Write-ahead log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct WalConfig {
    /// Enable WAL
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Sync mode
    #[serde(default)]
    pub sync_mode: WalSyncMode,
    
    /// Size limit in MB
    #[serde(default = "default_wal_size")]
    pub size_limit_mb: u64,
}

#[cfg(feature = "advanced-storage")]
impl Default for WalConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            sync_mode: WalSyncMode::default(),
            size_limit_mb: default_wal_size(),
        }
    }
}

#[cfg(feature = "advanced-storage")]
/// WAL sync modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum WalSyncMode {
    None,
    Normal,
    Full,
}

#[cfg(feature = "advanced-storage")]
impl Default for WalSyncMode {
    fn default() -> Self {
        WalSyncMode::Normal
    }
}

// ============================================================================
// MQTT CONFIGURATION
// ============================================================================

/// MQTT configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttConfig {
    /// Broker configuration
    pub broker: MqttBrokerConfig,
    
    /// Client ID
    #[serde(default = "default_mqtt_client_id")]
    pub client_id: String,
    
    /// Username
    #[serde(default)]
    pub username: Option<String>,
    
    /// Password
    #[serde(default)]
    pub password: Option<String>,
    
    /// Keep alive in seconds
    #[serde(default = "default_mqtt_keepalive")]
    pub keepalive_secs: u64,
    
    /// Clean session
    #[serde(default = "default_true")]
    pub clean_session: bool,
    
    /// QoS level
    #[serde(default)]
    pub qos: MqttQos,
    
    /// TLS configuration
    #[serde(default)]
    pub tls: Option<MqttTlsConfig>,
    
    /// Topics configuration
    #[serde(default)]
    pub topics: MqttTopicsConfig,
}

/// MQTT broker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttBrokerConfig {
    /// Broker host
    pub host: String,
    
    /// Broker port
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
}

impl Default for MqttBrokerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: default_mqtt_port(),
        }
    }
}

/// MQTT QoS levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[serde(rename_all = "lowercase")]
pub enum MqttQos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

impl Default for MqttQos {
    fn default() -> Self {
        MqttQos::AtLeastOnce
    }
}

/// MQTT TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttTlsConfig {
    /// CA certificate path
    pub ca_cert: PathBuf,
    
    /// Client certificate path
    #[serde(default)]
    pub client_cert: Option<PathBuf>,
    
    /// Client key path
    #[serde(default)]
    pub client_key: Option<PathBuf>,
}

/// MQTT topics configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttTopicsConfig {
    /// Subscribe topics
    #[serde(default)]
    pub subscribe: Vec<MqttSubscription>,
    
    /// Publish topics
    #[serde(default)]
    pub publish: Vec<MqttPublication>,
}

/// MQTT subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttSubscription {
    /// Topic pattern
    pub topic: String,
    
    /// Target signal
    pub signal: String,
    
    /// Value path (for JSON payloads)
    #[serde(default)]
    pub path: Option<String>,
}

/// MQTT publication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttPublication {
    /// Topic
    pub topic: String,
    
    /// Source signal
    pub signal: String,
    
    /// Publish interval in milliseconds
    #[serde(default)]
    pub interval_ms: Option<u64>,
    
    /// Only publish on change
    #[serde(default)]
    pub on_change: bool,
}

// ============================================================================
// DEFAULT VALUE FUNCTIONS
// ============================================================================

fn default_scan_time() -> u64 { 100 }
fn default_max_jitter() -> f64 { 10.0 }
fn default_true() -> bool { true }
fn default_burn_in_cycles() -> u64 { 1000 }
fn default_high_priority() -> u32 { 1 }
fn default_medium_priority() -> u32 { 5 }
fn default_low_priority() -> u32 { 10 }
fn default_mqtt_client_id() -> String { "petra".to_string() }
fn default_mqtt_keepalive() -> u64 { 60 }
fn default_mqtt_port() -> u16 { 1883 }
#[cfg(feature = "advanced-storage")]
fn default_wal_size() -> u64 { 100 }

// ============================================================================
// OTHER CONFIGURATIONS (S7, Alarms, etc.)
// ============================================================================

/// S7 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S7Config {
    // Add S7 specific fields
}

/// Alarm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmConfig {
    // Add alarm specific fields
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SecurityConfig {
    // Add security specific fields
}

#[cfg(feature = "web")]
/// Twilio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct TwilioConfig {
    // Add Twilio specific fields
}

#[cfg(feature = "history")]
/// History configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HistoryConfig {
    // Add history specific fields
}

#[cfg(feature = "opcua-support")]
/// OPC UA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OpcUaConfig {
    // Add OPC UA specific fields
}

#[cfg(feature = "modbus-support")]
/// Modbus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ModbusConfig {
    // Add Modbus specific fields
}

#[cfg(feature = "validation")]
/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ValidationConfig {
    // Add validation specific fields
}

#[cfg(feature = "advanced-storage")]
/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CompressionConfig {
    // Add compression specific fields
}

#[cfg(feature = "advanced-storage")]
/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct CacheConfig {
    // Add cache specific fields
}

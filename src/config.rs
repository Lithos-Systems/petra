// src/config.rs - Configuration structures for PETRA
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{PlcError, Result};

/// Main configuration structure for PETRA
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Scan cycle time in milliseconds
    pub scan_time_ms: u64,
    
    /// Signal definitions
    pub signals: Vec<SignalConfig>,
    
    /// Block definitions
    pub blocks: Vec<BlockConfig>,
    
    /// MQTT configuration (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    
    /// Security configuration (optional)
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    
    /// History configuration (optional)
    #[cfg(feature = "history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
    
    /// Alarm configuration (optional)
    #[cfg(feature = "alarms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<AlarmConfig>,
    
    /// Web server configuration (optional)
    #[cfg(feature = "web")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<WebConfig>,
    
    /// Protocol configurations (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<HashMap<String, ProtocolConfig>>,
}

/// Signal configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignalConfig {
    /// Signal name (must be unique)
    pub name: String,
    
    /// Signal type (bool, int, float, etc.)
    #[serde(rename = "type")]
    pub signal_type: String,
    
    /// Initial value (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<serde_yaml::Value>,
    
    /// Description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Engineering units (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<String>,
    
    /// Quality code support (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    #[serde(default)]
    pub quality_enabled: bool,
    
    /// Validation rules (requires validation feature)
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Block configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockConfig {
    /// Block name (must be unique)
    pub name: String,
    
    /// Block type (AND, OR, NOT, etc.)
    #[serde(rename = "type")]
    pub block_type: String,
    
    /// Input signal mappings
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    /// Output signal mappings
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Block parameters (using 'params' to match your blocks/mod.rs)
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    
    /// Description (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Enhanced error handling configuration
    #[cfg(feature = "enhanced-errors")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handling: Option<ErrorHandlingConfig>,
    
    /// Circuit breaker configuration
    #[cfg(feature = "circuit-breaker")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

/// MQTT configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub client_id: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    
    #[serde(default = "default_qos")]
    pub qos: u8,
    
    #[serde(default = "default_keepalive")]
    pub keepalive_secs: u64,
    
    #[serde(default)]
    pub topics: Vec<MqttTopicConfig>,
    
    #[cfg(feature = "mqtt-tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

fn default_qos() -> u8 {
    1
}

fn default_keepalive() -> u64 {
    60
}

/// MQTT topic configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MqttTopicConfig {
    pub topic: String,
    pub signal: String,
    pub direction: MqttDirection,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// MQTT communication direction
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MqttDirection {
    Read,
    Write,
    ReadWrite,
}

/// Security configuration
#[cfg(feature = "security")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub enabled: bool,
    
    #[cfg(feature = "basic-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basic_auth: Option<BasicAuthConfig>,
    
    #[cfg(feature = "jwt-auth")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt_auth: Option<JwtAuthConfig>,
    
    #[cfg(feature = "rbac")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rbac: Option<RbacConfig>,
}

/// Basic authentication configuration
#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasicAuthConfig {
    pub realm: String,
    pub users: Vec<UserConfig>,
}

/// User configuration
#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserConfig {
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<String>,
}

/// JWT authentication configuration
#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtAuthConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub expiry_hours: u64,
}

/// Role-based access control configuration
#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RbacConfig {
    pub roles: HashMap<String, RoleConfig>,
}

/// Role configuration
#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoleConfig {
    pub permissions: Vec<String>,
    pub inherits: Vec<String>,
}

/// History configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryConfig {
    pub enabled: bool,
    pub data_dir: PathBuf,
    pub retention_days: u32,
    pub batch_size: usize,
    
    #[cfg(feature = "compression")]
    pub compression: CompressionType,
    
    #[serde(default)]
    pub signals: Vec<HistorySignalConfig>,
}

/// History signal configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistorySignalConfig {
    pub signal: String,
    pub interval_ms: u64,
    pub deadband: Option<f64>,
}

/// Compression type
#[cfg(feature = "compression")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    None,
    Zstd,
    Lz4,
    Snappy,
}

/// Alarm configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmConfig {
    pub enabled: bool,
    pub alarms: Vec<AlarmDefinition>,
}

/// Alarm definition
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmDefinition {
    pub name: String,
    pub signal: String,
    pub condition: AlarmCondition,
    pub severity: AlarmSeverity,
    pub message: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay_ms: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_ack: Option<bool>,
}

/// Alarm condition
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmCondition {
    High(f64),
    Low(f64),
    Equal(serde_yaml::Value),
    NotEqual(serde_yaml::Value),
    InRange(f64, f64),
    OutOfRange(f64, f64),
}

/// Alarm severity
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AlarmSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Web server configuration
#[cfg(feature = "web")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub static_dir: Option<PathBuf>,
    
    #[cfg(feature = "web-tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
}

/// TLS configuration
#[cfg(any(feature = "mqtt-tls", feature = "web-tls"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    pub cert_file: PathBuf,
    pub key_file: PathBuf,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_file: Option<PathBuf>,
    
    #[serde(default)]
    pub verify_peer: bool,
}

/// Protocol configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ProtocolConfig {
    #[cfg(feature = "modbus-support")]
    Modbus(ModbusConfig),
    
    #[cfg(feature = "s7-support")]
    S7(S7Config),
    
    #[cfg(feature = "opcua-support")]
    OpcUa(OpcUaConfig),
}

/// Modbus configuration
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusConfig {
    pub mode: ModbusMode,
    pub slave_id: u8,
    pub mappings: Vec<ModbusMapping>,
}

/// Modbus mode
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModbusMode {
    Tcp { host: String, port: u16 },
    Rtu { port: String, baud_rate: u32 },
}

/// Modbus mapping
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusMapping {
    pub signal: String,
    pub register_type: ModbusRegisterType,
    pub address: u16,
    pub count: u16,
    pub data_type: ModbusDataType,
}

/// Modbus register type
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModbusRegisterType {
    Coil,
    DiscreteInput,
    HoldingRegister,
    InputRegister,
}

/// Modbus data type
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ModbusDataType {
    Bool,
    U16,
    I16,
    U32,
    I32,
    F32,
}

/// S7 configuration
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S7Config {
    pub host: String,
    pub rack: u16,
    pub slot: u16,
    pub mappings: Vec<S7Mapping>,
}

/// S7 mapping
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S7Mapping {
    pub signal: String,
    pub area: S7Area,
    pub db_number: u16,
    pub offset: u32,
    pub data_type: S7DataType,
}

/// S7 area
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "uppercase")]
pub enum S7Area {
    PE,
    PA,
    MK,
    DB,
    CT,
    TM,
}

/// S7 data type
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum S7DataType {
    Bool,
    Byte,
    Word,
    DWord,
    Int,
    DInt,
    Real,
}

/// OPC-UA configuration
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpcUaConfig {
    pub endpoint: String,
    pub namespace: u16,
    pub mappings: Vec<OpcUaMapping>,
}

/// OPC-UA mapping
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpcUaMapping {
    pub signal: String,
    pub node_id: String,
    pub direction: OpcUaDirection,
}

/// OPC-UA direction
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpcUaDirection {
    Read,
    Write,
    Subscribe,
}

/// Validation configuration
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ValidationConfig {
    Range { min: f64, max: f64 },
    Enum { values: Vec<serde_yaml::Value> },
    Pattern { regex: String },
    Custom { validator: String },
}

/// Error handling configuration
#[cfg(feature = "enhanced-errors")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorHandlingConfig {
    pub retry: Option<RetryConfig>,
    pub fallback_value: Option<serde_yaml::Value>,
    pub log_errors: bool,
}

/// Retry configuration
#[cfg(feature = "enhanced-errors")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

/// Circuit breaker configuration
#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    pub max_failures: u32,
    pub reset_timeout_ms: u64,
    pub half_open_max_calls: u32,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from a YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let config: Config = serde_yaml::from_str(yaml)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate scan time
        if self.scan_time_ms == 0 {
            return Err(PlcError::Config("Scan time must be greater than 0".to_string()));
        }
        
        // Validate signals
        let mut signal_names = std::collections::HashSet::new();
        for signal in &self.signals {
            if !signal_names.insert(&signal.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate signal name: '{}'", signal.name
                )));
            }
            signal.validate()?;
        }
        
        // Validate blocks
        let mut block_names = std::collections::HashSet::new();
        for block in &self.blocks {
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate block name: '{}'", block.name
                )));
            }
            block.validate(&signal_names)?;
        }
        
        // Validate optional configurations
        if let Some(mqtt) = &self.mqtt {
            mqtt.validate()?;
        }
        
        #[cfg(feature = "security")]
        if let Some(security) = &self.security {
            security.validate()?;
        }
        
        Ok(())
    }
}

impl SignalConfig {
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(PlcError::Config("Signal name cannot be empty".to_string()));
        }
        
        // Validate signal type
        match self.signal_type.as_str() {
            "bool" | "int" | "float" => Ok(()),
            #[cfg(feature = "extended-types")]
            "string" | "binary" | "timestamp" | "array" | "object" => Ok(()),
            _ => Err(PlcError::Config(format!(
                "Invalid signal type: '{}'", self.signal_type
            ))),
        }
    }
}

impl BlockConfig {
    fn validate(&self, available_signals: &std::collections::HashSet<&String>) -> Result<()> {
        if self.name.is_empty() {
            return Err(PlcError::Config("Block name cannot be empty".to_string()));
        }
        
        if self.block_type.is_empty() {
            return Err(PlcError::Config("Block type cannot be empty".to_string()));
        }
        
        // Validate signal references
        for signal in self.inputs.values() {
            if !available_signals.contains(signal) {
                return Err(PlcError::Config(format!(
                    "Block '{}' references unknown input signal '{}'",
                    self.name, signal
                )));
            }
        }
        
        for signal in self.outputs.values() {
            if !available_signals.contains(signal) {
                return Err(PlcError::Config(format!(
                    "Block '{}' references unknown output signal '{}'",
                    self.name, signal
                )));
            }
        }
        
        Ok(())
    }
}

impl MqttConfig {
    fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(PlcError::Config("MQTT host cannot be empty".to_string()));
        }
        
        if self.port == 0 {
            return Err(PlcError::Config("MQTT port cannot be 0".to_string()));
        }
        
        if self.client_id.is_empty() {
            return Err(PlcError::Config("MQTT client ID cannot be empty".to_string()));
        }
        
        if self.qos > 2 {
            return Err(PlcError::Config("MQTT QoS must be 0, 1, or 2".to_string()));
        }
        
        Ok(())
    }
}

#[cfg(feature = "security")]
impl SecurityConfig {
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
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_validation() {
        let config = Config {
            scan_time_ms: 100,
            signals: vec![
                SignalConfig {
                    name: "input1".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: None,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ],
            blocks: vec![
                BlockConfig {
                    name: "not1".to_string(),
                    block_type: "NOT".to_string(),
                    inputs: {
                        let mut map = HashMap::new();
                        map.insert("in".to_string(), "input1".to_string());
                        map
                    },
                    outputs: HashMap::new(),
                    params: HashMap::new(),
                    description: None,
                    tags: vec![],
                },
            ],
            mqtt: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "web")]
            web: None,
            protocols: None,
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_duplicate_signal_names() {
        let config = Config {
            scan_time_ms: 100,
            signals: vec![
                SignalConfig {
                    name: "signal1".to_string(),
                    signal_type: "bool".to_string(),
                    initial: None,
                    description: None,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
                SignalConfig {
                    name: "signal1".to_string(), // Duplicate!
                    signal_type: "int".to_string(),
                    initial: None,
                    description: None,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ],
            blocks: vec![],
            mqtt: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "web")]
            web: None,
            protocols: None,
        };
        
        let result = config.validate();
        assert!(result.is_err());
        if let Err(PlcError::Config(msg)) = result {
            assert!(msg.contains("Duplicate signal name"));
        }
    }
}

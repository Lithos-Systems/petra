// src/config.rs - Fixed configuration with missing MQTT fields
use crate::error::{PlcError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Scan time in milliseconds (direct field, not nested)
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    
    /// Maximum scan jitter allowed in milliseconds
    #[serde(default = "default_max_jitter")]
    pub max_scan_jitter_ms: u64,
    
    /// Enable error recovery
    #[serde(default = "default_error_recovery")]
    pub error_recovery: bool,
    
    /// Signal definitions
    #[serde(default)]
    pub signals: Vec<SignalConfig>,
    
    /// Block definitions
    #[serde(default)]
    pub blocks: Vec<BlockConfig>,
    
    /// MQTT configuration
    #[cfg(feature = "mqtt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mqtt: Option<MqttConfig>,
    
    /// Security configuration
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    
    /// History/storage configuration
    #[cfg(feature = "history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
    
    /// Alarm configuration
    #[cfg(feature = "alarms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<AlarmConfig>,
    
    /// Web configuration
    #[cfg(feature = "web")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web: Option<WebConfig>,
    
    /// Protocol configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<ProtocolConfig>,
}

// Default functions
fn default_scan_time() -> u64 { 100 }
fn default_max_jitter() -> u64 { 50 }
fn default_error_recovery() -> bool { true }

/// Engine configuration (if you prefer nested config)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineConfig {
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    
    #[serde(default = "default_max_jitter")]
    pub max_scan_jitter_ms: u64,
    
    #[serde(default = "default_error_recovery")]
    pub error_recovery: bool,
}

/// Signal configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SignalConfig {
    /// Signal name (unique identifier)
    pub name: String,
    
    /// Signal type (bool, int, float, etc.)
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
    
    /// Enable quality codes (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    #[serde(default)]
    pub quality_enabled: bool,
    
    /// Validation rules (requires validation feature)
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
    
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_yaml::Value>,
}

/// Block configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockConfig {
    /// Block name (unique identifier)
    pub name: String,
    
    /// Block type identifier
    #[serde(rename = "type")]
    pub block_type: String,
    
    /// Input signal mappings
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    /// Output signal mappings
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    /// Block parameters (using 'params' to match blocks/mod.rs)
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

/// MQTT configuration - Fixed with missing fields
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
    
    /// Missing field: Clean session flag
    #[serde(default = "default_clean_session")]
    pub clean_session: bool,
    
    /// Missing field: TLS configuration
    #[cfg(feature = "mqtt-tls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsConfig>,
    
    /// Missing field: Topic configurations
    #[serde(default)]
    pub topics: Vec<MqttTopicConfig>,
}

// Default functions for MQTT
fn default_qos() -> u8 { 1 }
fn default_keepalive() -> u64 { 60 }
fn default_clean_session() -> bool { true }

/// TLS configuration for MQTT
#[cfg(feature = "mqtt-tls")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    /// CA certificate path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert: Option<String>,
    
    /// Client certificate path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_cert: Option<String>,
    
    /// Client private key path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_key: Option<String>,
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
    Publish,
    Subscribe,
}

/// Enhanced error handling configuration
#[cfg(feature = "enhanced-errors")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorHandlingConfig {
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub escalate_on_failure: bool,
}

/// Circuit breaker configuration
#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout_ms: u64,
    pub half_open_max_calls: u32,
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
    pub jwt: Option<JwtConfig>,
    
    #[cfg(feature = "rbac")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rbac: Option<RbacConfig>,
}

/// Basic authentication configuration
#[cfg(feature = "basic-auth")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BasicAuthConfig {
    pub users: HashMap<String, String>, // username -> password hash
}

/// JWT configuration
#[cfg(feature = "jwt-auth")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration_hours: u64,
}

/// Role-based access control configuration
#[cfg(feature = "rbac")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RbacConfig {
    pub roles: HashMap<String, Vec<String>>, // role -> permissions
    pub user_roles: HashMap<String, Vec<String>>, // user -> roles
}

/// History configuration
#[cfg(feature = "history")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryConfig {
    pub data_dir: PathBuf,
    pub retention_days: u32,
    pub max_batch_size: usize,
}

/// Alarm configuration
#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlarmConfig {
    pub enabled: bool,
    
    #[cfg(feature = "email")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailConfig>,
    
    #[cfg(feature = "twilio")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twilio: Option<TwilioConfig>,
}

/// Email configuration
#[cfg(feature = "email")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

/// Twilio configuration
#[cfg(feature = "twilio")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
    pub to_numbers: Vec<String>,
}

/// Web configuration
#[cfg(feature = "web")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebConfig {
    pub enabled: bool,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

/// Validation configuration
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationConfig {
    pub rules: Vec<ValidationRule>,
}

/// Validation rule
#[cfg(feature = "validation")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidationRule {
    pub rule_type: String,
    pub parameters: HashMap<String, serde_yaml::Value>,
}

/// Protocol configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProtocolConfig {
    #[cfg(feature = "s7-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s7: Option<S7Config>,
    
    #[cfg(feature = "modbus-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modbus: Option<ModbusConfig>,
    
    #[cfg(feature = "opcua-support")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcua: Option<OpcuaConfig>,
}

/// S7 protocol configuration
#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct S7Config {
    pub ip: String,
    pub rack: u16,
    pub slot: u16,
}

/// Modbus protocol configuration
#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModbusConfig {
    pub ip: String,
    pub port: u16,
    pub unit_id: u8,
}

/// OPC-UA protocol configuration
#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpcuaConfig {
    pub endpoint: String,
    pub security_policy: String,
}

// ============================================================================
// IMPLEMENTATION
// ============================================================================

impl Config {
    /// Load configuration from file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| PlcError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| PlcError::Config(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the entire configuration
    pub fn validate(&self) -> Result<()> {
        // Validate scan time
        if self.scan_time_ms == 0 {
            return Err(PlcError::Config("Scan time cannot be zero".to_string()));
        }
        
        if self.scan_time_ms > 10000 {
            return Err(PlcError::Config("Scan time too large (max 10 seconds)".to_string()));
        }
        
        // Validate signals
        let mut signal_names = std::collections::HashSet::new();
        for signal in &self.signals {
            if !signal_names.insert(&signal.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate signal name: {}", signal.name
                )));
            }
            signal.validate()?;
        }
        
        // Create signal name lookup for block validation
        let signal_map: HashMap<&String, &str> = self.signals
            .iter()
            .map(|s| (&s.name, s.signal_type.as_str()))
            .collect();
        
        // Validate blocks
        let mut block_names = std::collections::HashSet::new();
        for block in &self.blocks {
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate block name: {}", block.name
                )));
            }
            block.validate(&signal_map)?;
        }
        
        // Validate MQTT configuration
        #[cfg(feature = "mqtt")]
        if let Some(mqtt) = &self.mqtt {
            mqtt.validate()?;
        }
        
        // Validate security configuration
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
        
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(PlcError::Config(format!(
                "Signal name '{}' contains invalid characters", self.name
            )));
        }
        
        match self.signal_type.as_str() {
            "bool" | "int" | "float" => {}
            #[cfg(feature = "extended-types")]
            "string" | "binary" | "timestamp" | "array" | "object" => {}
            _ => return Err(PlcError::Config(format!(
                "Unknown signal type: {}", self.signal_type
            ))),
        }
        
        Ok(())
    }
}

impl BlockConfig {
    fn validate(&self, available_signals: &HashMap<&String, &str>) -> Result<()> {
        if self.name.is_empty() {
            return Err(PlcError::Config("Block name cannot be empty".to_string()));
        }
        
        if self.block_type.is_empty() {
            return Err(PlcError::Config("Block type cannot be empty".to_string()));
        }
        
        // Validate input signals exist
        for (input_name, signal_name) in &self.inputs {
            if !available_signals.contains_key(signal_name) {
                return Err(PlcError::Config(format!(
                    "Block '{}' input '{}' references unknown signal '{}'",
                    self.name, input_name, signal_name
                )));
            }
        }
        
        // Validate output signals exist
        for (output_name, signal_name) in &self.outputs {
            if !available_signals.contains_key(signal_name) {
                return Err(PlcError::Config(format!(
                    "Block '{}' output '{}' references unknown signal '{}'",
                    self.name, output_name, signal_name
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
            max_scan_jitter_ms: 50,
            error_recovery: true,
            signals: vec![
                SignalConfig {
                    name: "input1".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: None,
                    tags: vec![],
                    #[cfg(feature = "engineering-types")]
                    units: None,
                    #[cfg(feature = "quality-codes")]
                    quality_enabled: false,
                    #[cfg(feature = "validation")]
                    validation: None,
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
                    #[cfg(feature = "enhanced-errors")]
                    error_handling: None,
                    #[cfg(feature = "circuit-breaker")]
                    circuit_breaker: None,
                },
            ],
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
            protocols: None,
        };
        
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_mqtt_validation() {
        let mqtt = MqttConfig {
            host: "localhost".to_string(),
            port: 1883,
            client_id: "test".to_string(),
            username: None,
            password: None,
            qos: 1,
            keepalive_secs: 60,
            clean_session: true,
            #[cfg(feature = "mqtt-tls")]
            tls: None,
            topics: vec![],
        };
        
        assert!(mqtt.validate().is_ok());
        
        let invalid_mqtt = MqttConfig {
            host: "".to_string(), // Invalid empty host
            port: 1883,
            client_id: "test".to_string(),
            username: None,
            password: None,
            qos: 1,
            keepalive_secs: 60,
            clean_session: true,
            #[cfg(feature = "mqtt-tls")]
            tls: None,
            topics: vec![],
        };
        
        assert!(invalid_mqtt.validate().is_err());
    }
}

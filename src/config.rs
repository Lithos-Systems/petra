// src/config.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use crate::error::{Result, PlcError};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// Conditionally import security types
#[cfg(feature = "security")]
use crate::security::{SecurityConfig, SignatureConfig, verify_signature, sign_config};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct Config {
    #[serde(default)]
    pub signals: Vec<SignalConfig>,
    
    #[serde(default)]
    pub blocks: Vec<BlockConfig>,
    
    pub scan_time_ms: u64,
    
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
    
    #[cfg(feature = "alarms")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alarms: Option<Vec<AlarmConfig>>,
    
    #[cfg(feature = "history")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<HistoryConfig>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_config: Option<serde_yaml::Value>,
    
    #[cfg(feature = "security")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<SecurityConfig>,
    
    #[cfg(feature = "security")]
    #[serde(skip)]
    signature: Option<SignatureConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalConfig {
    pub name: String,
    
    #[serde(rename = "type")]
    pub signal_type: String,
    
    #[serde(default)]
    pub initial: serde_yaml::Value,
    
    #[cfg(feature = "validation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationConfig>,
    
    #[cfg(feature = "enhanced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockConfig {
    pub name: String,
    
    #[serde(rename = "type")]
    pub block_type: String,
    
    #[serde(default)]
    pub inputs: HashMap<String, String>,
    
    #[serde(default)]
    pub outputs: HashMap<String, String>,
    
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    
    #[cfg(feature = "enhanced")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    #[cfg(feature = "enhanced")]
    #[serde(default)]
    pub tags: Vec<String>,
}

#[cfg(feature = "validation")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ValidationConfig {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub pattern: Option<String>,
    pub required: Option<bool>,
}

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
    
    #[cfg(feature = "security")]
    /// Load a signed configuration file
    pub fn from_signed_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)?;
        
        // Extract signature and config sections
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
        
        // Parse signature
        let signature: SignatureConfig = serde_yaml::from_str(sig_section)?;
        
        // Verify signature
        if signature.verify_enabled {
            verify_signature(config_section.as_bytes(), &signature)?;
        }
        
        // Parse config
        let mut config: Config = serde_yaml::from_str(config_section)?;
        config.signature = Some(signature);
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a YAML file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| PlcError::Config(format!("YAML serialization error: {}", e)))?;
        
        #[cfg(feature = "security")]
        if let Some(sig) = &self.signature {
            let signed = format!(
                "---BEGIN SIGNATURE---\n{}\n---BEGIN CONFIG---\n{}\n---END CONFIG---",
                serde_yaml::to_string(sig)
                    .map_err(|e| PlcError::Config(format!("YAML serialization error: {}", e)))?,
                yaml
            );
            std::fs::write(path, signed)?;
        } else {
            std::fs::write(path, yaml)?;
        }
        
        #[cfg(not(feature = "security"))]
        std::fs::write(path, yaml)?;
        
        Ok(())
    }
    
    /// Save configuration (alternative method)
    pub fn save(&self, path: &Path) -> Result<()> {
        self.to_file(path)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check scan time
        if self.scan_time_ms < 1 {
            return Err(PlcError::Config("Scan time must be at least 1ms".into()));
        }
        
        if self.scan_time_ms > 60000 {
            return Err(PlcError::Config("Scan time must be less than 60s".into()));
        }
        
        // Validate signal names are unique
        let mut signal_names = std::collections::HashSet::new();
        for signal in &self.signals {
            if !signal_names.insert(&signal.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate signal name: {}", signal.name
                )));
            }
            
            // Validate signal type
            match signal.signal_type.as_str() {
                "bool" | "int" | "float" => {},
                _ => return Err(PlcError::Config(format!(
                    "Invalid signal type '{}' for signal '{}'",
                    signal.signal_type, signal.name
                ))),
            }
        }
        
        // Validate block names are unique
        let mut block_names = std::collections::HashSet::new();
        for block in &self.blocks {
            if !block_names.insert(&block.name) {
                return Err(PlcError::Config(format!(
                    "Duplicate block name: {}", block.name
                )));
            }
        }
        
        // Validate MQTT config if present
        #[cfg(feature = "mqtt")]
        if let Some(mqtt) = &self.mqtt {
            if mqtt.broker_host.is_empty() {
                return Err(PlcError::Config("MQTT broker host cannot be empty".into()));
            }
        }
        
        // Feature-specific validation
        #[cfg(feature = "s7-support")]
        if let Some(s7) = &self.s7 {
            s7.validate()?;
        }
        
        #[cfg(feature = "security")]
        if let Some(security) = &self.security {
            security.validate()?;
        }
        
        Ok(())
    }
    
    #[cfg(feature = "json-schema")]
    /// Generate JSON schema for the configuration
    pub fn json_schema() -> schemars::schema::RootSchema {
        schemars::schema_for!(Config)
    }
    
    #[cfg(feature = "security")]
    /// Sign the configuration with a private key
    pub fn sign(&mut self, key_path: &Path) -> Result<()> {
        let yaml_str = serde_yaml::to_string(self)?;
        let config_bytes = yaml_str.into_bytes();
        let signature = sign_config(&config_bytes, key_path)?;
        
        self.signature = Some(signature);
        
        Ok(())
    }
}

// Feature-specific config types that should be defined in their respective modules
#[cfg(feature = "mqtt")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub keep_alive_secs: Option<u64>,
    pub clean_session: Option<bool>,
    pub qos: Option<u8>,
}

#[cfg(feature = "s7-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct S7Config {
    pub plc_address: String,
    pub rack: u16,
    pub slot: u16,
    pub connection_type: Option<String>,
    pub timeout_ms: Option<u32>,
}

#[cfg(feature = "s7-support")]
impl S7Config {
    pub fn validate(&self) -> Result<()> {
        if self.plc_address.is_empty() {
            return Err(PlcError::Config("S7 PLC address cannot be empty".into()));
        }
        Ok(())
    }
}

#[cfg(feature = "modbus-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ModbusConfig {
    pub server_address: String,
    pub port: u16,
    pub slave_id: u8,
    pub timeout_ms: Option<u32>,
    pub retry_count: Option<u32>,
}

#[cfg(feature = "opcua-support")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct OpcUaConfig {
    pub endpoint_url: String,
    pub security_mode: Option<String>,
    pub security_policy: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[cfg(feature = "alarms")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct AlarmConfig {
    pub name: String,
    pub condition: String,
    pub severity: String,
    pub message: String,
    pub auto_acknowledge: Option<bool>,
}

#[cfg(feature = "history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct HistoryConfig {
    pub storage_path: std::path::PathBuf,
    pub retention_days: Option<u32>,
    pub compression: Option<bool>,
    pub batch_size: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_load_save() {
        let config = Config {
            signals: vec![
                SignalConfig {
                    name: "test_signal".to_string(),
                    signal_type: "bool".to_string(),
                    initial: serde_yaml::Value::Bool(false),
                    #[cfg(feature = "validation")]
                    validation: None,
                    #[cfg(feature = "enhanced")]
                    metadata: None,
                },
            ],
            blocks: vec![],
            scan_time_ms: 100,
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "s7-support")]
            s7: None,
            #[cfg(feature = "modbus-support")]
            modbus: None,
            #[cfg(feature = "opcua-support")]
            opcua: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "history")]
            history: None,
            engine_config: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "security")]
            signature: None,
        };

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        config.to_file(temp_file.path()).unwrap();

        // Load back
        let loaded = Config::from_file(temp_file.path()).unwrap();
        assert_eq!(loaded.signals.len(), 1);
        assert_eq!(loaded.signals[0].name, "test_signal");
        assert_eq!(loaded.scan_time_ms, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config {
            signals: vec![],
            blocks: vec![],
            scan_time_ms: 0, // Invalid
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "s7-support")]
            s7: None,
            #[cfg(feature = "modbus-support")]
            modbus: None,
            #[cfg(feature = "opcua-support")]
            opcua: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "history")]
            history: None,
            engine_config: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "security")]
            signature: None,
        };

        assert!(config.validate().is_err());

        config.scan_time_ms = 100;
        assert!(config.validate().is_ok());
    }

    #[cfg(feature = "json-schema")]
    #[test]
    fn test_json_schema_generation() {
        let schema = Config::json_schema();
        let json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(json.contains("\"title\": \"Config\""));
        assert!(json.contains("\"type\": \"object\""));
    }
}

use crate::error::*;
use crate::mqtt::MqttConfig;
use crate::s7::S7Config;
use crate::twilio::TwilioConfig;
use crate::history::HistoryConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct Config {
    pub signals: Vec<SignalConfig>,
    pub blocks: Vec<BlockConfig>,
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    #[serde(default)]
    pub mqtt: MqttConfig,
    #[serde(default)]
    pub s7: Option<S7Config>,
    #[serde(default)]
    pub twilio: Option<TwilioConfig>,
    #[serde(default)]
    pub history: Option<HistoryConfig>,
    #[cfg(feature = "advanced-storage")]
    #[serde(default)]
    pub storage: Option<crate::storage::StorageConfig>,
    #[serde(default)]
    pub alarms: Option<AlarmConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub signal_type: String,
    #[serde(default)]
    pub initial: serde_yaml::Value,
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
}

fn default_scan_time() -> u64 {
    100
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(yaml)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        let mut seen = std::collections::HashSet::new();

        for signal in &self.signals {
            if !seen.insert(&signal.name) {
                return Err(PlcError::Config(format!("Duplicate signal: {}", signal.name)));
            }
            match signal.signal_type.as_str() {
                "bool" | "int" | "float" => {}
                _ => return Err(PlcError::Config(format!("Invalid signal type: {}", signal.signal_type))),
            }
        }

        seen.clear();
        for block in &self.blocks {
            if !seen.insert(&block.name) {
                return Err(PlcError::Config(format!("Duplicate block: {}", block.name)));
            }
        }

        if self.scan_time_ms < 10 || self.scan_time_ms > 10000 {
            return Err(PlcError::Config("Scan time must be 10–10000ms".into()));
        }

        Ok(())
    }
}

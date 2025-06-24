// src/config_schema.rs - New file
use schemars::{schema_for, JsonSchema};
use serde_json;
use std::fs;
use std::path::Path;

// Make config types derive JsonSchema
use crate::config::*;
use crate::mqtt::MqttConfig;
use crate::s7::{S7Config, S7Mapping, S7Area, S7DataType, Direction};
use crate::twilio::{TwilioConfig, TwilioAction, TwilioActionType};
use crate::history::{HistoryConfig, DownsampleRule, AggregationType};

// Generate and write schema
pub fn generate_schema() -> Result<(), Box<dyn std::error::Error>> {
    let schema = schema_for!(Config);
    let schema_json = serde_json::to_string_pretty(&schema)?;
    
    fs::create_dir_all("schemas")?;
    fs::write("schemas/petra-config.schema.json", schema_json)?;
    
    println!("Generated JSON schema at schemas/petra-config.schema.json");
    Ok(())
}

// Validate config against schema
pub fn validate_config(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(config_path)?;
    let config_value: serde_json::Value = serde_yaml::from_str(&config_str)?;
    
    // Load schema
    let schema_str = fs::read_to_string("schemas/petra-config.schema.json")?;
    let schema: serde_json::Value = serde_json::from_str(&schema_str)?;
    
    // Validate using jsonschema crate
    let compiled = jsonschema::JSONSchema::compile(&schema)?;
    let result = compiled.validate(&config_value);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Validation error: {}", error);
        }
        return Err("Config validation failed".into());
    }
    
    Ok(())
}

// Binary to generate schema
// src/bin/generate_schema.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    petra::config_schema::generate_schema()
}

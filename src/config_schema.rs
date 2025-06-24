#[cfg(feature = "json-schema")]
use schemars::{schema_for, JsonSchema};
use serde_json;
use std::fs;
use std::path::Path;
use crate::error::Result;
use crate::config::Config;

#[cfg(feature = "json-schema")]
pub fn generate_schema() -> Result<()> {
    let schema = schema_for!(Config);
    let schema_json = serde_json::to_string_pretty(&schema)
        .map_err(|e| crate::PlcError::Config(format!("Failed to serialize schema: {}", e)))?;
    
    fs::create_dir_all("schemas")?;
    fs::write("schemas/petra-config.schema.json", schema_json)?;
    
    println!("Generated JSON schema at schemas/petra-config.schema.json");
    Ok(())
}

#[cfg(feature = "json-schema")]
pub fn validate_config(config_path: &Path) -> Result<()> {
    let config_str = fs::read_to_string(config_path)?;
    let config_value: serde_json::Value = serde_yaml::from_str(&config_str)
        .map_err(|e| crate::PlcError::Yaml(e))?;
    
    let schema_str = fs::read_to_string("schemas/petra-config.schema.json")?;
    let schema: serde_json::Value = serde_json::from_str(&schema_str)
        .map_err(|e| crate::PlcError::Config(format!("Failed to parse schema: {}", e)))?;
    
    let compiled = jsonschema::JSONSchema::compile(&schema)
        .map_err(|e| crate::PlcError::Config(format!("Failed to compile schema: {}", e)))?;
    
    let result = compiled.validate(&config_value);
    
    if let Err(errors) = result {
        for error in errors {
            eprintln!("Validation error: {}", error);
        }
        return Err(crate::PlcError::Config("Config validation failed".into()));
    }
    
    Ok(())
}

// src/blocks/mod.rs - Block system implementation with fixed parameter references

pub mod base;
pub mod timer;
pub mod arithmetic;  // Changed from math to arithmetic
pub mod data;

#[cfg(feature = "edge-detection")]
pub mod edge;

#[cfg(feature = "memory-blocks")]
pub mod memory;

#[cfg(feature = "pid-control")]
pub mod pid;

#[cfg(feature = "communication")]
pub mod comm;

#[cfg(feature = "state-machine")]
pub mod state;

#[cfg(feature = "advanced-math")]
pub mod advanced_math;

#[cfg(feature = "ml")]
pub mod ml;

#[cfg(feature = "circuit-breaker")]
pub mod circuit_breaker;

#[cfg(feature = "simd-math")]
pub mod simd_math;

use crate::{
    config::BlockConfig,
    error::{PlcError, Result},
    signal::SignalBus,
};
use std::collections::HashMap;

#[cfg(feature = "enhanced-monitoring")]
use std::time::{Duration, Instant};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

/// Core trait that all blocks must implement
/// 
/// Blocks are the fundamental processing units in PETRA. They read signals
/// from the signal bus, perform computation, and write results back to the bus.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{Block, SignalBus, Value, Result};
/// 
/// struct MyBlock {
///     name: String,
///     input: String,
///     output: String,
/// }
/// 
/// impl Block for MyBlock {
///     fn execute(&mut self, bus: &SignalBus) -> Result<()> {
///         let input_value = bus.get_bool(&self.input)?;
///         bus.set(&self.output, Value::Bool(!input_value))?;
///         Ok(())
///     }
///     
///     fn name(&self) -> &str {
///         &self.name
///     }
///     
///     fn block_type(&self) -> &str {
///         "MY_BLOCK"
///     }
/// }
/// ```
pub trait Block: Send + Sync {
    /// Execute the block logic
    /// 
    /// This method is called during each scan cycle. It should:
    /// - Read input signals from the signal bus
    /// - Perform the block's computation
    /// - Write output signals back to the signal bus
    /// - Complete quickly (typically < 1ms for real-time performance)
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    
    /// Get the block's name (unique identifier)
    fn name(&self) -> &str;
    
    /// Get the block's type identifier
    fn block_type(&self) -> &str;
    
    /// Get the block's category (for organization)
    fn category(&self) -> &str {
        "general"
    }
    
    /// Validate block configuration (called at creation time)
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        // Default validation - subclasses can override
        if config.name.is_empty() {
            return Err(PlcError::Config("Block name cannot be empty".to_string()));
        }
        Ok(())
    }
    
    /// Initialize block with configuration
    fn initialize(&mut self, _config: &BlockConfig) -> Result<()> {
        Ok(())
    }
    
    /// Reset block to initial state
    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Get block description
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Enhanced monitoring support
    #[cfg(feature = "enhanced-monitoring")]
    fn last_execution_time(&self) -> Option<Duration> {
        None
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    fn execution_count(&self) -> u64 {
        0
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    fn error_count(&self) -> u64 {
        0
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    fn state(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
}

/// Block factory function - creates blocks based on configuration
/// 
/// All block types must be registered here.
/// 
/// # Examples
/// 
/// ```rust,ignore
/// use petra::blocks::create_block;
/// use petra::config::BlockConfig;
/// use std::collections::HashMap;
/// 
/// let config = BlockConfig {
///     name: "my_and_gate".to_string(),
///     block_type: "AND".to_string(),
///     inputs: HashMap::new(),
///     outputs: HashMap::new(),
///     params: HashMap::new(),
///     description: None,
///     tags: vec![],
/// };
/// 
/// let block = create_block(&config)?;
/// assert_eq!(block.block_type(), "AND");
/// # Ok::<(), petra::PlcError>(())
/// ```
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
    match config.block_type.as_str() {
        // Base logic blocks (always available)
        "AND" => base::create_and_block(config),
        "OR" => base::create_or_block(config),
        "NOT" => base::create_not_block(config),
        "XOR" => base::create_xor_block(config),
        
        // Comparison blocks (always available)
        "GT" => base::create_gt_block(config),
        "LT" => base::create_lt_block(config),
        "GTE" => base::create_gte_block(config),
        "LTE" => base::create_lte_block(config),
        "EQ" => base::create_eq_block(config),
        "NEQ" => base::create_neq_block(config),
        
        // Timer blocks (always available)
        "ON_DELAY" => timer::create_on_delay_block(config),
        "OFF_DELAY" => timer::create_off_delay_block(config),
        "PULSE" => timer::create_pulse_block(config),
        
        // Math blocks (always available)
        "ADD" => arithmetic::create_add_block(config),
        "SUB" => arithmetic::create_subtract_block(config),
        "MUL" => arithmetic::create_multiply_block(config),
        "DIV" => arithmetic::create_divide_block(config),
        
        // Data blocks (always available)
        "SCALE" => data::create_scale_block(config),
        "LIMIT" => data::create_limit_block(config),
        "SELECT" => data::create_select_block(config),
        "MUX" => data::create_mux_block(config),
        "DEMUX" => data::create_demux_block(config),
        "DATA_GENERATOR" => data::create_data_generator_block(config),
        
        // Edge detection blocks (feature-gated)
        #[cfg(feature = "edge-detection")]
        "RISING_EDGE" => edge::create_rising_edge_block(config),
        #[cfg(feature = "edge-detection")]
        "FALLING_EDGE" => edge::create_falling_edge_block(config),
        #[cfg(feature = "edge-detection")]
        "CHANGE_DETECT" => edge::create_change_detect_block(config),
        
        // Memory blocks (feature-gated)
        #[cfg(feature = "memory-blocks")]
        "SR_LATCH" => memory::create_sr_latch_block(config),
        #[cfg(feature = "memory-blocks")]
        "D_FLIPFLOP" => memory::create_d_flipflop_block(config),
        #[cfg(feature = "memory-blocks")]
        "JK_FLIPFLOP" => memory::create_jk_flipflop_block(config),
        #[cfg(feature = "memory-blocks")]
        "T_FLIPFLOP" => memory::create_t_flipflop_block(config),
        
        // PID control blocks (feature-gated)
        #[cfg(feature = "pid-control")]
        "PID" => pid::create_pid_block(config),
        #[cfg(feature = "pid-control")]
        "TUNE_PID" => pid::create_tune_pid_block(config),
        
        // Communication blocks (feature-gated)
        #[cfg(feature = "communication")]
        "MODBUS_READ" => comm::create_modbus_read_block(config),
        #[cfg(feature = "communication")]
        "MODBUS_WRITE" => comm::create_modbus_write_block(config),
        #[cfg(feature = "communication")]
        "TCP_CLIENT" => comm::create_tcp_client_block(config),
        #[cfg(feature = "communication")]
        "UDP_SEND" => comm::create_udp_send_block(config),
        
        // State machine blocks (feature-gated)
        #[cfg(feature = "state-machine")]
        "STATE_MACHINE" => state::create_state_machine_block(config),
        #[cfg(feature = "state-machine")]
        "SEQUENCE" => state::create_sequence_block(config),
        
        // Advanced math blocks (feature-gated)
        #[cfg(feature = "advanced-math")]
        "FFT" => advanced_math::create_fft_block(config),
        #[cfg(feature = "advanced-math")]
        "FILTER" => advanced_math::create_filter_block(config),
        #[cfg(feature = "advanced-math")]
        "STATISTICS" => advanced_math::create_statistics_block(config),
        
        // Machine learning blocks (feature-gated)
        #[cfg(feature = "ml")]
        "ML_INFERENCE" => ml::create_ml_inference_block(config),
        #[cfg(feature = "ml")]
        "ANOMALY_DETECT" => ml::create_anomaly_detect_block(config),

        // SIMD math blocks (feature-gated)
        #[cfg(feature = "simd-math")]
        "SIMD_ARRAY_ADD" => {
            let input_a = get_input_signal(config, "a", true)?.unwrap();
            let input_b = get_input_signal(config, "b", true)?.unwrap();
            let output = get_output_signal(config, "out", true)?.unwrap();
            Ok(Box::new(simd_math::SimdArrayAdd::new(
                config.name.clone(),
                input_a,
                input_b,
                output,
            )))
        }
        
        _ => Err(PlcError::Config(format!(
            "Unknown block type: '{}'. Available types: {}",
            config.block_type,
            get_available_block_types().join(", ")
        ))),
    }
}

/// Get list of all available block types
pub fn get_available_block_types() -> Vec<&'static str> {
    let types = vec![
        // Always available
        "AND", "OR", "NOT", "XOR",
        "GT", "LT", "GTE", "LTE", "EQ", "NEQ",
        "ON_DELAY", "OFF_DELAY", "PULSE",
        "ADD", "SUB", "MUL", "DIV",
        "SCALE", "LIMIT", "SELECT", "MUX", "DEMUX", "DATA_GENERATOR",
        #[cfg(feature = "edge-detection")]
        "RISING_EDGE",
        #[cfg(feature = "edge-detection")]
        "FALLING_EDGE",
        #[cfg(feature = "edge-detection")]
        "CHANGE_DETECT",
        #[cfg(feature = "memory-blocks")]
        "SR_LATCH",
        #[cfg(feature = "memory-blocks")]
        "D_FLIPFLOP",
        #[cfg(feature = "memory-blocks")]
        "JK_FLIPFLOP",
        #[cfg(feature = "memory-blocks")]
        "T_FLIPFLOP",
        #[cfg(feature = "pid-control")]
        "PID",
        #[cfg(feature = "pid-control")]
        "TUNE_PID",
        #[cfg(feature = "communication")]
        "MODBUS_READ",
        #[cfg(feature = "communication")]
        "MODBUS_WRITE",
        #[cfg(feature = "communication")]
        "TCP_CLIENT",
        #[cfg(feature = "communication")]
        "UDP_SEND",
        #[cfg(feature = "state-machine")]
        "STATE_MACHINE",
        #[cfg(feature = "state-machine")]
        "SEQUENCE",
        #[cfg(feature = "advanced-math")]
        "FFT",
        #[cfg(feature = "advanced-math")]
        "FILTER",
        #[cfg(feature = "advanced-math")]
        "STATISTICS",
        #[cfg(feature = "ml")]
        "ML_INFERENCE", 
        #[cfg(feature = "ml")]
        "ANOMALY_DETECT",
        #[cfg(feature = "simd-math")]
        "SIMD_ARRAY_ADD",
    ];

    types
}

/// Enhanced monitoring metadata for blocks
#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockMetadata {
    /// Block type identifier
    pub block_type: String,
    /// Block category
    pub category: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Tags for filtering and organization
    pub tags: Vec<String>,
    /// Input signal definitions
    pub inputs: Vec<SignalDefinition>,
    /// Output signal definitions
    pub outputs: Vec<SignalDefinition>,
    /// Parameter definitions
    pub parameters: HashMap<String, ParameterDefinition>,
}

/// Signal definition for block metadata
#[cfg(any(feature = "enhanced-monitoring", feature = "json-schema"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct SignalDefinition {
    pub name: String,
    pub signal_type: String,
    pub required: bool,
    pub description: Option<String>,
}

/// Parameter definition for block metadata
#[cfg(any(feature = "enhanced-monitoring", feature = "json-schema"))]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct ParameterDefinition {
    pub parameter_type: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: Option<String>,
    pub validation: Option<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Validate block configuration has required signals
pub fn validate_block_signals(config: &BlockConfig, available_signals: &HashMap<&String, &crate::value::ValueType>) -> Result<()> {
    // Validate input signals
    for (input_name, signal_name) in &config.inputs {
        if !available_signals.contains_key(signal_name) {
            return Err(PlcError::Config(format!(
                "Block '{}' input '{}' references unknown signal '{}'",
                config.name, input_name, signal_name
            )));
        }
    }
    
    // Validate output signals
    for (output_name, signal_name) in &config.outputs {
        if !available_signals.contains_key(signal_name) {
            return Err(PlcError::Config(format!(
                "Block '{}' output '{}' references unknown signal '{}'",
                config.name, output_name, signal_name
            )));
        }
    }
    
    Ok(())
}

/// Extract parameter from block configuration with type checking
pub fn get_parameter<T>(config: &BlockConfig, param_name: &str, default: Option<T>) -> Result<T>
where
    T: serde::de::DeserializeOwned + Clone,
{
    match config.params.get(param_name) {  // Fixed: using params instead of parameters
        Some(value) => {
            serde_yaml::from_value(value.clone())
                .map_err(|e| PlcError::Config(format!(
                    "Block '{}' parameter '{}' invalid: {}",
                    config.name, param_name, e
                )))
        }
        None => {
            if let Some(default_value) = default {
                Ok(default_value)
            } else {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required parameter '{}'",
                    config.name, param_name
                )))
            }
        }
    }
}

/// Helper to get string parameter
pub fn get_string_parameter(config: &BlockConfig, param_name: &str, default: Option<&str>) -> Result<String> {
    match config.params.get(param_name) {  // Fixed: using params instead of parameters
        Some(serde_yaml::Value::String(s)) => Ok(s.clone()),
        Some(value) => {
            // Convert serde_yaml::Value to string
            match value {
                serde_yaml::Value::String(s) => Ok(s.clone()),
                serde_yaml::Value::Number(n) => Ok(n.to_string()),
                serde_yaml::Value::Bool(b) => Ok(b.to_string()),
                _ => Ok(format!("{:?}", value)),
            }
        }
        None => {
            if let Some(default_value) = default {
                Ok(default_value.to_string())
            } else {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required string parameter '{}'",
                    config.name, param_name
                )))
            }
        }
    }
}

/// Helper to get numeric parameter
pub fn get_numeric_parameter<T>(config: &BlockConfig, param_name: &str, default: Option<T>) -> Result<T>
where
    T: serde::de::DeserializeOwned + std::str::FromStr + Clone,
    T::Err: std::fmt::Display,
{
    match config.params.get(param_name) {  // Fixed: using params instead of parameters
        Some(serde_yaml::Value::Number(n)) => {
            serde_yaml::from_value(serde_yaml::Value::Number(n.clone()))
                .map_err(|e| PlcError::Config(format!(
                    "Block '{}' parameter '{}' invalid number: {}",
                    config.name, param_name, e
                )))
        }
        Some(serde_yaml::Value::String(s)) => {
            s.parse::<T>()
                .map_err(|e| PlcError::Config(format!(
                    "Block '{}' parameter '{}' cannot parse '{}': {}",
                    config.name, param_name, s, e
                )))
        }
        Some(_) => Err(PlcError::Config(format!(
            "Block '{}' parameter '{}' must be a number",
            config.name, param_name
        ))),
        None => {
            if let Some(default_value) = default {
                Ok(default_value)
            } else {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required numeric parameter '{}'",
                    config.name, param_name
                )))
            }
        }
    }
}

/// Helper to get boolean parameter
pub fn get_bool_parameter(config: &BlockConfig, param_name: &str, default: Option<bool>) -> Result<bool> {
    match config.params.get(param_name) {  // Fixed: using params instead of parameters
        Some(serde_yaml::Value::Bool(b)) => Ok(*b),  // Fixed: dereferencing the bool
        Some(serde_yaml::Value::String(s)) => {
            match s.to_lowercase().as_str() {
                "true" | "yes" | "on" | "1" => Ok(true),
                "false" | "no" | "off" | "0" => Ok(false),
                _ => Err(PlcError::Config(format!(
                    "Block '{}' parameter '{}' invalid boolean value: '{}'",
                    config.name, param_name, s
                ))),
            }
        }
        Some(_) => Err(PlcError::Config(format!(
            "Block '{}' parameter '{}' must be a boolean",
            config.name, param_name
        ))),
        None => {
            if let Some(default_value) = default {
                Ok(default_value)
            } else {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required boolean parameter '{}'",
                    config.name, param_name
                )))
            }
        }
    }
}

/// Helper to get array parameter
pub fn get_array_parameter<T>(config: &BlockConfig, param_name: &str, default: Option<Vec<T>>) -> Result<Vec<T>>
where
    T: serde::de::DeserializeOwned + Clone,
{
    match config.params.get(param_name) {  // Fixed: using params instead of parameters
        Some(serde_yaml::Value::Sequence(seq)) => {
            let mut result = Vec::with_capacity(seq.len());
            for (i, item) in seq.iter().enumerate() {
                match serde_yaml::from_value(item.clone()) {
                    Ok(value) => result.push(value),
                    Err(e) => return Err(PlcError::Config(format!(
                        "Block '{}' parameter '{}' array item {} invalid: {}",
                        config.name, param_name, i, e
                    ))),
                }
            }
            Ok(result)
        }
        Some(_) => Err(PlcError::Config(format!(
            "Block '{}' parameter '{}' must be an array",
            config.name, param_name
        ))),
        None => {
            if let Some(default_value) = default {
                Ok(default_value)
            } else {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required array parameter '{}'",
                    config.name, param_name
                )))
            }
        }
    }
}

/// Helper to get input signal path
pub fn get_input_signal(config: &BlockConfig, input_name: &str, required: bool) -> Result<Option<String>> {
    match config.inputs.get(input_name) {
        Some(signal_path) => Ok(Some(signal_path.clone())),
        None => {
            if required {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required input '{}'",
                    config.name, input_name
                )))
            } else {
                Ok(None)
            }
        }
    }
}

/// Helper to get output signal path
pub fn get_output_signal(config: &BlockConfig, output_name: &str, required: bool) -> Result<Option<String>> {
    match config.outputs.get(output_name) {
        Some(signal_path) => Ok(Some(signal_path.clone())),
        None => {
            if required {
                Err(PlcError::Config(format!(
                    "Block '{}' missing required output '{}'",
                    config.name, output_name
                )))
            } else {
                Ok(None)
            }
        }
    }
}

/// Helper to get the first/primary input signal
pub fn get_primary_input(config: &BlockConfig) -> Result<String> {
    config
        .inputs
        .values()
        .min()
        .cloned()
        .ok_or_else(|| PlcError::Config(format!(
            "Block '{}' has no input signals",
            config.name
        )))
}

/// Helper to get the first/primary output signal
pub fn get_primary_output(config: &BlockConfig) -> Result<String> {
    config
        .outputs
        .values()
        .min()
        .cloned()
        .ok_or_else(|| PlcError::Config(format!(
            "Block '{}' has no output signals",
            config.name
        )))
}

// ============================================================================
// TEST UTILITIES
// ============================================================================

#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    /// Create a test block configuration
    pub fn create_test_config(block_type: &str, name: &str) -> BlockConfig {
        let config = BlockConfig {
            name: name.to_string(),
            block_type: block_type.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            params: HashMap::new(),
            description: Some(format!("Test {} block", block_type)),
            tags: vec!["test".to_string()],
        };

        config
    }
    
    /// Helper to create block config with inputs/outputs
    pub fn create_test_config_with_io(
        block_type: &str,
        name: &str,
        inputs: Vec<(&str, &str)>,
        outputs: Vec<(&str, &str)>,
    ) -> BlockConfig {
        let mut config = create_test_config(block_type, name);
        
        for (input_name, signal_name) in inputs {
            config.inputs.insert(input_name.to_string(), signal_name.to_string());
        }
        
        for (output_name, signal_name) in outputs {
            config.outputs.insert(output_name.to_string(), signal_name.to_string());
        }
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_factory() {
        let config = test_utils::create_test_config_with_io(
            "AND",
            "test_and",
            vec![("a", "in1")],
            vec![("out", "out1")],
        );
        let block = create_block(&config).unwrap();
        assert_eq!(block.block_type(), "AND");
        assert_eq!(block.name(), "test_and");
    }
    
    #[test]
    fn test_unknown_block_type() {
        let config = test_utils::create_test_config("UNKNOWN", "test_unknown");
        let result = create_block(&config);
        assert!(result.is_err());
        
        if let Err(PlcError::Config(msg)) = result {
            assert!(msg.contains("Unknown block type"));
            assert!(msg.contains("UNKNOWN"));
        }
    }
    
    #[test]
    fn test_available_block_types() {
        let types = get_available_block_types();
        assert!(types.contains(&"AND"));
        assert!(types.contains(&"OR"));
        assert!(types.contains(&"NOT"));
        assert!(types.contains(&"ADD"));
        assert!(types.contains(&"ON_DELAY"));
    }
    
    #[test]
    fn test_parameter_extraction() {
        let mut config = test_utils::create_test_config("TEST", "test");
        config.params.insert("delay".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(1000)));
        
        let delay: u64 = get_numeric_parameter(&config, "delay", None).unwrap();
        assert_eq!(delay, 1000);
        
        // Test with default
        let missing: u64 = get_numeric_parameter(&config, "missing", Some(500)).unwrap();
        assert_eq!(missing, 500);
    }
    
    #[test]
    fn test_boolean_parameter() {
        let mut config = test_utils::create_test_config("TEST", "test");
        config.params.insert("enabled".to_string(), serde_yaml::Value::Bool(true));
        config.params.insert("string_true".to_string(), serde_yaml::Value::String("yes".to_string()));
        config.params.insert("string_false".to_string(), serde_yaml::Value::String("no".to_string()));
        
        assert_eq!(get_bool_parameter(&config, "enabled", None).unwrap(), true);
        assert_eq!(get_bool_parameter(&config, "string_true", None).unwrap(), true);
        assert_eq!(get_bool_parameter(&config, "string_false", None).unwrap(), false);
        assert_eq!(get_bool_parameter(&config, "missing", Some(false)).unwrap(), false);
    }
    
    #[test]
    fn test_string_parameter() {
        let mut config = test_utils::create_test_config("TEST", "test");
        config.params.insert("mode".to_string(), serde_yaml::Value::String("fast".to_string()));
        config.params.insert("count".to_string(), serde_yaml::Value::Number(serde_yaml::Number::from(42)));
        
        assert_eq!(get_string_parameter(&config, "mode", None).unwrap(), "fast");
        assert_eq!(get_string_parameter(&config, "count", None).unwrap(), "42");
        assert_eq!(get_string_parameter(&config, "missing", Some("default")).unwrap(), "default");
    }
    
    #[test]
    fn test_signal_helpers() {
        let config = test_utils::create_test_config_with_io(
            "TEST", "test",
            vec![("in1", "signal.input1"), ("in2", "signal.input2")],
            vec![("out", "signal.output")]
        );
        
        assert_eq!(get_input_signal(&config, "in1", true).unwrap().unwrap(), "signal.input1");
        assert_eq!(get_output_signal(&config, "out", true).unwrap().unwrap(), "signal.output");
        assert!(get_input_signal(&config, "missing", false).unwrap().is_none());
        assert!(get_input_signal(&config, "missing", true).is_err());
        
        assert_eq!(get_primary_input(&config).unwrap(), "signal.input1");
        assert_eq!(get_primary_output(&config).unwrap(), "signal.output");
    }
}

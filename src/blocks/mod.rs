// src/blocks/mod.rs - Updated block system with complete factory
use crate::{error::*, signal::SignalBus, config::BlockConfig, value::Value};
use std::collections::HashMap;

#[cfg(feature = "async-blocks")]
use async_trait::async_trait;

#[cfg(feature = "circuit-breaker")]
use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};

#[cfg(feature = "circuit-breaker")]
use parking_lot::RwLock;

#[cfg(feature = "circuit-breaker")]
use std::time::{Instant, Duration};

#[cfg(feature = "enhanced-monitoring")]
use std::time::Duration as MonitoringDuration;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// Import all block modules
mod base;
mod timer;
mod math;
mod data;

#[cfg(feature = "edge-detection")]
mod edge;

#[cfg(feature = "memory-blocks")]
mod memory;

#[cfg(feature = "pid-control")]
mod pid;

#[cfg(feature = "statistics")]
mod statistics;

#[cfg(feature = "email")]
mod email;

#[cfg(feature = "twilio")]
mod sms;

#[cfg(feature = "web")]
mod web;

// ============================================================================
// CORE BLOCK TRAIT
// ============================================================================

/// Core trait that all blocks must implement
/// 
/// Blocks are the fundamental processing units in PETRA, executing logic
/// operations on signals from the signal bus.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::blocks::{Block, BlockConfig};
/// use petra::{SignalBus, Value, Result};
/// 
/// struct MyBlock {
///     name: String,
///     config: BlockConfig,
/// }
/// 
/// impl Block for MyBlock {
///     fn execute(&mut self, bus: &SignalBus) -> Result<()> {
///         // Read inputs
///         let input = bus.get_bool("my_input")?;
///         
///         // Process
///         let output = !input;
///         
///         // Write outputs
///         bus.set("my_output", Value::Bool(output))?;
///         
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
    fn execute(&mut self, bus: &SignalBus) -> Result<()>;
    
    /// Get the block instance name
    fn name(&self) -> &str;
    
    /// Get the block type identifier
    fn block_type(&self) -> &str;
    
    // Optional trait methods with default implementations
    
    /// Get the block category
    fn category(&self) -> &str {
        "core"
    }
    
    /// Validate block configuration
    fn validate_config(config: &BlockConfig) -> Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }
    
    /// Initialize the block
    fn initialize(&mut self, config: &BlockConfig) -> Result<()> {
        Ok(())
    }
    
    /// Reset the block to initial state
    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
    
    /// Get block description
    fn description(&self) -> Option<&str> {
        None
    }
    
    /// Get block tags
    fn tags(&self) -> Vec<&str> {
        Vec::new()
    }
    
    // Enhanced monitoring support
    #[cfg(feature = "enhanced-monitoring")]
    /// Get last execution time
    fn last_execution_time(&self) -> Option<MonitoringDuration> {
        None
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get execution count
    fn execution_count(&self) -> u64 {
        0
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get current state for debugging
    fn state(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get block metadata
    fn metadata(&self) -> Option<BlockMetadata> {
        None
    }
}

// ============================================================================
// BLOCK FACTORY
// ============================================================================

/// Create a block instance from configuration
/// 
/// This is the main factory function that creates blocks based on their type.
/// All block types must be registered here.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::blocks::{create_block, BlockConfig};
/// use std::collections::HashMap;
/// 
/// let config = BlockConfig {
///     name: "my_and_gate".to_string(),
///     block_type: "AND".to_string(),
///     inputs: HashMap::new(),
///     outputs: HashMap::new(),
///     parameters: HashMap::new(),
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
        "ADD" => math::create_add_block(config),
        "SUB" => math::create_sub_block(config),
        "MUL" => math::create_mul_block(config),
        "DIV" => math::create_div_block(config),
        
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
        "COUNTER" => memory::create_counter_block(config),
        
        // PID control (feature-gated)
        #[cfg(feature = "pid-control")]
        "PID" => pid::create_pid_block(config),
        
        // Statistics blocks (feature-gated)
        #[cfg(feature = "statistics")]
        "AVERAGE" => statistics::create_average_block(config),
        #[cfg(feature = "statistics")]
        "MIN_MAX" => statistics::create_min_max_block(config),
        #[cfg(feature = "statistics")]
        "STATISTICS" => statistics::create_statistics_block(config),
        
        // Communication blocks (feature-gated)
        #[cfg(feature = "email")]
        "EMAIL_SEND" => email::create_email_send_block(config),
        
        #[cfg(feature = "twilio")]
        "SMS_SEND" => sms::create_sms_send_block(config),
        
        #[cfg(feature = "web")]
        "HTTP_REQUEST" => web::create_http_request_block(config),
        #[cfg(feature = "web")]
        "REST_API" => web::create_rest_api_block(config),
        
        // Unknown block type
        _ => Err(PlcError::Config(format!(
            "Unknown block type: '{}'. Available types: {}",
            config.block_type,
            get_available_block_types().join(", ")
        ))),
    }
}

/// Get list of available block types
pub fn get_available_block_types() -> Vec<&'static str> {
    let mut types = vec![
        // Always available
        "AND", "OR", "NOT", "XOR",
        "GT", "LT", "GTE", "LTE", "EQ", "NEQ",
        "ON_DELAY", "OFF_DELAY", "PULSE",
        "ADD", "SUB", "MUL", "DIV",
        "SCALE", "LIMIT", "SELECT", "MUX", "DEMUX", "DATA_GENERATOR",
    ];
    
    #[cfg(feature = "edge-detection")]
    types.extend(&["RISING_EDGE", "FALLING_EDGE", "CHANGE_DETECT"]);
    
    #[cfg(feature = "memory-blocks")]
    types.extend(&["SR_LATCH", "D_FLIPFLOP", "COUNTER"]);
    
    #[cfg(feature = "pid-control")]
    types.push("PID");
    
    #[cfg(feature = "statistics")]
    types.extend(&["AVERAGE", "MIN_MAX", "STATISTICS"]);
    
    #[cfg(feature = "email")]
    types.push("EMAIL_SEND");
    
    #[cfg(feature = "twilio")]
    types.push("SMS_SEND");
    
    #[cfg(feature = "web")]
    types.extend(&["HTTP_REQUEST", "REST_API"]);
    
    types
}

// ============================================================================
// BLOCK METADATA (for enhanced monitoring)
// ============================================================================

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
    match config.params.get(param_name) {
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
    match config.params.get(param_name) {
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
    match config.params.get(param_name) {
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
    match config.params.get(param_name) {
        Some(serde_yaml::Value::Bool(b)) => Ok(*b),
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

// ============================================================================
// ASYNC BLOCK TRAIT (feature-gated)
// ============================================================================

/// Asynchronous block trait for blocks that need async operations
#[cfg(feature = "async-blocks")]
#[async_trait]
pub trait AsyncBlock: Send + Sync {
    /// Execute the block logic asynchronously
    async fn execute_async(&mut self, bus: &SignalBus) -> Result<()>;
    
    /// Get the block instance name
    fn name(&self) -> &str;
    
    /// Get the block type identifier
    fn block_type(&self) -> &str;
}

// ============================================================================
// CIRCUIT BREAKER (feature-gated)
// ============================================================================

#[cfg(feature = "circuit-breaker")]
/// Circuit breaker for enhanced error handling
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    max_failures: u32,
    reset_timeout: Duration,
    last_attempt: RwLock<Instant>,
    circuit_open: AtomicBool,
    half_open_calls: AtomicU32,
}

#[cfg(feature = "circuit-breaker")]
impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(max_failures: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            max_failures,
            reset_timeout,
            last_attempt: RwLock::new(Instant::now()),
            circuit_open: AtomicBool::new(false),
            half_open_calls: AtomicU32::new(0),
        }
    }
    
    /// Execute block with circuit breaker protection
    pub fn execute(&self, block: &mut dyn Block, bus: &SignalBus) -> Result<()> {
        // Check if circuit is open
        if self.circuit_open.load(Ordering::Relaxed) {
            let elapsed = self.last_attempt.read().elapsed();
            
            if elapsed < self.reset_timeout {
                return Err(PlcError::CircuitOpen);
            }
            
            // Try half-open state
            if self.half_open_calls.fetch_add(1, Ordering::Relaxed) > 3 {
                // Too many half-open attempts, keep circuit open
                self.half_open_calls.store(0, Ordering::Relaxed);
                *self.last_attempt.write() = Instant::now();
                return Err(PlcError::CircuitOpen);
            }
        }
        
        // Execute the block
        match block.execute(bus) {
            Ok(()) => {
                // Reset on success
                self.failure_count.store(0, Ordering::Relaxed);
                self.circuit_open.store(false, Ordering::Relaxed);
                self.half_open_calls.store(0, Ordering::Relaxed);
                Ok(())
            }
            Err(e) => {
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                *self.last_attempt.write() = Instant::now();
                
                if failures >= self.max_failures {
                    self.circuit_open.store(true, Ordering::Relaxed);
                    tracing::error!(
                        "Circuit breaker opened for block '{}' after {} failures", 
                        block.name(), failures
                    );
                    
                    return Err(PlcError::CircuitOpen);
                }
                
                Err(e)
            }
        }
    }
    
    /// Get circuit breaker status
    pub fn status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            circuit_open: self.circuit_open.load(Ordering::Relaxed),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            max_failures: self.max_failures,
            last_failure: *self.last_attempt.read(),
            reset_timeout: self.reset_timeout,
        }
    }
    
    /// Manually reset the circuit breaker
    pub fn reset(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.circuit_open.store(false, Ordering::Relaxed);
        self.half_open_calls.store(0, Ordering::Relaxed);
    }
}

/// Circuit breaker status information
#[cfg(feature = "circuit-breaker")]
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatus {
    pub circuit_open: bool,
    pub failure_count: u32,
    pub max_failures: u32,
    pub last_failure: Instant,
    pub reset_timeout: Duration,
}

// ============================================================================
// TEST UTILITIES
// ============================================================================

#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    /// Create a test block configuration
    pub fn create_test_config(block_type: &str, name: &str) -> BlockConfig {
        BlockConfig {
            name: name.to_string(),
            block_type: block_type.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            parameters: HashMap::new(),
            description: Some(format!("Test {} block", block_type)),
            tags: vec!["test".to_string()],
        }
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
        let config = test_utils::create_test_config("AND", "test_and");
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
}

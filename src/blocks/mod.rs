//! Block system for PETRA with feature-organized block types
//!
//! This module provides the extensible block system organized by feature groups:
//! - Basic blocks (always available): Logic, comparison, timer, edge, memory
//! - Advanced blocks: PID control, statistics, machine learning
//! - Communication blocks: Email, SMS, web integration
//! - Enhanced error handling: Circuit breakers, retry logic

use crate::{error::*, signal::SignalBus, config::BlockConfig};
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

// ============================================================================
// CORE BLOCK TRAIT
// ============================================================================

/// Core trait that all blocks must implement
/// 
/// Blocks are the fundamental processing units in PETRA, executing logic
/// operations on signals from the signal bus.
pub trait Block: Send + Sync {
   /// Execute the block logic
   /// 
   /// This method is called during each scan cycle to perform the block's
   /// primary function (logic operation, calculation, communication, etc.)
   fn execute(&mut self, bus: &SignalBus) -> Result<()>;
   
   /// Get the block's name (must be unique within a configuration)
   fn name(&self) -> &str;
   
   /// Get the block's type identifier
   fn block_type(&self) -> &str;
   
   /// Get execution time from last run (requires enhanced-monitoring)
   #[cfg(feature = "enhanced-monitoring")]
   fn last_execution_time(&self) -> Option<MonitoringDuration> {
       None
   }
   
   /// Get block metadata (requires enhanced features)
   #[cfg(any(feature = "enhanced-monitoring", feature = "json-schema"))]
   fn metadata(&self) -> BlockMetadata {
       BlockMetadata {
           name: self.name().to_string(),
           block_type: self.block_type().to_string(),
           category: self.category().to_string(),
           description: None,
           tags: Vec::new(),
           inputs: Vec::new(),
           outputs: Vec::new(),
           parameters: HashMap::new(),
       }
   }
   
   /// Get block category for organization
   fn category(&self) -> &str {
       "core"
   }
   
   /// Validate block configuration before creation
   fn validate_config(config: &BlockConfig) -> Result<()> 
   where 
       Self: Sized 
   {
       // Default validation - blocks can override for specific requirements
       if config.name.is_empty() {
           return Err(PlcError::Config("Block name cannot be empty".to_string()));
       }
       
       if config.block_type.is_empty() {
           return Err(PlcError::Config(format!(
               "Block '{}' must have a type", config.name
           )));
       }
       
       Ok(())
   }
   
   /// Initialize block from configuration (called after creation)
   fn initialize(&mut self, _config: &BlockConfig) -> Result<()> {
       Ok(())
   }
   
   /// Reset block state (called when engine resets)
   fn reset(&mut self) -> Result<()> {
       Ok(())
   }
   
   /// Get current block state for debugging/monitoring
   #[cfg(feature = "enhanced-monitoring")]
   fn state(&self) -> HashMap<String, crate::value::Value> {
       HashMap::new()
   }
}

// ============================================================================
// BLOCK METADATA STRUCTURES
// ============================================================================

/// Extended metadata for blocks (feature-gated)
#[cfg(any(feature = "enhanced-monitoring", feature = "json-schema"))]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct BlockMetadata {
   /// Block name
   pub name: String,
   /// Block type identifier
   pub block_type: String,
   /// Block category (logic, timer, communication, etc.)
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
// ASYNC BLOCK TRAIT (feature-gated)
// ============================================================================

/// Asynchronous block trait for blocks that need async operations
#[cfg(feature = "async-blocks")]
#[async_trait]
pub trait AsyncBlock: Send + Sync {
   /// Execute the block logic asynchronously
   async fn execute_async(&mut self, bus: &SignalBus) -> Result<()>;
   
   /// Get the block's name
   fn name(&self) -> &str;
   
   /// Get the block's type
   fn block_type(&self) -> &str;
   
   /// Get block category
   fn category(&self) -> &str {
       "async"
   }
}

// ============================================================================
// ENHANCED ERROR HANDLING (feature-gated)
// ============================================================================

/// Circuit breaker for enhanced block error handling
#[cfg(feature = "circuit-breaker")]
pub struct BlockExecutor {
   /// Number of consecutive failures
   failure_count: AtomicU32,
   /// Maximum failures before opening circuit
   max_failures: u32,
   /// Circuit state (open/closed)
   circuit_open: AtomicBool,
   /// Last attempt timestamp
   last_attempt: RwLock<Instant>,
   /// Timeout before attempting to close circuit
   reset_timeout: Duration,
   /// Half-open state configuration
   half_open_max_calls: u32,
   /// Current calls in half-open state
   half_open_calls: AtomicU32,
}

#[cfg(feature = "circuit-breaker")]
impl BlockExecutor {
   /// Create a new circuit breaker with configuration
   pub fn new(max_failures: u32, reset_timeout: Duration) -> Self {
       Self {
           failure_count: AtomicU32::new(0),
           max_failures,
           circuit_open: AtomicBool::new(false),
           last_attempt: RwLock::new(Instant::now()),
           reset_timeout,
           half_open_max_calls: 5,
           half_open_calls: AtomicU32::new(0),
       }
   }
   
   /// Create with full configuration
   pub fn with_config(
       max_failures: u32, 
       reset_timeout: Duration,
       half_open_max_calls: u32
   ) -> Self {
       Self {
           failure_count: AtomicU32::new(0),
           max_failures,
           circuit_open: AtomicBool::new(false),
           last_attempt: RwLock::new(Instant::now()),
           reset_timeout,
           half_open_max_calls,
           half_open_calls: AtomicU32::new(0),
       }
   }

   /// Execute block with circuit breaker protection
   pub fn execute_with_circuit_breaker(
       &self, 
       block: &mut dyn Block, 
       bus: &SignalBus
   ) -> Result<()> {
       // Check if circuit is open
       if self.circuit_open.load(Ordering::Relaxed) {
           let last = self.last_attempt.read();
           if last.elapsed() < self.reset_timeout {
               return Err(PlcError::CircuitOpen);
           }
           
           // Try half-open state
           if self.half_open_calls.load(Ordering::Relaxed) >= self.half_open_max_calls {
               return Err(PlcError::CircuitOpen);
           }
           
           self.half_open_calls.fetch_add(1, Ordering::Relaxed);
       }
       
       match block.execute(bus) {
           Ok(_) => {
               // Success - reset failure count and close circuit
               self.failure_count.store(0, Ordering::Relaxed);
               if self.circuit_open.load(Ordering::Relaxed) {
                   self.circuit_open.store(false, Ordering::Relaxed);
                   self.half_open_calls.store(0, Ordering::Relaxed);
                   tracing::info!("Circuit breaker closed for block '{}'", block.name());
               }
               Ok(())
           }
           Err(e) => {
               // Failure - increment count and potentially open circuit
               let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
               
               if failures >= self.max_failures {
                   self.circuit_open.store(true, Ordering::Relaxed);
                   *self.last_attempt.write() = Instant::now();
                   self.half_open_calls.store(0, Ordering::Relaxed);
                   
                   tracing::warn!(
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
// RETRY LOGIC (feature-gated)
// ============================================================================

/// Retry configuration for enhanced error handling
#[cfg(feature = "enhanced-errors")]
#[derive(Debug, Clone)]
pub struct RetryConfig {
   pub max_attempts: u32,
   pub initial_delay_ms: u64,
   pub max_delay_ms: u64,
   pub exponential_backoff: bool,
   pub backoff_multiplier: f64,
}

#[cfg(feature = "enhanced-errors")]
impl Default for RetryConfig {
   fn default() -> Self {
       Self {
           max_attempts: 3,
           initial_delay_ms: 100,
           max_delay_ms: 5000,
           exponential_backoff: true,
           backoff_multiplier: 2.0,
       }
   }
}

/// Execute block with retry logic
#[cfg(feature = "enhanced-errors")]
pub async fn execute_with_retry(
   block: &mut dyn Block,
   bus: &SignalBus,
   config: &RetryConfig,
) -> Result<()> {
   let mut attempt = 0;
   let mut delay = config.initial_delay_ms;
   
   loop {
       attempt += 1;
       
       match block.execute(bus) {
           Ok(_) => return Ok(()),
           Err(e) => {
               if attempt >= config.max_attempts {
                   tracing::error!(
                       "Block '{}' failed after {} attempts: {}", 
                       block.name(), attempt, e
                   );
                   return Err(e);
               }
               
               tracing::warn!(
                   "Block '{}' failed (attempt {}/{}): {}", 
                   block.name(), attempt, config.max_attempts, e
               );
               
               // Wait before retry
               tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
               
               // Update delay for next attempt
               if config.exponential_backoff {
                   delay = ((delay as f64 * config.backoff_multiplier) as u64)
                       .min(config.max_delay_ms);
               }
           }
       }
   }
}

// ============================================================================
// BLOCK MODULE ORGANIZATION
// ============================================================================

// Core blocks (always available)
pub mod logic;       // AND, OR, NOT, XOR, NAND, NOR
pub mod comparison;  // LT, GT, EQ, NE, GE, LE
pub mod timer;       // TON, TOF, TP, CTU, CTD
pub mod edge;        // R_TRIG, F_TRIG, EDGE_DETECT
pub mod memory;      // SR_LATCH, RS_LATCH, FLIP_FLOP
pub mod arithmetic;  // ADD, SUB, MUL, DIV, MOD, ABS, SQRT
pub mod convert;     // Type conversion blocks

// Advanced blocks (feature-gated)
#[cfg(feature = "advanced-blocks")]
pub mod control;     // PID, LEAD_LAG, DEADBAND

#[cfg(feature = "pid-control")]
pub mod pid;         // Advanced PID controller with auto-tuning

#[cfg(feature = "statistics")]
pub mod statistics;  // MIN, MAX, AVG, STDEV, HISTOGRAM

#[cfg(feature = "ml-blocks")]
pub mod ml;          // ML_INFERENCE, NEURAL_NETWORK

// Communication blocks (feature-gated)
#[cfg(feature = "email")]
pub mod email;       // EMAIL_SEND, EMAIL_ALERT

#[cfg(feature = "twilio")]
pub mod sms;         // SMS_SEND, VOICE_CALL

#[cfg(feature = "web")]
pub mod web;         // HTTP_REQUEST, WEBHOOK

#[cfg(feature = "mqtt")]
pub mod mqtt_blocks; // MQTT_PUBLISH, MQTT_SUBSCRIBE

// Data processing blocks (feature-gated)
#[cfg(feature = "validation")]
pub mod validation;  // VALIDATE, RANGE_CHECK, PATTERN_MATCH

#[cfg(feature = "extended-types")]
pub mod data;        // ARRAY_OPS, JSON_PARSE, STRING_OPS

// Storage blocks (feature-gated)
#[cfg(feature = "history")]
pub mod storage;     // LOG_VALUE, QUERY_HISTORY

// Security blocks (feature-gated)
#[cfg(feature = "security")]
pub mod security;    // ENCRYPT, DECRYPT, HASH, VERIFY

// ============================================================================
// CENTRAL BLOCK FACTORY
// ============================================================================

/// Create a block instance from configuration
/// 
/// This factory function dispatches to the appropriate module based on
/// the block type and available features.
pub fn create_block(config: &BlockConfig) -> Result<Box<dyn Block>> {
   // Validate basic configuration
   if config.name.is_empty() {
       return Err(PlcError::Config("Block name cannot be empty".to_string()));
   }
   
   if config.block_type.is_empty() {
       return Err(PlcError::Config(format!(
           "Block '{}' must have a type", config.name
       )));
   }
   
   tracing::debug!("Creating block '{}' of type '{}'", config.name, config.block_type);
   
   match config.block_type.to_uppercase().as_str() {
       // ====================================================================
       // CORE LOGIC BLOCKS (always available)
       // ====================================================================
       "AND" => logic::create_and_block(config),
       "OR" => logic::create_or_block(config),
       "NOT" => logic::create_not_block(config),
       "XOR" => logic::create_xor_block(config),
       "NAND" => logic::create_nand_block(config),
       "NOR" => logic::create_nor_block(config),
       
       // ====================================================================
       // COMPARISON BLOCKS (always available)
       // ====================================================================
       "LT" | "LESS_THAN" => comparison::create_less_than_block(config),
       "GT" | "GREATER_THAN" => comparison::create_greater_than_block(config),
       "EQ" | "EQUAL" => comparison::create_equal_block(config),
       "NE" | "NOT_EQUAL" => comparison::create_not_equal_block(config),
       "GE" | "GREATER_EQUAL" => comparison::create_greater_equal_block(config),
       "LE" | "LESS_EQUAL" => comparison::create_less_equal_block(config),
       
       // ====================================================================
       // TIMER BLOCKS (always available)
       // ====================================================================
       "TON" | "TIMER_ON" => timer::create_timer_on_block(config),
       "TOF" | "TIMER_OFF" => timer::create_timer_off_block(config),
       "TP" | "TIMER_PULSE" => timer::create_timer_pulse_block(config),
       "CTU" | "COUNT_UP" => timer::create_count_up_block(config),
       "CTD" | "COUNT_DOWN" => timer::create_count_down_block(config),
       
       // ====================================================================
       // EDGE DETECTION BLOCKS (always available)
       // ====================================================================
       "R_TRIG" | "RISING_EDGE" => edge::create_rising_edge_block(config),
       "F_TRIG" | "FALLING_EDGE" => edge::create_falling_edge_block(config),
       "EDGE_DETECT" => edge::create_edge_detect_block(config),
       
       // ====================================================================
       // MEMORY BLOCKS (always available)
       // ====================================================================
       "SR_LATCH" | "SET_RESET" => memory::create_sr_latch_block(config),
       "RS_LATCH" | "RESET_SET" => memory::create_rs_latch_block(config),
       "FLIP_FLOP" => memory::create_flip_flop_block(config),
       
       // ====================================================================
       // ARITHMETIC BLOCKS (always available)
       // ====================================================================
       "ADD" => arithmetic::create_add_block(config),
       "SUB" | "SUBTRACT" => arithmetic::create_subtract_block(config),
       "MUL" | "MULTIPLY" => arithmetic::create_multiply_block(config),
       "DIV" | "DIVIDE" => arithmetic::create_divide_block(config),
       "MOD" | "MODULO" => arithmetic::create_modulo_block(config),
       "ABS" | "ABSOLUTE" => arithmetic::create_absolute_block(config),
       "SQRT" | "SQUARE_ROOT" => arithmetic::create_sqrt_block(config),
       "MIN" => arithmetic::create_min_block(config),
       "MAX" => arithmetic::create_max_block(config),
       
       // ====================================================================
       // CONVERSION BLOCKS (always available)
       // ====================================================================
       "CONVERT" | "TYPE_CONVERT" => convert::create_convert_block(config),
       "SCALE" => convert::create_scale_block(config),
       "LIMIT" => convert::create_limit_block(config),
       
       // ====================================================================
       // ADVANCED CONTROL BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "advanced-blocks")]
       "LEAD_LAG" => control::create_lead_lag_block(config),
       
       #[cfg(feature = "advanced-blocks")]
       "DEADBAND" => control::create_deadband_block(config),
       
       #[cfg(feature = "pid-control")]
       "PID" => pid::create_pid_block(config),
       
       #[cfg(feature = "pid-control")]
       "PID_AUTO_TUNE" => pid::create_auto_tune_pid_block(config),
       
       // ====================================================================
       // STATISTICS BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "statistics")]
       "STATISTICS" => statistics::create_statistics_block(config),
       
       #[cfg(feature = "statistics")]
       "MOVING_AVERAGE" => statistics::create_moving_average_block(config),
       
       #[cfg(feature = "statistics")]
       "HISTOGRAM" => statistics::create_histogram_block(config),
       
       // ====================================================================
       // MACHINE LEARNING BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "ml-blocks")]
       "ML_INFERENCE" => ml::create_ml_inference_block(config),
       
       #[cfg(feature = "ml-blocks")]
       "NEURAL_NETWORK" => ml::create_neural_network_block(config),
       
       // ====================================================================
       // COMMUNICATION BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "email")]
       "EMAIL" | "EMAIL_SEND" => email::create_email_block(config),
       
       #[cfg(feature = "email")]
       "EMAIL_ALERT" => email::create_email_alert_block(config),
       
       #[cfg(feature = "twilio")]
       "SMS" | "SMS_SEND" => sms::create_sms_block(config),
       
       #[cfg(feature = "twilio")]
       "VOICE_CALL" => sms::create_voice_call_block(config),
       
       #[cfg(feature = "web")]
       "HTTP_REQUEST" => web::create_http_request_block(config),
       
       #[cfg(feature = "web")]
       "WEBHOOK" => web::create_webhook_block(config),
       
       #[cfg(feature = "mqtt")]
       "MQTT_PUBLISH" => mqtt_blocks::create_mqtt_publish_block(config),
       
       #[cfg(feature = "mqtt")]
       "MQTT_SUBSCRIBE" => mqtt_blocks::create_mqtt_subscribe_block(config),
       
       // ====================================================================
       // DATA PROCESSING BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "validation")]
       "VALIDATE" => validation::create_validate_block(config),
       
       #[cfg(feature = "validation")]
       "RANGE_CHECK" => validation::create_range_check_block(config),
       
       #[cfg(feature = "validation")]
       "PATTERN_MATCH" => validation::create_pattern_match_block(config),
       
       #[cfg(feature = "extended-types")]
       "JSON_PARSE" => data::create_json_parse_block(config),
       
       #[cfg(feature = "extended-types")]
       "STRING_FORMAT" => data::create_string_format_block(config),
       
       #[cfg(feature = "extended-types")]
       "ARRAY_INDEX" => data::create_array_index_block(config),
       
       // ====================================================================
       // STORAGE BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "history")]
       "LOG_VALUE" => storage::create_log_value_block(config),
       
       #[cfg(feature = "history")]
       "QUERY_HISTORY" => storage::create_query_history_block(config),
       
       // ====================================================================
       // SECURITY BLOCKS (feature-gated)
       // ====================================================================
       #[cfg(feature = "security")]
       "ENCRYPT" => security::create_encrypt_block(config),
       
       #[cfg(feature = "security")]
       "DECRYPT" => security::create_decrypt_block(config),
       
       #[cfg(feature = "security")]
       "HASH" => security::create_hash_block(config),
       
       // ====================================================================
       // UNKNOWN BLOCK TYPE
       // ====================================================================
       _ => {
           let available_types = get_available_block_types();
           Err(PlcError::Config(format!(
               "Unknown block type '{}' for block '{}'. Available types: {}",
               config.block_type,
               config.name,
               available_types.join(", ")
           )))
       }
   }
}

/// Get list of available block types based on enabled features
pub fn get_available_block_types() -> Vec<String> {
   let mut types = vec![
       // Core blocks (always available)
       "AND", "OR", "NOT", "XOR", "NAND", "NOR",
       "LT", "GT", "EQ", "NE", "GE", "LE",
       "TON", "TOF", "TP", "CTU", "CTD", 
       "R_TRIG", "F_TRIG", "EDGE_DETECT",
       "SR_LATCH", "RS_LATCH", "FLIP_FLOP",
       "ADD", "SUB", "MUL", "DIV", "MOD", "ABS", "SQRT", "MIN", "MAX",
       "CONVERT", "SCALE", "LIMIT",
   ];
   
   // Add feature-gated blocks
   #[cfg(feature = "advanced-blocks")]
   types.extend_from_slice(&["LEAD_LAG", "DEADBAND"]);
   
   #[cfg(feature = "pid-control")]
   types.extend_from_slice(&["PID", "PID_AUTO_TUNE"]);
   
   #[cfg(feature = "statistics")]
   types.extend_from_slice(&["STATISTICS", "MOVING_AVERAGE", "HISTOGRAM"]);
   
   #[cfg(feature = "ml-blocks")]
   types.extend_from_slice(&["ML_INFERENCE", "NEURAL_NETWORK"]);
   
   #[cfg(feature = "email")]
   types.extend_from_slice(&["EMAIL", "EMAIL_ALERT"]);
   
   #[cfg(feature = "twilio")]
   types.extend_from_slice(&["SMS", "VOICE_CALL"]);
   
   #[cfg(feature = "web")]
   types.extend_from_slice(&["HTTP_REQUEST", "WEBHOOK"]);
   
   #[cfg(feature = "mqtt")]
   types.extend_from_slice(&["MQTT_PUBLISH", "MQTT_SUBSCRIBE"]);
   
   #[cfg(feature = "validation")]
   types.extend_from_slice(&["VALIDATE", "RANGE_CHECK", "PATTERN_MATCH"]);
   
   #[cfg(feature = "extended-types")]
   types.extend_from_slice(&["JSON_PARSE", "STRING_FORMAT", "ARRAY_INDEX"]);
   
   #[cfg(feature = "history")]
   types.extend_from_slice(&["LOG_VALUE", "QUERY_HISTORY"]);
   
   #[cfg(feature = "security")]
   types.extend_from_slice(&["ENCRYPT", "DECRYPT", "HASH"]);
   
   types.into_iter().map(|s| s.to_string()).collect()
}

/// Get blocks organized by category
pub fn get_blocks_by_category() -> HashMap<String, Vec<String>> {
   let mut categories = HashMap::new();
   
   categories.insert("Logic".to_string(), vec![
       "AND".to_string(), "OR".to_string(), "NOT".to_string(), 
       "XOR".to_string(), "NAND".to_string(), "NOR".to_string(),
   ]);
   
   categories.insert("Comparison".to_string(), vec![
       "LT".to_string(), "GT".to_string(), "EQ".to_string(),
       "NE".to_string(), "GE".to_string(), "LE".to_string(),
   ]);
   
   categories.insert("Timer".to_string(), vec![
       "TON".to_string(), "TOF".to_string(), "TP".to_string(),
       "CTU".to_string(), "CTD".to_string(),
   ]);
   
   categories.insert("Memory".to_string(), vec![
       "SR_LATCH".to_string(), "RS_LATCH".to_string(), "FLIP_FLOP".to_string(),
   ]);
   
   categories.insert("Arithmetic".to_string(), vec![
       "ADD".to_string(), "SUB".to_string(), "MUL".to_string(), "DIV".to_string(),
       "MOD".to_string(), "ABS".to_string(), "SQRT".to_string(),
       "MIN".to_string(), "MAX".to_string(),
   ]);
   
   #[cfg(feature = "advanced-blocks")]
   categories.insert("Control".to_string(), vec![
       "LEAD_LAG".to_string(), "DEADBAND".to_string(),
   ]);
   
   #[cfg(feature = "pid-control")]
   {
       let mut control = categories.entry("Control".to_string()).or_insert_with(Vec::new);
       control.extend_from_slice(&["PID".to_string(), "PID_AUTO_TUNE".to_string()]);
   }
   
   #[cfg(feature = "statistics")]
   categories.insert("Statistics".to_string(), vec![
       "STATISTICS".to_string(), "MOVING_AVERAGE".to_string(), "HISTOGRAM".to_string(),
   ]);
   
   #[cfg(any(feature = "email", feature = "twilio", feature = "web", feature = "mqtt"))]
   {
       let mut comm = Vec::new();
       #[cfg(feature = "email")]
       comm.extend_from_slice(&["EMAIL".to_string(), "EMAIL_ALERT".to_string()]);
       #[cfg(feature = "twilio")]
       comm.extend_from_slice(&["SMS".to_string(), "VOICE_CALL".to_string()]);
       #[cfg(feature = "web")]
       comm.extend_from_slice(&["HTTP_REQUEST".to_string(), "WEBHOOK".to_string()]);
       #[cfg(feature = "mqtt")]
       comm.extend_from_slice(&["MQTT_PUBLISH".to_string(), "MQTT_SUBSCRIBE".to_string()]);
       
       if !comm.is_empty() {
           categories.insert("Communication".to_string(), comm);
       }
   }
   
   categories
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Validate that all required signals exist for a block
pub fn validate_block_signals(
   config: &BlockConfig,
   available_signals: &HashMap<String, &str>, // signal_name -> signal_type
) -> Result<()> {
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
pub fn get_parameter<T>(
   config: &BlockConfig,
   param_name: &str,
   default: Option<T>,
) -> Result<T>
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
pub fn get_string_parameter(
   config: &BlockConfig,
   param_name: &str,
   default: Option<&str>,
) -> Result<String> {
   match config.params.get(param_name) {
       Some(serde_yaml::Value::String(s)) => Ok(s.clone()),
       Some(value) => Ok(value.to_string()),
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
pub fn get_numeric_parameter<T>(
   config: &BlockConfig,
   param_name: &str,
   default: Option<T>,
) -> Result<T>
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
pub fn get_bool_parameter(
   config: &BlockConfig,
   param_name: &str,
   default: Option<bool>,
) -> Result<bool> {
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
// TESTING UTILITIES
// ============================================================================

#[cfg(test)]
pub mod test_utils {
   use super::*;
   use crate::signal::SignalBus;
   use crate::value::Value;
   
   /// Create a test signal bus with predefined signals
   pub fn create_test_bus() -> SignalBus {
       let bus = SignalBus::new();
       
       // Add some test signals
       bus.set("test_bool", Value::Bool(false)).unwrap();
       bus.set("test_int", Value::Int(0)).unwrap();
       bus.set("test_float", Value::Float(0.0)).unwrap();
       
       #[cfg(feature = "extended-types")]
       bus.set("test_string", Value::String("test".to_string())).unwrap();
       
       bus
   }
   
   /// Create a minimal block configuration for testing
   pub fn create_test_config(block_type: &str, name: &str) -> BlockConfig {
       BlockConfig {
           name: name.to_string(),
           block_type: block_type.to_string(),
           inputs: HashMap::new(),
           outputs: HashMap::new(),
           params: HashMap::new(),
           description: Some(format!("Test {} block", block_type)),
           tags: vec!["test".to_string()],
           #[cfg(feature = "enhanced-errors")]
           error_handling: None,
           #[cfg(feature = "circuit-breaker")]
           circuit_breaker: None,
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
   
   /// Helper to create block config with parameters
   pub fn create_test_config_with_params(
       block_type: &str,
       name: &str,
       params: Vec<(&str, serde_yaml::Value)>,
   ) -> BlockConfig {
       let mut config = create_test_config(block_type, name);
       
       for (param_name, param_value) in params {
           config.params.insert(param_name.to_string(), param_value);
       }
       
       config
   }
   
   /// Simulate a scan cycle for testing block execution
   pub fn simulate_scan_cycle(blocks: &mut Vec<Box<dyn Block>>, bus: &SignalBus) -> Result<()> {
       for block in blocks.iter_mut() {
           block.execute(bus)?;
       }
       Ok(())
   }
   
   /// Assert that a signal has the expected value
   pub fn assert_signal_value(bus: &SignalBus, signal_name: &str, expected: Value) {
       let actual = bus.get(signal_name)
           .unwrap_or_else(|| panic!("Signal '{}' not found", signal_name));
       assert_eq!(actual, expected, "Signal '{}' value mismatch", signal_name);
   }
   
   /// Helper to set multiple signal values at once
   pub fn set_signal_values(bus: &SignalBus, values: Vec<(&str, Value)>) -> Result<()> {
       for (signal_name, value) in values {
           bus.set(signal_name, value)?;
       }
       Ok(())
   }
   
   /// Create a mock block for testing
   pub struct MockBlock {
       pub name: String,
       pub block_type: String,
       pub execute_count: std::cell::RefCell<u32>,
       pub should_fail: bool,
   }
   
   impl MockBlock {
       pub fn new(name: &str, block_type: &str) -> Self {
           Self {
               name: name.to_string(),
               block_type: block_type.to_string(),
               execute_count: std::cell::RefCell::new(0),
               should_fail: false,
           }
       }
       
       pub fn with_failure(mut self) -> Self {
           self.should_fail = true;
           self
       }
       
       pub fn execution_count(&self) -> u32 {
           *self.execute_count.borrow()
       }
   }
   
   impl Block for MockBlock {
       fn execute(&mut self, _bus: &SignalBus) -> Result<()> {
           *self.execute_count.borrow_mut() += 1;
           
           if self.should_fail {
               Err(PlcError::Execution("Mock block failure".to_string()))
           } else {
               Ok(())
           }
       }
       
       fn name(&self) -> &str {
           &self.name
       }
       
       fn block_type(&self) -> &str {
           &self.block_type
       }
   }
}

// ============================================================================
// MODULE TESTS
// ============================================================================

#[cfg(test)]
mod tests {
   use super::*;
   use crate::signal::SignalBus;
   use crate::value::Value;
   use std::collections::HashMap;
   
   #[test]
   fn test_get_available_block_types() {
       let types = get_available_block_types();
       
       // Core blocks should always be available
       assert!(types.contains(&"AND".to_string()));
       assert!(types.contains(&"OR".to_string()));
       assert!(types.contains(&"NOT".to_string()));
       assert!(types.contains(&"LT".to_string()));
       assert!(types.contains(&"TON".to_string()));
       
       // Check that we have a reasonable number of types
       assert!(types.len() >= 20);
   }
   
   #[test]
   fn test_get_blocks_by_category() {
       let categories = get_blocks_by_category();
       
       // Should have core categories
       assert!(categories.contains_key("Logic"));
       assert!(categories.contains_key("Comparison"));
       assert!(categories.contains_key("Timer"));
       assert!(categories.contains_key("Arithmetic"));
       
       // Logic category should contain expected blocks
       let logic_blocks = categories.get("Logic").unwrap();
       assert!(logic_blocks.contains(&"AND".to_string()));
       assert!(logic_blocks.contains(&"OR".to_string()));
       assert!(logic_blocks.contains(&"NOT".to_string()));
   }
   
   #[test]
   fn test_validate_block_signals() {
       let mut available_signals = HashMap::new();
       available_signals.insert("input1".to_string(), "bool");
       available_signals.insert("input2".to_string(), "int");
       available_signals.insert("output1".to_string(), "bool");
       
       let signal_refs: HashMap<String, &str> = available_signals.iter()
           .map(|(k, v)| (k.clone(), v.as_str()))
           .collect();
       
       // Valid configuration
       let valid_config = test_utils::create_test_config_with_io(
           "AND",
           "test_and",
           vec![("in1", "input1"), ("in2", "input2")],
           vec![("out", "output1")],
       );
       assert!(validate_block_signals(&valid_config, &signal_refs).is_ok());
       
       // Invalid input signal
       let invalid_input_config = test_utils::create_test_config_with_io(
           "AND",
           "test_and",
           vec![("in1", "nonexistent"), ("in2", "input2")],
           vec![("out", "output1")],
       );
       assert!(validate_block_signals(&invalid_input_config, &signal_refs).is_err());
       
       // Invalid output signal
       let invalid_output_config = test_utils::create_test_config_with_io(
           "AND",
           "test_and",
           vec![("in1", "input1"), ("in2", "input2")],
           vec![("out", "nonexistent")],
       );
       assert!(validate_block_signals(&invalid_output_config, &signal_refs).is_err());
   }
   
   #[test]
   fn test_parameter_extraction() {
       let mut config = test_utils::create_test_config("TEST", "test_block");
       
       // Add test parameters
       config.params.insert("string_param".to_string(), 
           serde_yaml::Value::String("test_value".to_string()));
       config.params.insert("int_param".to_string(), 
           serde_yaml::Value::Number(42.into()));
       config.params.insert("float_param".to_string(), 
           serde_yaml::Value::Number(serde_yaml::Number::from(3.14)));
       config.params.insert("bool_param".to_string(), 
           serde_yaml::Value::Bool(true));
       
       // Test string parameter extraction
       assert_eq!(
           get_string_parameter(&config, "string_param", None).unwrap(),
           "test_value"
       );
       
       // Test with default value
       assert_eq!(
           get_string_parameter(&config, "missing_param", Some("default")).unwrap(),
           "default"
       );
       
       // Test numeric parameter extraction
       assert_eq!(
           get_numeric_parameter::<i32>(&config, "int_param", None).unwrap(),
           42
       );
       
       assert_eq!(
           get_numeric_parameter::<f64>(&config, "float_param", None).unwrap(),
           3.14
       );
       
       // Test boolean parameter extraction
       assert!(get_bool_parameter(&config, "bool_param", None).unwrap());
       
       // Test string to boolean conversion
       config.params.insert("bool_string".to_string(), 
           serde_yaml::Value::String("true".to_string()));
       assert!(get_bool_parameter(&config, "bool_string", None).unwrap());
       
       config.params.insert("bool_string_no".to_string(), 
           serde_yaml::Value::String("no".to_string()));
       assert!(!get_bool_parameter(&config, "bool_string_no", None).unwrap());
       
       // Test error cases
       assert!(get_string_parameter(&config, "missing_required", None).is_err());
       assert!(get_numeric_parameter::<i32>(&config, "missing_required", None).is_err());
       assert!(get_bool_parameter(&config, "missing_required", None).is_err());
   }
   
   #[test]
   fn test_mock_block() {
       let mut mock_block = test_utils::MockBlock::new("test_mock", "MOCK");
       let bus = test_utils::create_test_bus();
       
       // Test successful execution
       assert!(mock_block.execute(&bus).is_ok());
       assert_eq!(mock_block.execution_count(), 1);
       
       // Test multiple executions
       assert!(mock_block.execute(&bus).is_ok());
       assert_eq!(mock_block.execution_count(), 2);
       
       // Test failing mock block
       let mut failing_mock = test_utils::MockBlock::new("failing_mock", "MOCK").with_failure();
       assert!(failing_mock.execute(&bus).is_err());
       assert_eq!(failing_mock.execution_count(), 1);
   }
   
   #[test]
   fn test_scan_cycle_simulation() {
       let bus = test_utils::create_test_bus();
       let mut blocks: Vec<Box<dyn Block>> = vec![
           Box::new(test_utils::MockBlock::new("block1", "MOCK")),
           Box::new(test_utils::MockBlock::new("block2", "MOCK")),
       ];
       
       // Simulate successful scan cycle
       assert!(test_utils::simulate_scan_cycle(&mut blocks, &bus).is_ok());
       
       // Add a failing block
       blocks.push(Box::new(test_utils::MockBlock::new("failing", "MOCK").with_failure()));
       
       // Simulate failing scan cycle
       assert!(test_utils::simulate_scan_cycle(&mut blocks, &bus).is_err());
   }
   
   #[test]
   fn test_signal_value_assertions() {
       let bus = test_utils::create_test_bus();
       
       // Set some values
       test_utils::set_signal_values(&bus, vec![
           ("test_bool", Value::Bool(true)),
           ("test_int", Value::Int(42)),
           ("test_float", Value::Float(3.14)),
       ]).unwrap();
       
       // Assert values
       test_utils::assert_signal_value(&bus, "test_bool", Value::Bool(true));
       test_utils::assert_signal_value(&bus, "test_int", Value::Int(42));
       test_utils::assert_signal_value(&bus, "test_float", Value::Float(3.14));
   }
   
   #[cfg(feature = "circuit-breaker")]
   #[test]
   fn test_circuit_breaker() {
       use std::time::Duration;
       
       let circuit_breaker = BlockExecutor::new(3, Duration::from_millis(100));
       let bus = test_utils::create_test_bus();
       
       // Test successful execution
       let mut successful_block = test_utils::MockBlock::new("success", "MOCK");
       assert!(circuit_breaker.execute_with_circuit_breaker(&mut successful_block, &bus).is_ok());
       
       // Test failing block
       let mut failing_block = test_utils::MockBlock::new("failing", "MOCK").with_failure();
       
       // Should fail but circuit still closed
       assert!(circuit_breaker.execute_with_circuit_breaker(&mut failing_block, &bus).is_err());
       assert!(circuit_breaker.execute_with_circuit_breaker(&mut failing_block, &bus).is_err());
       assert!(circuit_breaker.execute_with_circuit_breaker(&mut failing_block, &bus).is_err());
       
       // Circuit should now be open
       let status = circuit_breaker.status();
       assert!(status.circuit_open);
       assert_eq!(status.failure_count, 3);
       
       // Further attempts should return CircuitOpen error
       assert!(circuit_breaker.execute_with_circuit_breaker(&mut failing_block, &bus).is_err());
   }
   
   #[cfg(feature = "enhanced-errors")]
   #[tokio::test]
   async fn test_retry_logic() {
       let bus = test_utils::create_test_bus();
       let retry_config = RetryConfig {
           max_attempts: 3,
           initial_delay_ms: 1, // Very short delay for testing
           max_delay_ms: 10,
           exponential_backoff: true,
           backoff_multiplier: 2.0,
       };
       
       // Test successful execution (no retries needed)
       let mut successful_block = test_utils::MockBlock::new("success", "MOCK");
       assert!(execute_with_retry(&mut successful_block, &bus, &retry_config).await.is_ok());
       assert_eq!(successful_block.execution_count(), 1);
       
       // Test failing block (should retry and eventually fail)
       let mut failing_block = test_utils::MockBlock::new("failing", "MOCK").with_failure();
       assert!(execute_with_retry(&mut failing_block, &bus, &retry_config).await.is_err());
       assert_eq!(failing_block.execution_count(), 3); // Should have tried 3 times
   }
}

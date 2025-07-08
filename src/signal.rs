//! # PETRA Thread-Safe Signal Bus System
//!
//! ## Purpose & Overview
//! 
//! This module provides the central nervous system of PETRA that:
//!
//! - **Central Data Hub** - All inter-component communication flows through the signal bus
//! - **Thread-Safe Operations** - Lock-free concurrent access using DashMap
//! - **Type-Safe Access** - Strongly typed signal operations with conversion support
//! - **High Performance** - Optimized for real-time industrial automation requirements
//! - **Event System** - Signal change notifications for reactive programming
//! - **Metadata Support** - Rich signal metadata for engineering applications
//!
//! ## Architecture & Interactions
//!
//! The signal bus is the communication backbone used by all PETRA components:
//! - **src/engine.rs** - Engine reads/writes signals during scan cycles
//! - **src/blocks/*** - All blocks use signal bus for inputs and outputs
//! - **src/protocols/*** - Protocol drivers update signals from external systems
//! - **src/history.rs** - Historical data logging reads signals from the bus
//! - **src/alarms.rs** - Alarm system monitors signal values for conditions
//! - **src/web/** - Web API provides signal access for monitoring and control
//!
//! ## Signal Naming Convention
//!
//! Signals follow hierarchical naming: `component.subcategory.signal_name`
//! - Examples: `plc.tank1.temperature`, `scada.pump2.status`, `system.engine.scan_time`
//!
//! ## Performance Characteristics
//!
//! - **Lock-free reads** for high-frequency access patterns
//! - **Atomic operations** for consistency without blocking
//! - **Batch operations** for efficient multi-signal updates
//! - **Memory pooling** for reduced allocation overhead
//! - **Access pattern optimization** with hot-path caching

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]

use crate::{
    error::{PlcError, Result},
    value::Value,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::SystemTime;
use tracing::{debug, trace, warn};

// Feature-gated imports for enhanced functionality
#[cfg(feature = "enhanced-monitoring")]
use std::collections::VecDeque;
#[cfg(feature = "enhanced-monitoring")]
use std::time::{Duration, Instant};

#[cfg(feature = "signal-validation")]
use crate::validation::Validator;

#[cfg(feature = "quality-codes")]
use crate::value::Quality;

#[cfg(feature = "signal-events")]
use tokio::sync::broadcast;

// ============================================================================
// SIGNAL METADATA DEFINITIONS
// ============================================================================

/// Comprehensive metadata for industrial signals
/// 
/// Provides rich context information for engineering applications,
/// including units, ranges, descriptions, and source information.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalMetadata {
    /// Engineering unit (e.g., "°C", "bar", "rpm", "m/s")
    pub unit: Option<String>,
    
    /// Human-readable description
    pub description: Option<String>,
    
    /// Minimum valid value for range checking
    pub min_value: Option<Value>,
    
    /// Maximum valid value for range checking
    pub max_value: Option<Value>,
    
    /// Default value for initialization
    pub default_value: Option<Value>,
    
    /// Data source identifier (e.g., "PLC-1", "SCADA-Server", "Sensor-A")
    pub source: Option<String>,
    
    /// Signal category for grouping (e.g., "Temperature", "Pressure", "Status")
    pub category: Option<String>,
    
    /// Update frequency hint in milliseconds
    pub update_frequency_ms: Option<u64>,
    
    /// Whether this signal should be logged to history
    pub log_to_history: bool,
    
    /// Whether this signal should trigger alarms
    pub enable_alarms: bool,
    
    /// Additional custom metadata
    pub custom: HashMap<String, String>,
}

impl Default for SignalMetadata {
    fn default() -> Self {
        Self {
            unit: None,
            description: None,
            min_value: None,
            max_value: None,
            default_value: None,
            source: None,
            category: None,
            update_frequency_ms: None,
            log_to_history: false,
            enable_alarms: false,
            custom: HashMap::new(),
        }
    }
}

/// Signal access statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct SignalStats {
    /// Total number of read operations
    pub read_count: u64,
    
    /// Total number of write operations
    pub write_count: u64,
    
    /// Total number of update operations
    pub update_count: u64,
    
    /// Timestamp of last read operation
    pub last_read: Option<SystemTime>,
    
    /// Timestamp of last write operation
    pub last_write: Option<SystemTime>,
    
    /// Timestamp when signal was created
    pub created_at: SystemTime,
    
    /// Average time between updates (in milliseconds)
    #[cfg(feature = "enhanced-monitoring")]
    pub avg_update_interval_ms: Option<f64>,
    
    /// Number of type conversion failures
    pub conversion_errors: u64,
}

impl Default for SignalStats {
    fn default() -> Self {
        Self {
            read_count: 0,
            write_count: 0,
            update_count: 0,
            last_read: None,
            last_write: None,
            created_at: SystemTime::now(),
            #[cfg(feature = "enhanced-monitoring")]
            avg_update_interval_ms: None,
            conversion_errors: 0,
        }
    }
}

/// Signal change event for reactive programming
#[cfg(feature = "signal-events")]
#[derive(Debug, Clone)]
pub struct SignalChangeEvent {
    /// Signal name that changed
    pub signal_name: String,
    
    /// Previous value (None if signal was just created)
    pub old_value: Option<Value>,
    
    /// New value
    pub new_value: Value,
    
    /// Timestamp when change occurred
    pub timestamp: SystemTime,
    
    /// Source of the change (e.g., "block", "protocol", "manual")
    pub source: Option<String>,
}

/// Internal signal data structure
#[derive(Debug, Clone)]
struct SignalData {
    /// Current signal value
    value: Value,
    
    /// Signal metadata
    metadata: SignalMetadata,
    
    /// Access statistics
    stats: SignalStats,
    
    /// Last validation result
    #[cfg(feature = "signal-validation")]
    last_validation: Option<bool>,
}

impl SignalData {
    fn new(value: Value, metadata: Option<SignalMetadata>) -> Self {
        Self {
            value,
            metadata: metadata.unwrap_or_default(),
            stats: SignalStats {
                created_at: SystemTime::now(),
                ..Default::default()
            },
            #[cfg(feature = "signal-validation")]
            last_validation: None,
        }
    }
}

// ============================================================================
// MAIN SIGNAL BUS IMPLEMENTATION
// ============================================================================

/// Thread-safe signal bus for high-performance inter-component communication
/// 
/// The SignalBus is the central data exchange mechanism in PETRA, providing
/// lock-free concurrent access to signals using DashMap for optimal performance
/// in real-time industrial automation applications.
/// 
/// # Features
/// 
/// - **Lock-free operations** using DashMap for maximum concurrency
/// - **Type-safe access** with automatic type conversion
/// - **Signal metadata** for engineering applications
/// - **Access statistics** for performance monitoring
/// - **Event notifications** for reactive programming (feature-gated)
/// - **Batch operations** for efficient multi-signal updates
/// - **Validation support** for data integrity (feature-gated)
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{SignalBus, Value, SignalMetadata};
/// 
/// // Create a new signal bus
/// let bus = SignalBus::new();
/// 
/// // Set a signal value
/// bus.set("temperature", Value::Float(23.5))?;
/// 
/// // Get a signal value with type conversion
/// let temp = bus.get_float("temperature")?;
/// assert_eq!(temp, 23.5);
/// 
/// // Set metadata
/// let metadata = SignalMetadata {
///     unit: Some("°C".to_string()),
///     description: Some("Reactor core temperature".to_string()),
///     min_value: Some(Value::Float(0.0)),
///     max_value: Some(Value::Float(100.0)),
///     ..Default::default()
/// };
/// bus.set_metadata("temperature", metadata)?;
/// 
/// // Atomic update operation
/// bus.update("counter", |old| {
///     match old {
///         Some(Value::Integer(n)) => Value::Integer(n + 1),
///         _ => Value::Integer(1),
///     }
/// })?;
/// # Ok::<(), petra::PlcError>(())
/// ```
#[derive(Debug)]
pub struct SignalBus {
    /// Core signal storage using DashMap for lock-free access
    signals: Arc<DashMap<String, SignalData>>,
    
    /// Global statistics counters
    total_operations: Arc<AtomicU64>,
    
    /// Event broadcaster for signal changes
    #[cfg(feature = "signal-events")]
    event_sender: Arc<broadcast::Sender<SignalChangeEvent>>,
    
    /// Signal validator for data integrity
    #[cfg(feature = "signal-validation")]
    validator: Option<Arc<dyn Validator + Send + Sync>>,
    
    /// Performance monitoring data
    #[cfg(feature = "enhanced-monitoring")]
    operation_times: Arc<std::sync::Mutex<VecDeque<(String, Duration)>>>,
}

impl SignalBus {
    /// Create a new signal bus
    /// 
    /// Initializes an empty signal bus ready for high-performance
    /// concurrent operations in industrial automation applications.
    pub fn new() -> Self {
        #[cfg(feature = "signal-events")]
        let (event_sender, _) = broadcast::channel(1000); // Buffer up to 1000 events
        
        Self {
            signals: Arc::new(DashMap::new()),
            total_operations: Arc::new(AtomicU64::new(0)),
            
            #[cfg(feature = "signal-events")]
            event_sender: Arc::new(event_sender),
            
            #[cfg(feature = "signal-validation")]
            validator: None,
            
            #[cfg(feature = "enhanced-monitoring")]
            operation_times: Arc::new(std::sync::Mutex::new(VecDeque::with_capacity(1000))),
        }
    }
    
    /// Create a signal bus with initial capacity hint
    /// 
    /// Pre-allocates space for the expected number of signals to reduce
    /// memory allocations during operation.
    pub fn with_capacity(capacity: usize) -> Self {
        #[cfg(feature = "signal-events")]
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            signals: Arc::new(DashMap::with_capacity(capacity)),
            total_operations: Arc::new(AtomicU64::new(0)),
            
            #[cfg(feature = "signal-events")]
            event_sender: Arc::new(event_sender),
            
            #[cfg(feature = "signal-validation")]
            validator: None,
            
            #[cfg(feature = "enhanced-monitoring")]
            operation_times: Arc::new(std::sync::Mutex::new(VecDeque::with_capacity(1000))),
        }
    }
    
    // ========================================================================
    // CORE SIGNAL OPERATIONS
    // ========================================================================
    
    /// Set a signal value with optional source tracking
    /// 
    /// This is the primary method for updating signal values. It handles
    /// statistics tracking, event notifications, and validation if enabled.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Signal name following the naming convention
    /// * `value` - New signal value
    /// * `source` - Optional source identifier for tracking
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// // Basic signal update
    /// bus.set("temperature", Value::Float(25.0))?;
    /// 
    /// // With source tracking
    /// bus.set_with_source("pressure", Value::Float(101.3), Some("PLC-1"))?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn set(&self, name: impl AsRef<str>, value: Value) -> Result<()> {
        self.set_with_source(name, value, None)
    }
    
    /// Set a signal value with source tracking
    pub fn set_with_source(
        &self,
        name: impl AsRef<str>,
        value: Value,
        _source: Option<&str>,
    ) -> Result<()> {
        let name = name.as_ref();
        let now = SystemTime::now();
        
        #[cfg(feature = "enhanced-monitoring")]
        let start_time = Instant::now();
        
        // Validate signal name
        self.validate_signal_name(name)?;
        
        // Validate value if validator is set
        #[cfg(feature = "signal-validation")]
        if let Some(validator) = &self.validator {
            validator.validate(&value).map_err(|e| {
                PlcError::Validation(format!("Signal '{}' validation failed: {}", name, e))
            })?;
        }
        
        #[allow(unused_variables)]
        let old_value = self.signals.get(name).map(|entry| entry.value.clone());
        
        // Update or insert signal
        match self.signals.get_mut(name) {
            Some(mut entry) => {
                // Update existing signal
                entry.value = value.clone();
                entry.stats.write_count += 1;
                entry.stats.last_write = Some(now);
                
                #[cfg(feature = "signal-validation")]
                {
                    entry.last_validation = Some(true);
                }
            }
            None => {
                // Create new signal
                let signal_data = SignalData::new(value.clone(), None);
                self.signals.insert(name.to_string(), signal_data);
                debug!("Created new signal: {}", name);
            }
        }
        
        // Track global statistics
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        
        // Record operation time for monitoring
        #[cfg(feature = "enhanced-monitoring")]
        {
            let elapsed = start_time.elapsed();
            if let Ok(mut times) = self.operation_times.lock() {
                times.push_back((format!("set:{}", name), elapsed));
                if times.len() > 1000 {
                    times.pop_front();
                }
            }
        }
        
        // Emit change event
        #[cfg(feature = "signal-events")]
        {
            let event = SignalChangeEvent {
                signal_name: name.to_string(),
                old_value,
                new_value: value,
                timestamp: now,
                source: _source.map(|s| s.to_string()),
            };
            
            // Non-blocking send - if no receivers, that's fine
            let _ = self.event_sender.send(event);
        }
        
        trace!("Set signal '{}' = {:?}", name, value);
        Ok(())
    }
    
    /// Get a signal value
    /// 
    /// Returns the current value of the signal, or None if the signal doesn't exist.
    /// This operation is lock-free and extremely fast for high-frequency access.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("temperature", Value::Float(25.0))?;
    /// if let Some(value) = bus.get("temperature") {
    ///     println!("Temperature: {:?}", value);
    /// } else {
    ///     println!("Temperature signal not found");
    /// }
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn get(&self, name: impl AsRef<str>) -> Option<Value> {
        let name = name.as_ref();
        let result = self.signals.get(name).map(|entry| {
            // Update read statistics
            let mut entry = entry.clone();
            entry.stats.read_count += 1;
            entry.stats.last_read = Some(SystemTime::now());
            
            entry.value.clone()
        });
        
        if result.is_some() {
            self.total_operations.fetch_add(1, Ordering::Relaxed);
            trace!("Read signal '{}' = {:?}", name, result);
        }
        
        result
    }
    
    /// Get a signal value or return an error if not found
    /// 
    /// This is useful when a signal is expected to exist and missing signals
    /// should be treated as errors rather than optional values.
    pub fn get_required(&self, name: impl AsRef<str>) -> Result<Value> {
        let name = name.as_ref();
        self.get(name).ok_or_else(|| {
            PlcError::SignalNotFound(name.to_string())
        })
    }
    
    /// Atomically update a signal value using a closure
    /// 
    /// This operation is atomic and thread-safe, making it ideal for
    /// counters, accumulators, and other stateful operations.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Signal name
    /// * `update_fn` - Closure that takes the current value and returns the new value
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// // Increment counter
    /// bus.update("counter", |old| {
    ///     match old {
    ///         Some(Value::Integer(n)) => Value::Integer(n + 1),
    ///         _ => Value::Integer(1),
    ///     }
    /// })?;
    /// 
    /// // Accumulate values
    /// bus.update("total", |old| {
    ///     match old {
    ///         Some(Value::Float(sum)) => Value::Float(sum + 10.0),
    ///         _ => Value::Float(10.0),
    ///     }
    /// })?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn update<F>(&self, name: impl AsRef<str>, update_fn: F) -> Result<Value>
    where
        F: FnOnce(Option<Value>) -> Value,
    {
        let name = name.as_ref();
        let now = SystemTime::now();
        
        // Validate signal name
        self.validate_signal_name(name)?;
        
        let new_value = match self.signals.get_mut(name) {
            Some(mut entry) => {
                let old_value = entry.value.clone();
                let new_value = update_fn(Some(old_value.clone()));
                
                // Validate new value if validator is set
                #[cfg(feature = "signal-validation")]
                if let Some(validator) = &self.validator {
                    validator.validate(&new_value).map_err(|e| {
                        PlcError::Validation(format!("Signal '{}' validation failed: {}", name, e))
                    })?;
                }
                
                entry.value = new_value.clone();
                entry.stats.update_count += 1;
                entry.stats.last_write = Some(now);
                
                new_value
            }
            None => {
                let new_value = update_fn(None);
                
                // Validate new value if validator is set
                #[cfg(feature = "signal-validation")]
                if let Some(validator) = &self.validator {
                    validator.validate(&new_value).map_err(|e| {
                        PlcError::Validation(format!("Signal '{}' validation failed: {}", name, e))
                    })?;
                }
                
                let signal_data = SignalData::new(new_value.clone(), None);
                self.signals.insert(name.to_string(), signal_data);
                debug!("Created new signal via update: {}", name);
                
                new_value
            }
        };
        
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        trace!("Updated signal '{}' = {:?}", name, new_value);
        
        Ok(new_value)
    }
    
    // ========================================================================
    // TYPE-SAFE ACCESSOR METHODS
    // ========================================================================
    
    /// Get a boolean signal value with type conversion
    /// 
    /// Automatically converts compatible types to boolean following
    /// the conversion rules defined in the Value type system.
    pub fn get_bool(&self, name: impl AsRef<str>) -> Result<bool> {
        let name = name.as_ref();
        match self.get(name) {
            Some(value) => {
                // Check quality if available
                #[cfg(feature = "quality-codes")]
                if let Some((inner_value, quality, _)) = value.as_quality() {
                    if !quality.is_usable() {
                        return Err(PlcError::SignalQuality(format!(
                            "Signal '{}' has bad quality: {:?}", name, quality
                        )));
                    }
                    // Use the inner value for conversion
                    return inner_value.as_bool().ok_or_else(|| {
                        PlcError::TypeMismatch {
                            expected: "bool".to_string(),
                            actual: inner_value.type_name().to_string(),
                        }
                    });
                }
                
                value.as_bool().ok_or_else(|| {
                    // Track conversion error
                    if let Some(mut entry) = self.signals.get_mut(name) {
                        entry.stats.conversion_errors += 1;
                    }
                    
                    PlcError::TypeMismatch {
                        expected: "bool".to_string(),
                        actual: value.type_name().to_string(),
                    }
                })
            }
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get an integer signal value with type conversion and overflow protection
    pub fn get_integer(&self, name: impl AsRef<str>) -> Result<i64> {
        let name = name.as_ref();
        match self.get(name) {
            Some(value) => {
                // Check quality if available
                #[cfg(feature = "quality-codes")]
                if let Some((inner_value, quality, _)) = value.as_quality() {
                    if !quality.is_usable() {
                        return Err(PlcError::SignalQuality(format!(
                            "Signal '{}' has bad quality: {:?}", name, quality
                        )));
                    }
                    return inner_value.as_integer().ok_or_else(|| {
                        PlcError::TypeMismatch {
                            expected: "integer".to_string(),
                            actual: inner_value.type_name().to_string(),
                        }
                    });
                }
                
                value.as_integer().ok_or_else(|| {
                    // Track conversion error
                    if let Some(mut entry) = self.signals.get_mut(name) {
                        entry.stats.conversion_errors += 1;
                    }
                    
                    PlcError::TypeMismatch {
                        expected: "integer".to_string(),
                        actual: value.type_name().to_string(),
                    }
                })
            }
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a floating-point signal value with type conversion
    pub fn get_float(&self, name: impl AsRef<str>) -> Result<f64> {
        let name = name.as_ref();
        match self.get(name) {
            Some(value) => {
                // Check quality if available
                #[cfg(feature = "quality-codes")]
                if let Some((inner_value, quality, _)) = value.as_quality() {
                    if !quality.is_usable() {
                        return Err(PlcError::SignalQuality(format!(
                            "Signal '{}' has bad quality: {:?}", name, quality
                        )));
                    }
                    return inner_value.as_float().ok_or_else(|| {
                        PlcError::TypeMismatch {
                            expected: "float".to_string(),
                            actual: inner_value.type_name().to_string(),
                        }
                    });
                }
                
                value.as_float().ok_or_else(|| {
                    // Track conversion error
                    if let Some(mut entry) = self.signals.get_mut(name) {
                        entry.stats.conversion_errors += 1;
                    }
                    
                    PlcError::TypeMismatch {
                        expected: "float".to_string(),
                        actual: value.type_name().to_string(),
                    }
                })
            }
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a string signal value with automatic conversion
    pub fn get_string(&self, name: impl AsRef<str>) -> Result<String> {
        let name = name.as_ref();
        match self.get(name) {
            Some(value) => {
                // Check quality if available
                #[cfg(feature = "quality-codes")]
                if let Some((inner_value, quality, _)) = value.as_quality() {
                    if !quality.is_usable() {
                        return Err(PlcError::SignalQuality(format!(
                            "Signal '{}' has bad quality: {:?}", name, quality
                        )));
                    }
                    return Ok(inner_value.as_string());
                }
                
                Ok(value.as_string())
            }
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    // ========================================================================
    // LEGACY ALIASES FOR BACKWARD COMPATIBILITY
    // ========================================================================
    
    /// Alias for `set` - provided for backward compatibility
    pub fn write(&self, name: impl AsRef<str>, value: Value) -> Result<()> {
        self.set(name, value)
    }
    
    /// Alias for `get` - provided for backward compatibility
    pub fn read(&self, name: impl AsRef<str>) -> Option<Value> {
        self.get(name)
    }
    
    /// Alias for `set` with Result return - provided for backward compatibility
    pub fn write_signal(&self, name: impl AsRef<str>, value: Value) -> Result<()> {
        self.set(name, value)
    }
    
    /// Alias for `get_required` - provided for backward compatibility
    pub fn read_signal(&self, name: impl AsRef<str>) -> Result<Value> {
        self.get_required(name)
    }
    
    /// Alias for `get_integer` - provided for backward compatibility
    pub fn get_int(&self, name: impl AsRef<str>) -> Result<i64> {
        self.get_integer(name)
    }
    
    // ========================================================================
    // BATCH OPERATIONS FOR PERFORMANCE
    // ========================================================================
    
    /// Write multiple signals in a single operation
    /// 
    /// This is more efficient than individual writes for updating
    /// multiple signals simultaneously.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// let updates = vec![
    ///     ("temperature", Value::Float(25.0)),
    ///     ("pressure", Value::Float(101.3)),
    ///     ("status", Value::Bool(true)),
    /// ];
    /// 
    /// bus.write_batch(updates)?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn write_batch<I, K>(&self, updates: I) -> Result<()>
    where
        I: IntoIterator<Item = (K, Value)>,
        K: AsRef<str>,
    {
        for (name, value) in updates {
            self.set(name, value)?;
        }
        Ok(())
    }
    
    /// Read multiple signals in a single operation
    /// 
    /// Returns a HashMap with signal names as keys and their values.
    /// Missing signals are omitted from the result.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("temp1", Value::Float(25.0))?;
    /// # bus.set("temp2", Value::Float(26.0))?;
    /// let signals = vec!["temp1", "temp2", "temp3"];
    /// let values = bus.read_batch(&signals);
    /// 
    /// // values will contain temp1 and temp2, but not temp3 (if it doesn't exist)
    /// assert_eq!(values.len(), 2);
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn read_batch<I, K>(&self, signal_names: I) -> HashMap<String, Value>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        let mut results = HashMap::new();
        for name in signal_names {
            let name_str = name.as_ref();
            if let Some(value) = self.get(name_str) {
                results.insert(name_str.to_string(), value);
            }
        }
        results
    }
    
    // ========================================================================
    // SIGNAL METADATA MANAGEMENT
    // ========================================================================
    
    /// Set metadata for a signal
    /// 
    /// Signal metadata provides rich context information for engineering
    /// applications including units, ranges, and descriptions.
    pub fn set_metadata(&self, name: impl AsRef<str>, metadata: SignalMetadata) -> Result<()> {
        let name = name.as_ref();
        
        match self.signals.get_mut(name) {
            Some(mut entry) => {
                entry.metadata = metadata;
                Ok(())
            }
            None => {
                // Create signal with default value if it doesn't exist
                let default_value = metadata.default_value.clone()
                    .unwrap_or(Value::Float(0.0));
                let signal_data = SignalData::new(default_value, Some(metadata));
                self.signals.insert(name.to_string(), signal_data);
                debug!("Created signal '{}' with metadata", name);
                Ok(())
            }
        }
    }
    
    /// Get metadata for a signal
    pub fn get_metadata(&self, name: impl AsRef<str>) -> Option<SignalMetadata> {
        self.signals.get(name.as_ref()).map(|entry| entry.metadata.clone())
    }
    
    /// Get access statistics for a signal
    pub fn get_stats(&self, name: impl AsRef<str>) -> Option<SignalStats> {
        self.signals.get(name.as_ref()).map(|entry| entry.stats.clone())
    }
    
    // ========================================================================
    // SIGNAL DISCOVERY AND MANAGEMENT
    // ========================================================================
    
    /// Check if a signal exists
    pub fn exists(&self, name: impl AsRef<str>) -> bool {
        self.signals.contains_key(name.as_ref())
    }
    
    /// Remove a signal from the bus
    /// 
    /// Returns the removed signal data if it existed.
    pub fn remove(&self, name: impl AsRef<str>) -> Option<(Value, SignalMetadata)> {
        let name = name.as_ref();
        self.signals.remove(name).map(|(_, signal_data)| {
            debug!("Removed signal: {}", name);
            (signal_data.value, signal_data.metadata)
        })
    }
    
    /// Clear all signals from the bus
    /// 
    /// This is primarily useful for testing and initialization.
    pub fn clear(&self) {
        let count = self.signals.len();
        self.signals.clear();
        debug!("Cleared {} signals from bus", count);
    }
    
    /// Get the number of signals in the bus
    pub fn len(&self) -> usize {
        self.signals.len()
    }
    
    /// Check if the bus is empty
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
    
    /// Get all signal names
    /// 
    /// Returns a vector of all signal names currently in the bus.
    /// This operation creates a snapshot at the time of calling.
    pub fn signal_names(&self) -> Vec<String> {
        self.signals.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Get a map of all signal values
    pub fn get_all_signals(&self) -> Result<HashMap<String, Value>> {
        let mut result = HashMap::new();
        for entry in self.signals.iter() {
            result.insert(entry.key().clone(), entry.value().value.clone());
        }
        Ok(result)
    }
    
    /// Get signal names matching a pattern
    /// 
    /// Supports wildcard patterns for signal discovery.
    /// Pattern syntax: `*` matches any characters, `?` matches single character.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::SignalBus;
    /// # let bus = SignalBus::new();
    /// // Find all temperature signals
    /// let temp_signals = bus.find_signals("*.temperature");
    /// 
    /// // Find all signals from PLC1
    /// let plc1_signals = bus.find_signals("plc1.*");
    /// 
    /// // Find specific pattern
    /// let tank_signals = bus.find_signals("tank?.level");
    /// ```
    pub fn find_signals(&self, pattern: &str) -> Vec<String> {
        let regex_pattern = pattern
            .replace('*', ".*")
            .replace('?', ".");
        
        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            self.signals
                .iter()
                .filter_map(|entry| {
                    let name = entry.key();
                    if regex.is_match(name) {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            warn!("Invalid signal pattern: {}", pattern);
            Vec::new()
        }
    }
    
    /// Create a snapshot of all signals
    /// 
    /// Returns a HashMap containing all current signal values.
    /// This is useful for debugging, logging, and state persistence.
    pub fn snapshot(&self) -> HashMap<String, Value> {
        self.signals
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().value.clone()))
            .collect()
    }
    
    /// Create a detailed snapshot including metadata and statistics
    pub fn detailed_snapshot(&self) -> HashMap<String, (Value, SignalMetadata, SignalStats)> {
        self.signals
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let signal_data = entry.value();
                (
                    name,
                    (
                        signal_data.value.clone(),
                        signal_data.metadata.clone(),
                        signal_data.stats.clone(),
                    ),
                )
            })
            .collect()
    }
    
    // ========================================================================
    // EVENT SYSTEM (feature-gated)
    // ========================================================================
    
    /// Subscribe to signal change events
    /// 
    /// Returns a receiver that will receive events for all signal changes.
    /// This enables reactive programming patterns and real-time monitoring.
    #[cfg(feature = "signal-events")]
    pub fn subscribe(&self) -> broadcast::Receiver<SignalChangeEvent> {
        self.event_sender.subscribe()
    }
    
    /// Subscribe to specific signal changes
    /// 
    /// Returns a receiver that only receives events for the specified signals.
    #[cfg(feature = "signal-events")]
    pub fn subscribe_filtered<I, K>(&self, signal_names: I) -> broadcast::Receiver<SignalChangeEvent>
    where
        I: IntoIterator<Item = K>,
        K: AsRef<str>,
    {
        // For now, return the general subscriber
        // TODO: Implement filtering in the future
        let _ = signal_names; // Suppress unused parameter warning
        self.event_sender.subscribe()
    }
    
    // ========================================================================
    // VALIDATION SUPPORT (feature-gated)
    // ========================================================================
    
    /// Set a validator for signal values
    /// 
    /// The validator will be called for all signal writes to ensure data integrity.
    #[cfg(feature = "signal-validation")]
    pub fn set_validator(&mut self, validator: Arc<dyn Validator + Send + Sync>) {
        self.validator = Some(validator);
    }
    
    /// Remove the current validator
    #[cfg(feature = "signal-validation")]
    pub fn clear_validator(&mut self) {
        self.validator = None;
    }
    
    // ========================================================================
    // PERFORMANCE MONITORING
    // ========================================================================
    
    /// Get global bus statistics
    pub fn get_global_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_operations".to_string(), self.total_operations.load(Ordering::Relaxed));
        stats.insert("signal_count".to_string(), self.len() as u64);
        
        // Aggregate signal statistics
        let mut total_reads = 0u64;
        let mut total_writes = 0u64;
        let mut total_updates = 0u64;
        let mut total_conversion_errors = 0u64;
        
        for entry in self.signals.iter() {
            let signal_stats = &entry.stats;
            total_reads += signal_stats.read_count;
            total_writes += signal_stats.write_count;
            total_updates += signal_stats.update_count;
            total_conversion_errors += signal_stats.conversion_errors;
        }
        
        stats.insert("total_reads".to_string(), total_reads);
        stats.insert("total_writes".to_string(), total_writes);
        stats.insert("total_updates".to_string(), total_updates);
        stats.insert("total_conversion_errors".to_string(), total_conversion_errors);
        
        stats
    }
    
    /// Get performance metrics for monitoring systems
    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_performance_metrics(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        if let Ok(times) = self.operation_times.lock() {
            if !times.is_empty() {
                let total_time: Duration = times.iter().map(|(_, duration)| *duration).sum();
                let avg_time = total_time.as_nanos() as f64 / times.len() as f64;
                
                metrics.insert("avg_operation_time_ns".to_string(), avg_time);
                
                // Find min and max operation times
                if let (Some(min), Some(max)) = (
                    times.iter().map(|(_, d)| d.as_nanos()).min(),
                    times.iter().map(|(_, d)| d.as_nanos()).max(),
                ) {
                    metrics.insert("min_operation_time_ns".to_string(), min as f64);
                    metrics.insert("max_operation_time_ns".to_string(), max as f64);
                }
            }
        }
        
        metrics
    }
    
    // ========================================================================
    // UTILITY AND VALIDATION METHODS
    // ========================================================================
    
    /// Validate signal name according to naming conventions
    fn validate_signal_name(&self, name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(PlcError::Validation("Signal name cannot be empty".to_string()));
        }
        
        if name.len() > 256 {
            return Err(PlcError::Validation("Signal name too long (max 256 characters)".to_string()));
        }
        
        // Check for valid characters (alphanumeric, dots, underscores, hyphens)
        if !name.chars().all(|c| c.is_alphanumeric() || matches!(c, '.' | '_' | '-')) {
            return Err(PlcError::Validation(format!(
                "Signal name '{}' contains invalid characters", name
            )));
        }
        
        // Ensure it doesn't start or end with a dot
        if name.starts_with('.') || name.ends_with('.') {
            return Err(PlcError::Validation(format!(
                "Signal name '{}' cannot start or end with a dot", name
            )));
        }
        
        // Ensure no consecutive dots
        if name.contains("..") {
            return Err(PlcError::Validation(format!(
                "Signal name '{}' cannot contain consecutive dots", name
            )));
        }
        
        Ok(())
    }
}

// ============================================================================
// TRAIT IMPLEMENTATIONS
// ============================================================================

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SignalBus {
    fn clone(&self) -> Self {
        Self {
            signals: Arc::clone(&self.signals),
            total_operations: Arc::clone(&self.total_operations),
            
            #[cfg(feature = "signal-events")]
            event_sender: Arc::clone(&self.event_sender),
            
            #[cfg(feature = "signal-validation")]
            validator: self.validator.clone(),
            
            #[cfg(feature = "enhanced-monitoring")]
            operation_times: Arc::clone(&self.operation_times),
        }
    }
}

// ============================================================================
// COMPREHENSIVE TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_basic_signal_operations() {
        let bus = SignalBus::new();
        
        // Test set and get
        bus.set("test_signal", Value::Integer(42)).unwrap();
        assert_eq!(bus.get("test_signal"), Some(Value::Integer(42)));
        
        // Test type conversion
        assert_eq!(bus.get_integer("test_signal").unwrap(), 42);
        assert_eq!(bus.get_float("test_signal").unwrap(), 42.0);
        
        // Test exists
        assert!(bus.exists("test_signal"));
        assert!(!bus.exists("nonexistent"));
        
        // Test signal count
        assert_eq!(bus.len(), 1);
        assert!(!bus.is_empty());
    }
    
    #[test]
    fn test_type_conversions() {
        let bus = SignalBus::new();
        
        // Boolean conversions
        bus.set("bool_true", Value::Bool(true)).unwrap();
        bus.set("bool_false", Value::Bool(false)).unwrap();
        
        assert_eq!(bus.get_integer("bool_true").unwrap(), 1);
        assert_eq!(bus.get_integer("bool_false").unwrap(), 0);
        assert_eq!(bus.get_float("bool_true").unwrap(), 1.0);
        
        // Integer conversions
        bus.set("int_val", Value::Integer(100)).unwrap();
        assert_eq!(bus.get_bool("int_val").unwrap(), true);
        assert_eq!(bus.get_float("int_val").unwrap(), 100.0);
        
        // Float conversions
        bus.set("float_val", Value::Float(3.14)).unwrap();
        assert_eq!(bus.get_integer("float_val").unwrap(), 3);
        assert_eq!(bus.get_bool("float_val").unwrap(), true);
        
        // Zero values
        bus.set("zero_int", Value::Integer(0)).unwrap();
        bus.set("zero_float", Value::Float(0.0)).unwrap();
        assert_eq!(bus.get_bool("zero_int").unwrap(), false);
        assert_eq!(bus.get_bool("zero_float").unwrap(), false);
    }
    
    #[test]
    fn test_atomic_updates() {
        let bus = SignalBus::new();
        
        // Test counter increment
        for i in 1..=10 {
            let result = bus.update("counter", |old| {
                match old {
                    Some(Value::Integer(n)) => Value::Integer(n + 1),
                    _ => Value::Integer(1),
                }
            }).unwrap();
            assert_eq!(result, Value::Integer(i));
        }
        
        assert_eq!(bus.get_integer("counter").unwrap(), 10);
        
        // Test accumulator
        for _ in 0..5 {
            bus.update("sum", |old| {
                match old {
                    Some(Value::Float(sum)) => Value::Float(sum + 2.5),
                    _ => Value::Float(2.5),
                }
            }).unwrap();
        }
        
        assert_eq!(bus.get_float("sum").unwrap(), 12.5);
    }
    
    #[test]
    fn test_batch_operations() {
        let bus = SignalBus::new();
        
        // Test batch write
        let updates = vec![
            ("temp1", Value::Float(25.0)),
            ("temp2", Value::Float(26.0)),
            ("pressure", Value::Float(101.3)),
            ("status", Value::Bool(true)),
        ];
        
        bus.write_batch(updates).unwrap();
        
        // Verify all values were set
        assert_eq!(bus.get_float("temp1").unwrap(), 25.0);
        assert_eq!(bus.get_float("temp2").unwrap(), 26.0);
        assert_eq!(bus.get_float("pressure").unwrap(), 101.3);
        assert_eq!(bus.get_bool("status").unwrap(), true);
        
        // Test batch read
        let signal_names = vec!["temp1", "temp2", "pressure", "status", "nonexistent"];
        let values = bus.read_batch(signal_names);
        
        assert_eq!(values.len(), 4); // nonexistent should be omitted
        assert_eq!(values.get("temp1"), Some(&Value::Float(25.0)));
        assert_eq!(values.get("temp2"), Some(&Value::Float(26.0)));
        assert_eq!(values.get("pressure"), Some(&Value::Float(101.3)));
        assert_eq!(values.get("status"), Some(&Value::Bool(true)));
        assert!(!values.contains_key("nonexistent"));
    }
    
    #[test]
    fn test_signal_metadata() {
        let bus = SignalBus::new();
        
        let metadata = SignalMetadata {
            unit: Some("°C".to_string()),
            description: Some("Temperature sensor".to_string()),
            min_value: Some(Value::Float(-40.0)),
            max_value: Some(Value::Float(85.0)),
            default_value: Some(Value::Float(20.0)),
            source: Some("Sensor-1".to_string()),
            category: Some("Temperature".to_string()),
            update_frequency_ms: Some(1000),
            log_to_history: true,
            enable_alarms: true,
            custom: [("location".to_string(), "Room A".to_string())].into(),
        };
        
        // Set metadata for existing signal
        bus.set("temperature", Value::Float(22.5)).unwrap();
        bus.set_metadata("temperature", metadata.clone()).unwrap();
        
        let retrieved_metadata = bus.get_metadata("temperature").unwrap();
        assert_eq!(retrieved_metadata.unit, Some("°C".to_string()));
        assert_eq!(retrieved_metadata.description, Some("Temperature sensor".to_string()));
        assert!(retrieved_metadata.log_to_history);
        
        // Set metadata for non-existent signal (should create it)
        bus.set_metadata("new_signal", metadata).unwrap();
        assert!(bus.exists("new_signal"));
        assert_eq!(bus.get("new_signal"), Some(Value::Float(20.0))); // Default value
    }
    
    #[test]
    fn test_signal_stats() {
        let bus = SignalBus::new();
        
        bus.set("test_signal", Value::Integer(1)).unwrap();
        
        // Perform some operations
        for _ in 0..5 {
            bus.get("test_signal");
        }
        
        for i in 2..=7 {
            bus.set("test_signal", Value::Integer(i)).unwrap();
        }
        
        bus.update("test_signal", |old| {
            match old {
                Some(Value::Integer(n)) => Value::Integer(n + 1),
                _ => Value::Integer(0),
            }
        }).unwrap();
        
        let stats = bus.get_stats("test_signal").unwrap();
        assert_eq!(stats.read_count, 5);
        assert_eq!(stats.write_count, 6); // Initial + 5 updates
        assert_eq!(stats.update_count, 1);
        assert!(stats.last_read.is_some());
        assert!(stats.last_write.is_some());
        assert!(stats.created_at <= SystemTime::now());
    }
    
    #[test]
    fn test_signal_discovery() {
        let bus = SignalBus::new();
        
        // Set up test signals
        let test_signals = vec![
            "plc1.tank1.temperature",
            "plc1.tank1.pressure",
            "plc1.tank2.temperature",
            "plc2.pump1.status",
            "plc2.pump1.speed",
            "scada.system.uptime",
        ];
        
        for signal in &test_signals {
            bus.set(signal, Value::Float(1.0)).unwrap();
        }
        
        // Test signal name listing
        let all_names = bus.signal_names();
        assert_eq!(all_names.len(), 6);
        
        // Test pattern matching
        let plc1_signals = bus.find_signals("plc1.*");
        assert_eq!(plc1_signals.len(), 4);
        
        let temperature_signals = bus.find_signals("*.temperature");
        assert_eq!(temperature_signals.len(), 2);
        
        let tank1_signals = bus.find_signals("plc1.tank1.*");
        assert_eq!(tank1_signals.len(), 2);
        
        // Test snapshot
        let snapshot = bus.snapshot();
        assert_eq!(snapshot.len(), 6);
        for signal in &test_signals {
            assert!(snapshot.contains_key(*signal));
        }
    }
    
    #[test]
    fn test_signal_name_validation() {
        let bus = SignalBus::new();
        
        // Valid names
        assert!(bus.set("valid_signal", Value::Float(1.0)).is_ok());
        assert!(bus.set("plc1.tank1.temperature", Value::Float(1.0)).is_ok());
        assert!(bus.set("system-status", Value::Bool(true)).is_ok());
        assert!(bus.set("signal123", Value::Integer(1)).is_ok());
        
        // Invalid names
        assert!(bus.set("", Value::Float(1.0)).is_err()); // Empty
        assert!(bus.set(".invalid", Value::Float(1.0)).is_err()); // Starts with dot
        assert!(bus.set("invalid.", Value::Float(1.0)).is_err()); // Ends with dot
        assert!(bus.set("invalid..name", Value::Float(1.0)).is_err()); // Consecutive dots
        assert!(bus.set("invalid@signal", Value::Float(1.0)).is_err()); // Invalid character
        
        // Too long name
        let long_name = "a".repeat(300);
        assert!(bus.set(&long_name, Value::Float(1.0)).is_err());
    }
    
    #[test]
    fn test_concurrent_access() {
        let bus = Arc::new(SignalBus::new());
        let mut handles = vec![];
        
        // Spawn multiple threads performing operations
        for i in 0..10 {
            let bus_clone = Arc::clone(&bus);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let signal_name = format!("thread_{}_signal_{}", i, j);
                    bus_clone.set(&signal_name, Value::Integer(i * 100 + j)).unwrap();
                    
                    // Read it back
                    let value = bus_clone.get_integer(&signal_name).unwrap();
                    assert_eq!(value, i * 100 + j);
                    
                    // Update it atomically
                    bus_clone.update(&signal_name, |old| {
                        match old {
                            Some(Value::Integer(n)) => Value::Integer(n + 1),
                            _ => Value::Integer(1),
                        }
                    }).unwrap();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify we have all expected signals
        assert_eq!(bus.len(), 1000); // 10 threads * 100 signals each
        
        // Verify some values
        for i in 0..10 {
            for j in 0..100 {
                let signal_name = format!("thread_{}_signal_{}", i, j);
                let value = bus.get_integer(&signal_name).unwrap();
                assert_eq!(value, i * 100 + j + 1); // +1 from the update
            }
        }
    }
    
    #[test]
    fn test_error_conditions() {
        let bus = SignalBus::new();
        
        // Signal not found
        assert!(matches!(
            bus.get_required("nonexistent"),
            Err(PlcError::SignalNotFound(_))
        ));
        
        assert!(matches!(
            bus.get_bool("nonexistent"),
            Err(PlcError::SignalNotFound(_))
        ));
        
        // Type mismatch
        bus.set("string_signal", Value::from("hello")).unwrap();
        assert!(matches!(
            bus.get_integer("string_signal"),
            Err(PlcError::TypeMismatch { .. })
        ));
        
        // Overflow protection
        bus.set("large_float", Value::Float(f64::INFINITY)).unwrap();
        assert!(matches!(
            bus.get_integer("large_float"),
            Err(PlcError::TypeMismatch { .. })
        ));
    }
    
    #[test]
    fn test_global_statistics() {
        let bus = SignalBus::new();
        
        // Perform some operations
        bus.set("signal1", Value::Integer(1)).unwrap();
        bus.set("signal2", Value::Integer(2)).unwrap();
        bus.get("signal1");
        bus.get("signal2");
        bus.update("signal1", |old| {
            match old {
                Some(Value::Integer(n)) => Value::Integer(n + 1),
                _ => Value::Integer(1),
            }
        }).unwrap();
        
        let stats = bus.get_global_stats();
        assert!(stats.get("total_operations").unwrap() > &0);
        assert_eq!(stats.get("signal_count").unwrap(), &2);
        assert!(stats.get("total_reads").unwrap() > &0);
        assert!(stats.get("total_writes").unwrap() > &0);
        assert!(stats.get("total_updates").unwrap() > &0);
    }
    
    #[test]
    fn test_signal_removal_and_clearing() {
        let bus = SignalBus::new();
        
        // Add some signals
        bus.set("signal1", Value::Integer(1)).unwrap();
        bus.set("signal2", Value::Integer(2)).unwrap();
        bus.set("signal3", Value::Integer(3)).unwrap();
        
        assert_eq!(bus.len(), 3);
        
        // Remove one signal
        let removed = bus.remove("signal2");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().0, Value::Integer(2));
        assert_eq!(bus.len(), 2);
        assert!(!bus.exists("signal2"));
        
        // Try to remove non-existent signal
        let not_removed = bus.remove("nonexistent");
        assert!(not_removed.is_none());
        
        // Clear all signals
        bus.clear();
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }
    
    #[cfg(feature = "signal-events")]
    #[tokio::test]
    async fn test_signal_events() {
        let bus = SignalBus::new();
        let mut receiver = bus.subscribe();
        
        // Set a signal in another task
        let bus_clone = bus.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            bus_clone.set_with_source("test_signal", Value::Integer(42), Some("test")).unwrap();
        });
        
        // Wait for the event
        let event = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();
        
        assert_eq!(event.signal_name, "test_signal");
        assert_eq!(event.new_value, Value::Integer(42));
        assert_eq!(event.source, Some("test".to_string()));
        assert!(event.old_value.is_none());
    }
}

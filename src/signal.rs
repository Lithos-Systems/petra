// src/signal.rs - Fixed signal bus implementation
use crate::{error::{PlcError, Result}, value::Value};
use dashmap::DashMap;
use std::sync::Arc;
use tracing::trace;

#[cfg(feature = "quality-codes")]
use crate::value::Quality;

/// Thread-safe signal bus for inter-block communication
/// 
/// The signal bus is the central data exchange mechanism in PETRA,
/// providing lock-free concurrent access to signals.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{SignalBus, Value};
/// 
/// let bus = SignalBus::new();
/// 
/// // Set a signal value
/// bus.set("temperature", Value::Float(23.5))?;
/// 
/// // Get a signal value
/// let temp = bus.get_float("temperature")?;
/// assert_eq!(temp, 23.5);
/// 
/// // Atomic update
/// bus.update("counter", |old| {
///     match old {
///         Some(Value::Int(n)) => Value::Int(n + 1),
///         _ => Value::Int(1),
///     }
/// })?;
/// # Ok::<(), petra::PlcError>(())
/// ```
#[derive(Debug, Clone)]
pub struct SignalBus {
    signals: Arc<DashMap<String, Value>>,
}

impl SignalBus {
    /// Create a new signal bus
    pub fn new() -> Self {
        Self {
            signals: Arc::new(DashMap::new()),
        }
    }
    
    /// Set a signal value
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// bus.set("my_signal", Value::Bool(true))?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn set(&self, name: impl AsRef<str>, value: Value) -> Result<()> {
        let name = name.as_ref();
        trace!("Setting signal {} = {:?}", name, value);
        self.signals.insert(name.to_string(), value);
        Ok(())
    }
    
    /// Get a signal value
    /// 
    /// Returns `None` if the signal doesn't exist.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("my_signal", Value::Int(42))?;
    /// if let Some(value) = bus.get("my_signal") {
    ///     println!("Signal value: {:?}", value);
    /// }
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn get(&self, name: impl AsRef<str>) -> Option<Value> {
        let name = name.as_ref();
        self.signals.get(name).map(|entry| entry.value().clone())
    }
    
    /// Atomically update a signal value
    /// 
    /// The updater function receives the current value (if any) and returns the new value.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// // Increment a counter atomically
    /// bus.update("counter", |old| {
    ///     match old {
    ///         Some(Value::Int(n)) => Value::Int(n + 1),
    ///         _ => Value::Int(1),
    ///     }
    /// })?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn update<F>(&self, name: impl AsRef<str>, updater: F) -> Result<()>
    where
        F: FnOnce(Option<Value>) -> Value,
    {
        let name = name.as_ref();
        self.signals.alter(name, |_, old| updater(Some(old)));
        Ok(())
    }
    
    /// Get a boolean signal value
    /// 
    /// Performs type conversion where appropriate.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("enabled", Value::Bool(true))?;
    /// let enabled = bus.get_bool("enabled")?;
    /// assert_eq!(enabled, true);
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.get(name) {
            Some(Value::Bool(b)) => Ok(b),
            Some(Value::Int(i)) => Ok(i != 0),
            Some(Value::Float(f)) => Ok(f != 0.0 && !f.is_nan()),
            #[cfg(feature = "extended-types")]
            Some(Value::String(s)) => {
                match s.to_lowercase().as_str() {
                    "true" | "yes" | "on" | "1" => Ok(true),
                    "false" | "no" | "off" | "0" => Ok(false),
                    _ => Err(PlcError::TypeMismatch {
                        expected: "bool".to_string(),
                        actual: format!("string '{}'", s),
                    }),
                }
            }
            #[cfg(feature = "quality-codes")]
            Some(Value::QualityValue { value, quality, .. }) => {
                if quality.is_good() {
                    // Recursively call get_bool on the wrapped value
                    match value.as_ref() {
                        Value::Bool(b) => Ok(*b),
                        Value::Int(i) => Ok(*i != 0),
                        Value::Float(f) => Ok(*f != 0.0 && !f.is_nan()),
                        _ => Err(PlcError::TypeMismatch {
                            expected: "bool".to_string(),
                            actual: value.type_name().to_string(),
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch {
                expected: "bool".to_string(),
                actual: v.type_name().to_string(),
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get an integer signal value
    /// 
    /// Performs type conversion where appropriate.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("count", Value::Int(42))?;
    /// let count = bus.get_int("count")?;
    /// assert_eq!(count, 42);
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn get_int(&self, name: &str) -> Result<i32> {
        match self.get(name) {
            Some(Value::Int(i)) => Ok(i as i32),
            Some(Value::Float(f)) => {
                if f.is_finite() && f >= i32::MIN as f64 && f <= i32::MAX as f64 {
                    Ok(f as i32)
                } else {
                    Err(PlcError::Runtime(format!("Float value {} out of i32 range", f)))
                }
            }
            Some(Value::Bool(b)) => Ok(if b { 1 } else { 0 }),
            #[cfg(feature = "extended-types")]
            Some(Value::String(s)) => {
                s.parse::<i32>()
                    .map_err(|e| PlcError::TypeMismatch {
                        expected: "int".to_string(),
                        actual: format!("string (parse error: {})", e),
                    })
            }
            #[cfg(feature = "quality-codes")]
            Some(Value::QualityValue { value, quality, .. }) => {
                if quality.is_good() {
                    // Recursively call get_int on the wrapped value
                    match value.as_ref() {
                        Value::Int(i) => Ok(*i as i32),
                        Value::Float(f) => {
                            if f.is_finite() && *f >= i32::MIN as f64 && *f <= i32::MAX as f64 {
                                Ok(*f as i32)
                            } else {
                                Err(PlcError::Runtime(format!("Float value {} out of i32 range", f)))
                            }
                        }
                        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
                        _ => Err(PlcError::TypeMismatch {
                            expected: "int".to_string(),
                            actual: value.type_name().to_string(),
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch {
                expected: "int".to_string(),
                actual: v.type_name().to_string(),
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a float signal value
    /// 
    /// Performs type conversion where appropriate.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::{SignalBus, Value};
    /// # let bus = SignalBus::new();
    /// # bus.set("temperature", Value::Float(23.5))?;
    /// let temp = bus.get_float("temperature")?;
    /// assert_eq!(temp, 23.5);
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn get_float(&self, name: &str) -> Result<f64> {
        match self.get(name) {
            Some(Value::Float(f)) => Ok(f),
            Some(Value::Int(i)) => Ok(i as f64),
            Some(Value::Bool(b)) => Ok(if b { 1.0 } else { 0.0 }),
            #[cfg(feature = "extended-types")]
            Some(Value::String(s)) => {
                s.parse::<f64>()
                    .map_err(|e| PlcError::TypeMismatch {
                        expected: "float".to_string(),
                        actual: format!("string (parse error: {})", e),
                    })
            }
            #[cfg(feature = "engineering-types")]
            Some(Value::Engineering { value, .. }) => Ok(value),
            #[cfg(feature = "quality-codes")]
            Some(Value::QualityValue { value, quality, .. }) => {
                if quality.is_good() {
                    // Recursively call get_float on the wrapped value
                    match value.as_ref() {
                        Value::Float(f) => Ok(*f),
                        Value::Int(i) => Ok(*i as f64),
                        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
                        #[cfg(feature = "engineering-types")]
                        Value::Engineering { value, .. } => Ok(*value),
                        _ => Err(PlcError::TypeMismatch {
                            expected: "float".to_string(),
                            actual: value.type_name().to_string(),
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch {
                expected: "float".to_string(),
                actual: v.type_name().to_string(),
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a string signal value
    #[cfg(feature = "extended-types")]
    pub fn get_string(&self, name: &str) -> Result<String> {
        match self.get(name) {
            Some(Value::String(s)) => Ok(s),
            Some(v) => Ok(v.as_string()),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Check if a signal exists
    pub fn exists(&self, name: &str) -> bool {
        self.signals.contains_key(name)
    }
    
    /// Remove a signal from the bus
    pub fn remove(&self, name: &str) -> Option<Value> {
        self.signals.remove(name).map(|(_, v)| v)
    }
    
    /// Clear all signals
    pub fn clear(&self) {
        self.signals.clear();
    }
    
    /// Get the number of signals
    pub fn len(&self) -> usize {
        self.signals.len()
    }
    
    /// Check if the bus is empty
    pub fn is_empty(&self) -> bool {
        self.signals.is_empty()
    }
    
    /// Get all signal names
    pub fn signal_names(&self) -> Vec<String> {
        self.signals.iter().map(|entry| entry.key().clone()).collect()
    }
    
    /// Create a snapshot of all signals
    pub fn snapshot(&self) -> std::collections::HashMap<String, Value> {
        self.signals
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_bus_basic() {
        let bus = SignalBus::new();
        
        // Test set and get
        bus.set("test", Value::Int(42)).unwrap();
        assert_eq!(bus.get("test"), Some(Value::Int(42)));
        
        // Test get_int
        assert_eq!(bus.get_int("test").unwrap(), 42);
        
        // Test exists
        assert!(bus.exists("test"));
        assert!(!bus.exists("nonexistent"));
    }

    #[test]
    fn test_signal_bus_type_conversion() {
        let bus = SignalBus::new();
        
        // Int to float
        bus.set("int_val", Value::Int(10)).unwrap();
        assert_eq!(bus.get_float("int_val").unwrap(), 10.0);
        
        // Bool to int
        bus.set("bool_val", Value::Bool(true)).unwrap();
        assert_eq!(bus.get_int("bool_val").unwrap(), 1);
        
        // Float to int (within range)
        bus.set("float_val", Value::Float(42.7)).unwrap();
        assert_eq!(bus.get_int("float_val").unwrap(), 42);
    }

    #[test]
    fn test_signal_bus_atomic_update() {
        let bus = SignalBus::new();
        
        // Initialize counter
        bus.set("counter", Value::Int(0)).unwrap();
        
        // Increment counter
        bus.update("counter", |old| {
            match old {
                Some(Value::Int(n)) => Value::Int(n + 1),
                _ => Value::Int(1),
            }
        }).unwrap();
        
        assert_eq!(bus.get_int("counter").unwrap(), 1);
    }

    #[test]
    fn test_signal_bus_errors() {
        let bus = SignalBus::new();
        
        // Test signal not found
        assert!(matches!(
            bus.get_int("nonexistent"),
            Err(PlcError::SignalNotFound(_))
        ));
        
        // Test type mismatch
        bus.set("string_val", Value::Float(3.14)).unwrap();
        bus.set("bad_bool", Value::Int(42)).unwrap();
        
        // Float should convert to bool
        assert_eq!(bus.get_bool("string_val").unwrap(), true);
    }

    #[cfg(feature = "extended-types")]
    #[test]
    fn test_string_conversions() {
        let bus = SignalBus::new();
        
        // String to bool
        bus.set("str_true", Value::String("true".to_string())).unwrap();
        assert_eq!(bus.get_bool("str_true").unwrap(), true);
        
        bus.set("str_false", Value::String("false".to_string())).unwrap();
        assert_eq!(bus.get_bool("str_false").unwrap(), false);
        
        // String to int
        bus.set("str_num", Value::String("123".to_string())).unwrap();
        assert_eq!(bus.get_int("str_num").unwrap(), 123);
        
        // String to float
        bus.set("str_float", Value::String("3.14".to_string())).unwrap();
        assert_eq!(bus.get_float("str_float").unwrap(), 3.14);
    }
}

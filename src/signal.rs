// src/signal.rs - Complete implementation with all feature flags

use crate::error::{PlcError, Result};
use crate::value::Value;

#[cfg(not(feature = "optimized"))]
use dashmap::DashMap;

#[cfg(feature = "optimized")]
use parking_lot::RwLock;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "enhanced")]
use chrono::{DateTime, Utc};

#[cfg(feature = "async")]
use tokio::sync::broadcast;

// ============================================================================
// SIGNAL BUS CORE
// ============================================================================

/// Thread-safe signal bus for PLC communication
#[derive(Clone)]
pub struct SignalBus {
    #[cfg(not(feature = "optimized"))]
    signals: Arc<DashMap<String, Value>>,
    
    #[cfg(feature = "optimized")]
    signals: Arc<RwLock<HashMap<String, Value>>>,
    
    #[cfg(feature = "async")]
    updates: Arc<broadcast::Sender<SignalUpdate>>,
    
    #[cfg(feature = "enhanced")]
    metadata: Arc<DashMap<String, SignalMetadata>>,
    
    total_updates: Arc<AtomicU64>,
}

/// Signal update notification
#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct SignalUpdate {
    pub name: String,
    pub value: Value,
    pub timestamp: DateTime<Utc>,
}

/// Signal metadata for enhanced monitoring
#[cfg(feature = "enhanced")]
#[derive(Debug, Clone)]
pub struct SignalMetadata {
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub update_count: u64,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

// ============================================================================
// SIGNAL BUS IMPLEMENTATION
// ============================================================================

impl SignalBus {
    /// Create a new signal bus
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "optimized"))]
            signals: Arc::new(DashMap::new()),
            
            #[cfg(feature = "optimized")]
            signals: Arc::new(RwLock::new(HashMap::new())),
            
            #[cfg(feature = "async")]
            updates: Arc::new(broadcast::channel(1024).0),
            
            #[cfg(feature = "enhanced")]
            metadata: Arc::new(DashMap::new()),
            
            total_updates: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Set a signal value
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        self.total_updates.fetch_add(1, Ordering::Relaxed);
        
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.insert(name.to_string(), value.clone());
        }
        
        #[cfg(feature = "optimized")]
        {
            let mut signals = self.signals.write();
            signals.insert(name.to_string(), value.clone());
        }
        
        #[cfg(feature = "enhanced")]
        {
            let now = Utc::now();
            self.metadata
                .entry(name.to_string())
                .and_modify(|meta| {
                    meta.last_updated = now;
                    meta.update_count += 1;
                })
                .or_insert_with(|| SignalMetadata {
                    created: now,
                    last_updated: now,
                    update_count: 1,
                    description: None,
                    tags: Vec::new(),
                });
        }
        
        #[cfg(feature = "async")]
        {
            let update = SignalUpdate {
                name: name.to_string(),
                value: value.clone(),
                timestamp: Utc::now(),
            };
            let _ = self.updates.send(update);
        }
        
        Ok(())
    }

    /// Get a signal value
    pub fn get(&self, name: &str) -> Option<Value> {
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.get(name).map(|entry| entry.value().clone())
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            signals.get(name).cloned()
        }
    }

    /// Remove a signal
    pub fn remove(&self, name: &str) -> Option<Value> {
        #[cfg(feature = "enhanced")]
        {
            self.metadata.remove(name);
        }
        
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.remove(name).map(|(_, v)| v)
        }
        
        #[cfg(feature = "optimized")]
        {
            let mut signals = self.signals.write();
            signals.remove(name)
        }
    }

    /// Clear all signals
    pub fn clear(&self) {
        #[cfg(feature = "enhanced")]
        {
            self.metadata.clear();
        }
        
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.clear();
        }
        
        #[cfg(feature = "optimized")]
        {
            let mut signals = self.signals.write();
            signals.clear();
        }
        
        self.total_updates.store(0, Ordering::Relaxed);
    }

    /// Get the number of signals
    pub fn len(&self) -> usize {
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.len()
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            signals.len()
        }
    }

    /// Check if the signal bus is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all signal names
    pub fn keys(&self) -> Vec<String> {
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.iter().map(|entry| entry.key().clone()).collect()
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            signals.keys().cloned().collect()
        }
    }

    /// Subscribe to signal updates
    #[cfg(feature = "async")]
    pub fn subscribe(&self) -> broadcast::Receiver<SignalUpdate> {
        self.updates.subscribe()
    }

    // ============================================================================
    // TYPED GETTER METHODS (required by blocks)
    // ============================================================================
    
    /// Get a boolean signal value
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.get(name) {
            Some(Value::Bool(b)) => Ok(b),
            Some(Value::Int(i)) => Ok(i != 0),
            Some(Value::Float(f)) => Ok(f != 0.0),
            #[cfg(feature = "extended-types")]
            Some(Value::String(s)) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Ok(true),
                "false" | "0" | "no" | "off" => Ok(false),
                _ => Err(PlcError::TypeMismatch(format!(
                    "Expected bool but got {}",
                    v.type_name()
                }),
            },
            #[cfg(feature = "quality-codes")]
            Some(Value::QualityValue { value, quality, .. }) => {
                if quality.is_good() {
                    // Recursively call get_bool on the wrapped value
                    match value.as_ref() {
                        Value::Bool(b) => Ok(*b),
                        Value::Int(i) => Ok(*i != 0),
                        Value::Float(f) => Ok(*f != 0.0),
                        _ => Err(PlcError::TypeMismatch(format!(
                            "Expected bool but got {}",
                            v.type_name()
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch(format!(
                "Expected bool but got {}",
                v.type_name()
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get an integer signal value  
    pub fn get_int(&self, name: &str) -> Result<i32> {
        match self.get(name) {
            Some(Value::Int(i)) => Ok(i),
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
                        Value::Int(i) => Ok(*i),
                        Value::Float(f) => {
                            if f.is_finite() && *f >= i32::MIN as f64 && *f <= i32::MAX as f64 {
                                Ok(*f as i32)
                            } else {
                                Err(PlcError::Runtime(format!("Float value {} out of i32 range", f)))
                            }
                        }
                        Value::Bool(b) => Ok(if *b { 1 } else { 0 }),
                        _ => Err(PlcError::TypeMismatch(format!(
                        "Expected bool but got {}",
                        v.type_name()
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch(format!(
                    "Expected bool but got {}",
                    v.type_name()
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a float signal value
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
                        _ => EErr(PlcError::TypeMismatch(format!(
                        "Expected bool but got {}",
                        v.type_name()
                        }),
                    }
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Err(PlcError::TypeMismatch(format!(
                    "Expected bool but got {}",
                    v.type_name()
            }),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    /// Get a string signal value (feature-gated)
    #[cfg(feature = "extended-types")]
    pub fn get_string(&self, name: &str) -> Result<String> {
        match self.get(name) {
            Some(Value::String(s)) => Ok(s),
            Some(Value::Int(i)) => Ok(i.to_string()),
            Some(Value::Float(f)) => Ok(f.to_string()),
            Some(Value::Bool(b)) => Ok(b.to_string()),
            #[cfg(feature = "engineering-types")]
            Some(Value::Engineering { value, unit, .. }) => Ok(format!("{} {}", value, unit)),
            #[cfg(feature = "quality-codes")]
            Some(Value::QualityValue { value, quality, .. }) => {
                if quality.is_good() {
                    // Recursively call as_string on the wrapped value
                    Ok(value.as_string())
                } else {
                    Err(PlcError::SignalQuality(format!("Signal '{}' has bad quality: {:?}", name, quality)))
                }
            }
            Some(v) => Ok(v.as_string()),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }

    // ============================================================================
    // ENHANCED SIGNAL OPERATIONS (feature-gated)
    // ============================================================================

    /// Set signal with quality code (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub fn set_with_quality(&self, name: &str, value: Value, quality: crate::value::QualityCode) -> Result<()> {
        let qualified_value = Value::QualityValue {
            value: Box::new(value),
            quality,
            timestamp: chrono::Utc::now(),
            source: None,
        };
        self.set(name, qualified_value)
    }

    /// Set signal with quality and source (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub fn set_with_quality_and_source(&self, name: &str, value: Value, quality: crate::value::QualityCode, source: String) -> Result<()> {
        let qualified_value = Value::QualityValue {
            value: Box::new(value),
            quality,
            timestamp: chrono::Utc::now(),
            source: Some(source),
        };
        self.set(name, qualified_value)
    }

    /// Get signal quality (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub fn get_quality(&self, name: &str) -> Option<crate::value::QualityCode> {
        match self.get(name) {
            Some(Value::QualityValue { quality, .. }) => Some(quality),
            _ => None,
        }
    }

    /// Check if signal has good quality (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub fn is_quality_good(&self, name: &str) -> bool {
        self.get_quality(name)
            .map(|q| q.is_good())
            .unwrap_or(false)
    }

    /// Set engineering value with units (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    pub fn set_engineering(&self, name: &str, value: f64, unit: String) -> Result<()> {
        let eng_value = Value::Engineering {
            value,
            unit,
            #[cfg(feature = "unit-conversion")]
            base_unit: None,
            #[cfg(feature = "unit-conversion")]
            scale_factor: None,
        };
        self.set(name, eng_value)
    }

    /// Set engineering value with unit conversion (requires unit-conversion feature)
    #[cfg(feature = "unit-conversion")]
    pub fn set_engineering_with_conversion(&self, name: &str, value: f64, unit: String, base_unit: String, scale_factor: f64) -> Result<()> {
        let eng_value = Value::Engineering {
            value,
            unit,
            base_unit: Some(base_unit),
            scale_factor: Some(scale_factor),
        };
        self.set(name, eng_value)
    }

    /// Get engineering value (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    pub fn get_engineering(&self, name: &str) -> Option<(f64, String)> {
        match self.get(name) {
            Some(Value::Engineering { value, unit, .. }) => Some((value, unit)),
            _ => None,
        }
    }

    /// Convert engineering units (requires unit-conversion feature)
    #[cfg(feature = "unit-conversion")]
    pub fn convert_units(&self, name: &str, target_unit: &str) -> Result<Option<f64>> {
        match self.get(name) {
            Some(Value::Engineering { value, unit, base_unit, scale_factor, .. }) => {
                // Simplified unit conversion - in practice this would use a proper unit library
                if unit == target_unit {
                    Ok(Some(value))
                } else if let (Some(base), Some(factor)) = (base_unit, scale_factor) {
                    if base == target_unit {
                        Ok(Some(value * factor))
                    } else {
                        // More complex conversion would be implemented here
                        Err(PlcError::SignalNotFound(format!("Cannot convert from {} to {}", unit, target_unit)))
                    }
                } else {
                    Err(PlcError::SignalNotFound(format!("No conversion available from {} to {}", unit, target_unit)))
                }
            }
            _ => Ok(None),
        }
    }

    // ========================================================================
    // ENHANCED MONITORING (feature-gated)
    // ========================================================================

    /// Get signal metadata (requires enhanced-monitoring feature)
    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_metadata(&self, name: &str) -> Option<SignalMetadata> {
        #[cfg(feature = "enhanced")]
        {
            self.metadata.get(name).map(|meta| meta.clone())
        }
        #[cfg(not(feature = "enhanced"))]
        {
            None
        }
    }

    /// Get signal update count (requires enhanced-monitoring feature)
    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_update_count(&self, name: &str) -> Option<u64> {
        #[cfg(feature = "enhanced")]
        {
            self.metadata.get(name).map(|meta| meta.update_count)
        }
        #[cfg(not(feature = "enhanced"))]
        {
            None
        }
    }

    /// Get last update time (requires enhanced-monitoring feature)
    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_last_update(&self, name: &str) -> Option<chrono::DateTime<chrono::Utc>> {
        #[cfg(feature = "enhanced")]
        {
            self.metadata.get(name).map(|meta| meta.last_updated)
        }
        #[cfg(not(feature = "enhanced"))]
        {
            None
        }
    }

    /// Get signals with quality issues (requires quality-codes feature)
    #[cfg(feature = "quality-codes")]
    pub fn get_bad_quality_signals(&self) -> Vec<String> {
        let mut bad_signals = Vec::new();
        
        #[cfg(not(feature = "optimized"))]
        {
            for entry in self.signals.iter() {
                if let Value::QualityValue { quality, .. } = entry.value() {
                    if quality.is_bad() {
                        bad_signals.push(entry.key().clone());
                    }
                }
            }
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            for (name, value) in signals.iter() {
                if let Value::QualityValue { quality, .. } = value {
                    if quality.is_bad() {
                        bad_signals.push(name.clone());
                    }
                }
            }
        }
        
        bad_signals
    }

    /// Get engineering signals (requires engineering-types feature)
    #[cfg(feature = "engineering-types")]
    pub fn get_engineering_signals(&self) -> Vec<(String, f64, String)> {
        let mut eng_signals = Vec::new();
        
        #[cfg(not(feature = "optimized"))]
        {
            for entry in self.signals.iter() {
                if let Value::Engineering { value, unit, .. } = entry.value() {
                    eng_signals.push((entry.key().clone(), *value, unit.clone()));
                }
            }
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            for (name, value) in signals.iter() {
                if let Value::Engineering { value: val, unit, .. } = value {
                    eng_signals.push((name.clone(), *val, unit.clone()));
                }
            }
        }
        
        eng_signals
    }

    /// Get memory usage for signal bus (requires enhanced-monitoring feature)
    #[cfg(feature = "enhanced-monitoring")]
    pub fn memory_usage(&self) -> u64 {
        // Rough estimation of memory usage
        let signal_count = self.len();
        let avg_signal_size = 64; // Rough estimate per signal
        let metadata_size = if cfg!(feature = "enhanced") {
            signal_count * 128 // Rough estimate per metadata entry
        } else {
            0
        };
        
        (signal_count * avg_signal_size + metadata_size) as u64
    }

    /// Export signals with metadata (requires enhanced-monitoring feature)
    #[cfg(feature = "enhanced-monitoring")]
    pub fn export_with_metadata(&self) -> Vec<SignalExport> {
        let mut exports = Vec::new();
        
        #[cfg(not(feature = "optimized"))]
        {
            for entry in self.signals.iter() {
                let name = entry.key().clone();
                let value = entry.value().clone();
                
                #[cfg(feature = "enhanced")]
                let metadata = self.metadata.get(&name).map(|m| m.clone());
                #[cfg(not(feature = "enhanced"))]
                let metadata = None;
                
                exports.push(SignalExport {
                    name,
                    value,
                    metadata,
                });
            }
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            for (name, value) in signals.iter() {
                #[cfg(feature = "enhanced")]
                let metadata = self.metadata.get(name).map(|m| m.clone());
                #[cfg(not(feature = "enhanced"))]
                let metadata = None;
                
                exports.push(SignalExport {
                    name: name.clone(),
                    value: value.clone(),
                    metadata,
                });
            }
        }
        
        exports
    }

    // ========================================================================
    // VALIDATION INTEGRATION (feature-gated)
    // ========================================================================

    /// Set signal with validation (requires validation feature)
    #[cfg(feature = "validation")]
    pub fn set_with_validation(&self, name: &str, value: Value, validator: &dyn Fn(&Value) -> crate::validation::ValidationResult) -> Result<()> {
        let validation_result = validator(&value);
        
        if !validation_result.valid {
            return Err(PlcError::Validation(format!(
                "Validation failed for signal '{}': {}",
                name,
                validation_result.errors.iter()
                    .map(|e| &e.message)
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        }
        
        self.set(name, value)
    }

    /// Validate all signals (requires validation feature)
    #[cfg(feature = "validation")]
    pub fn validate_all(&self, validators: &std::collections::HashMap<String, Box<dyn Fn(&Value) -> crate::validation::ValidationResult>>) -> Vec<(String, crate::validation::ValidationResult)> {
        let mut results = Vec::new();
        
        #[cfg(not(feature = "optimized"))]
        {
            for entry in self.signals.iter() {
                let name = entry.key();
                let value = entry.value();
                
                if let Some(validator) = validators.get(name) {
                    let result = validator(value);
                    results.push((name.clone(), result));
                }
            }
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            for (name, value) in signals.iter() {
                if let Some(validator) = validators.get(name) {
                    let result = validator(value);
                    results.push((name.clone(), result));
                }
            }
        }
        
        results
    }

    /// Get comprehensive statistics
    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_statistics(&self) -> SignalStatistics {
        let mut stats = SignalStatistics {
            total_signals: self.len(),
            signals_by_type: std::collections::HashMap::new(),
            memory_usage_bytes: self.memory_usage(),
            total_updates: self.total_updates.load(std::sync::atomic::Ordering::Relaxed),
            
            #[cfg(feature = "quality-codes")]
            quality_distribution: std::collections::HashMap::new(),
            
            #[cfg(feature = "engineering-types")]
            engineering_signals: 0,
            
            #[cfg(feature = "engineering-types")]
            units_distribution: std::collections::HashMap::new(),
        };
        
        // Collect detailed statistics
        #[cfg(not(feature = "optimized"))]
        {
            for entry in self.signals.iter() {
                let value = entry.value();
                let type_name = value.type_name().to_string();
                *stats.signals_by_type.entry(type_name).or_insert(0) += 1;
                
                #[cfg(feature = "quality-codes")]
                if let Value::QualityValue { quality, .. } = value {
                    let quality_str = format!("{:?}", quality);
                    *stats.quality_distribution.entry(quality_str).or_insert(0) += 1;
                }
                
                #[cfg(feature = "engineering-types")]
                if let Value::Engineering { unit, .. } = value {
                    stats.engineering_signals += 1;
                    *stats.units_distribution.entry(unit.clone()).or_insert(0) += 1;
                }
            }
        }
        
        #[cfg(feature = "optimized")]
        {
            let signals = self.signals.read();
            for value in signals.values() {
                let type_name = value.type_name().to_string();
                *stats.signals_by_type.entry(type_name).or_insert(0) += 1;
                
                #[cfg(feature = "quality-codes")]
                if let Value::QualityValue { quality, .. } = value {
                    let quality_str = format!("{:?}", quality);
                    *stats.quality_distribution.entry(quality_str).or_insert(0) += 1;
                }
                
                #[cfg(feature = "engineering-types")]
                if let Value::Engineering { unit, .. } = value {
                    stats.engineering_signals += 1;
                    *stats.units_distribution.entry(unit.clone()).or_insert(0) += 1;
                }
            }
        }
        
        stats
    }
}

impl Default for SignalBus {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ENHANCED DATA STRUCTURES
// ============================================================================

/// Signal export structure for monitoring
#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone)]
pub struct SignalExport {
    pub name: String,
    pub value: Value,
    pub metadata: Option<SignalMetadata>,
}

/// Enhanced signal statistics
#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone)]
pub struct SignalStatistics {
    pub total_signals: usize,
    pub signals_by_type: std::collections::HashMap<String, usize>,
    pub memory_usage_bytes: u64,
    pub total_updates: u64,
    
    #[cfg(feature = "quality-codes")]
    pub quality_distribution: std::collections::HashMap<String, usize>,
    
    #[cfg(feature = "engineering-types")]
    pub engineering_signals: usize,
    
    #[cfg(feature = "engineering-types")]
    pub units_distribution: std::collections::HashMap<String, usize>,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_bus_basic() {
        let bus = SignalBus::new();
        
        // Test set and get
        bus.set("test", Value::Integer(42)).unwrap();
        assert_eq!(bus.get("test"), Some(Value::Integer(42)));
        
        // Test update
        bus.set("test", Value::Integer(100)).unwrap();
        assert_eq!(bus.get("test"), Some(Value::Integer(100)));
        
        // Test remove
        let removed = bus.remove("test");
        assert_eq!(removed, Some(Value::Integer(100)));
        assert_eq!(bus.get("test"), None);
    }

    #[test]
    fn test_signal_bus_clear() {
        let bus = SignalBus::new();
        
        bus.set("signal1", Value::Integer(1)).unwrap();
        bus.set("signal2", Value::Float(2.0)).unwrap();
        bus.set("signal3", Value::Bool(true)).unwrap();
        
        assert_eq!(bus.len(), 3);
        
        bus.clear();
        assert_eq!(bus.len(), 0);
        assert!(bus.is_empty());
    }

    #[test]
    fn test_signal_bus_keys() {
        let bus = SignalBus::new();
        
        bus.set("alpha", Value::Integer(1)).unwrap();
        bus.set("beta", Value::Integer(2)).unwrap();
        bus.set("gamma", Value::Integer(3)).unwrap();
        
        let mut keys = bus.keys();
        keys.sort();
        
        assert_eq!(keys, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_typed_getters() {
        let bus = SignalBus::new();
        
        // Test bool getter
        bus.set("bool_test", Value::Bool(true)).unwrap();
        assert_eq!(bus.get_bool("bool_test").unwrap(), true);
        
        // Test int getter
        bus.set("int_test", Value::Integer(42)).unwrap();
        assert_eq!(bus.get_int("int_test").unwrap(), 42);
        
        // Test float getter
        bus.set("float_test", Value::Float(3.14)).unwrap();
        assert_eq!(bus.get_float("float_test").unwrap(), 3.14);
        
        // Test type conversions
        bus.set("convert_test", Value::Integer(1)).unwrap();
        assert_eq!(bus.get_bool("convert_test").unwrap(), true);
        assert_eq!(bus.get_float("convert_test").unwrap(), 1.0);
        
        // Test missing signal
        assert!(bus.get_bool("missing").is_err());
    }

    #[cfg(feature = "quality-codes")]
    #[test]
    fn test_quality_codes() {
        let bus = SignalBus::new();
        
        // Test set with quality
        bus.set_with_quality("temp", Value::Float(25.5), crate::value::QualityCode::Good).unwrap();
        assert!(bus.is_quality_good("temp"));
        
        // Test bad quality
        bus.set_with_quality("pressure", Value::Float(0.0), crate::value::QualityCode::Bad).unwrap();
        assert!(!bus.is_quality_good("pressure"));
        
        // Test get quality
        assert_eq!(bus.get_quality("temp"), Some(crate::value::QualityCode::Good));
        assert_eq!(bus.get_quality("pressure"), Some(crate::value::QualityCode::Bad));
        
        // Test bad quality signals list
        let bad_signals = bus.get_bad_quality_signals();
        assert_eq!(bad_signals, vec!["pressure"]);
    }

    #[cfg(feature = "engineering-types")]
    #[test]
    fn test_engineering_values() {
        let bus = SignalBus::new();
        
        // Test set engineering value
        bus.set_engineering("temperature", 25.5, "°C".to_string()).unwrap();
        
        // Test get engineering value
        let eng = bus.get_engineering("temperature");
        assert_eq!(eng, Some((25.5, "°C".to_string())));
        
        // Test engineering signals list
        bus.set_engineering("pressure", 101.3, "kPa".to_string()).unwrap();
        let eng_signals = bus.get_engineering_signals();
        assert_eq!(eng_signals.len(), 2);
    }

    #[cfg(feature = "unit-conversion")]
    #[test]
    fn test_unit_conversion() {
        let bus = SignalBus::new();
        
        // Test conversion setup
        bus.set_engineering_with_conversion("length", 1000.0, "mm".to_string(), "m".to_string(), 0.001).unwrap();
        
        // Test same unit
        let result = bus.convert_units("length", "mm").unwrap();
        assert_eq!(result, Some(1000.0));
        
        // Test conversion to base unit
        let result = bus.convert_units("length", "m").unwrap();
        assert_eq!(result, Some(1.0));
    }

    #[cfg(all(feature = "enhanced", feature = "enhanced-monitoring"))]
    #[test]
    fn test_metadata() {
        let bus = SignalBus::new();
        
        bus.set("test", Value::Integer(42)).unwrap();
        
        // Test metadata exists
        let meta = bus.get_metadata("test");
        assert!(meta.is_some());
        
        // Test update count
        bus.set("test", Value::Integer(100)).unwrap();
        let count = bus.get_update_count("test");
        assert_eq!(count, Some(2));
        
        // Test last update time
        let last_update = bus.get_last_update("test");
        assert!(last_update.is_some());
    }

    #[cfg(feature = "validation")]
    #[test]
    fn test_validation() {
        let bus = SignalBus::new();
        
        // Test valid value
        let validator = |v: &Value| -> crate::validation::ValidationResult {
            match v {
                Value::Integer(n) if *n >= 0 && *n <= 100 => crate::validation::ValidationResult {
                    valid: true,
                    errors: vec![],
                    warnings: vec![],
                },
                _ => crate::validation::ValidationResult {
                    valid: false,
                    errors: vec![crate::validation::ValidationError {
                        path: "".to_string(),
                        message: "Value must be integer between 0 and 100".to_string(),
                        severity: crate::validation::ValidationSeverity::Error,
                    }],
                    warnings: vec![],
                },
            }
        };
        
        // Test valid value
        let result = bus.set_with_validation("percentage", Value::Integer(50), &validator);
        assert!(result.is_ok());
        
        // Test invalid value
        let result = bus.set_with_validation("percentage", Value::Integer(150), &validator);
        assert!(result.is_err());
    }

    #[cfg(feature = "enhanced-monitoring")]
    #[test]
    fn test_statistics() {
        let bus = SignalBus::new();
        
        bus.set("int1", Value::Integer(1)).unwrap();
        bus.set("float1", Value::Float(1.0)).unwrap();
        bus.set("bool1", Value::Bool(true)).unwrap();
        
        let stats = bus.get_statistics();
        assert_eq!(stats.total_signals, 3);
        assert_eq!(stats.signals_by_type.get("Integer"), Some(&1));
        assert_eq!(stats.signals_by_type.get("Float"), Some(&1));
        assert_eq!(stats.signals_by_type.get("Bool"), Some(&1));
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async_updates() {
        use tokio::time::{timeout, Duration};
        
        let bus = SignalBus::new();
        let mut rx = bus.subscribe();
        
        // Send update
        bus.set("async_test", Value::Integer(42)).unwrap();
        
        // Receive update
        let update = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
        assert_eq!(update.name, "async_test");
        assert_eq!(update.value, Value::Integer(42));
    }
}

// src/signal.rs
use crate::{error::*, value::Value};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{debug, trace};

#[cfg(feature = "metrics")]
use metrics::{counter, gauge};

#[cfg(feature = "optimized")]
use parking_lot::RwLock;

#[cfg(not(feature = "optimized"))]
use dashmap::DashMap;

#[cfg(feature = "enhanced")]
use {
    chrono::{DateTime, Utc},
    std::sync::atomic::{AtomicU64, Ordering},
};

#[cfg(all(feature = "enhanced", feature = "metrics"))]
use std::time::Instant;

// Signal metadata for enhanced mode
#[cfg(feature = "enhanced")]
#[derive(Debug, Clone)]
pub struct SignalMetadata {
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub update_count: u64,
    pub last_value: Option<Value>,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
}

#[cfg(feature = "enhanced")]
impl Default for SignalMetadata {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            last_updated: Utc::now(),
            update_count: 0,
            last_value: None,
            min_value: None,
            max_value: None,
        }
    }
}

// Main SignalBus structure with conditional fields
#[derive(Clone)]
pub struct SignalBus {
    #[cfg(not(feature = "optimized"))]
    signals: Arc<DashMap<String, Value>>,
    
    #[cfg(feature = "optimized")]
    signals: Arc<RwLock<HashMap<String, Value>>>,
    
    #[cfg(feature = "enhanced")]
    metadata: Arc<DashMap<String, SignalMetadata>>,
    
    #[cfg(feature = "enhanced")]
    total_updates: Arc<AtomicU64>,
    
    #[cfg(all(feature = "enhanced", feature = "history"))]
    history_buffer: Arc<RwLock<Vec<(String, Value, DateTime<Utc>)>>>,
    
    #[cfg(feature = "enhanced")]
    enable_tracking: bool,
}

impl SignalBus {
    pub fn new() -> Self {
        #[cfg(feature = "metrics")]
        {
            gauge!("petra_signal_bus_created").set(1.0);
        }
        
        Self {
            #[cfg(not(feature = "optimized"))]
            signals: Arc::new(DashMap::new()),
            
            #[cfg(feature = "optimized")]
            signals: Arc::new(RwLock::new(HashMap::new())),
            
            #[cfg(feature = "enhanced")]
            metadata: Arc::new(DashMap::new()),
            
            #[cfg(feature = "enhanced")]
            total_updates: Arc::new(AtomicU64::new(0)),
            
            #[cfg(all(feature = "enhanced", feature = "history"))]
            history_buffer: Arc::new(RwLock::new(Vec::with_capacity(1000))),
            
            #[cfg(feature = "enhanced")]
            enable_tracking: true,
        }
    }
    
    #[cfg(feature = "enhanced")]
    pub fn with_tracking(enable: bool) -> Self {
        let mut bus = Self::new();
        bus.enable_tracking = enable;
        bus
    }
    
    // Core set method with feature-specific implementations
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        trace!("Setting signal '{}' to {:?}", name, value);
        
        #[cfg(all(feature = "enhanced", feature = "metrics"))]
        let start = Instant::now();
        
        // Update the value
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.insert(name.to_string(), value.clone());
        }
        
        #[cfg(feature = "optimized")]
        {
            self.signals.write().insert(name.to_string(), value.clone());
        }
        
        // Enhanced tracking
        #[cfg(feature = "enhanced")]
        if self.enable_tracking {
            self.update_metadata(name, &value);
            self.total_updates.fetch_add(1, Ordering::Relaxed);
            
            #[cfg(feature = "history")]
            if let Ok(mut history) = self.history_buffer.try_write() {
                history.push((name.to_string(), value.clone(), Utc::now()));
                if history.len() > 10000 {
                    history.drain(0..5000);
                }
            }
        }
        
        // Metrics
        #[cfg(feature = "metrics")]
        {
            counter!("petra_signal_updates_total", "signal" => name.to_string()).increment(1);
            
            #[cfg(feature = "enhanced")]
            if self.enable_tracking {
                histogram!("petra_signal_update_duration_us")
                    .record(start.elapsed().as_micros() as f64);
            }
        }
        
        debug!("Signal '{}' set to {:?}", name, value);
        Ok(())
    }
    
    // Core get method
    pub fn get(&self, name: &str) -> Option<Value> {
        trace!("Getting signal '{}'", name);
        
        #[cfg(not(feature = "optimized"))]
        let result = self.signals.get(name).map(|v| v.clone());
        
        #[cfg(feature = "optimized")]
        let result = self.signals.read().get(name).cloned();
        
        #[cfg(feature = "metrics")]
        {
            counter!("petra_signal_reads_total", "signal" => name.to_string()).increment(1);
            if result.is_none() {
                counter!("petra_signal_misses_total", "signal" => name.to_string()).increment(1);
            }
        }
        
        result
    }
    
    // Convenience methods
    pub fn get_bool(&self, name: &str) -> Result<bool> {
        match self.get(name) {
            Some(Value::Bool(b)) => Ok(b),
            Some(v) => Err(PlcError::TypeMismatch(format!(
                "Expected bool for '{}', got {:?}", name, v
            ))),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    pub fn get_int(&self, name: &str) -> Result<i32> {
        match self.get(name) {
            Some(Value::Int(i)) => Ok(i),
            Some(v) => Err(PlcError::TypeMismatch(format!(
                "Expected int for '{}', got {:?}", name, v
            ))),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    pub fn get_float(&self, name: &str) -> Result<f64> {
        match self.get(name) {
            Some(Value::Float(f)) => Ok(f),
            Some(Value::Int(i)) => Ok(i as f64), // Allow int to float conversion
            Some(v) => Err(PlcError::TypeMismatch(format!(
                "Expected float for '{}', got {:?}", name, v
            ))),
            None => Err(PlcError::SignalNotFound(name.to_string())),
        }
    }
    
    // Snapshot for atomic operations
    pub fn snapshot(&self) -> HashMap<String, Value> {
        #[cfg(not(feature = "optimized"))]
        {
            self.signals.iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect()
        }
        
        #[cfg(feature = "optimized")]
        {
            self.signals.read().clone()
        }
    }
    
    pub fn len(&self) -> usize {
        #[cfg(not(feature = "optimized"))]
        { self.signals.len() }
        
        #[cfg(feature = "optimized")]
        { self.signals.read().len() }
    }
    
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    pub fn clear(&self) {
        #[cfg(not(feature = "optimized"))]
        { self.signals.clear(); }
        
        #[cfg(feature = "optimized")]
        { self.signals.write().clear(); }
        
        #[cfg(feature = "enhanced")]
        {
            self.metadata.clear();
            self.total_updates.store(0, Ordering::Relaxed);
            
            #[cfg(feature = "history")]
            if let Ok(mut history) = self.history_buffer.try_write() {
                history.clear();
            }
        }
    }
    
    // Enhanced mode methods
    #[cfg(feature = "enhanced")]
    fn update_metadata(&self, name: &str, value: &Value) {
        use dashmap::mapref::entry::Entry;
        
        match self.metadata.entry(name.to_string()) {
            Entry::Occupied(mut entry) => {
                let meta = entry.get_mut();
                meta.last_updated = Utc::now();
                meta.update_count += 1;
                meta.last_value = Some(value.clone());
                
                // Update min/max for numeric values
                match (value, &meta.min_value, &meta.max_value) {
                    (Value::Int(v), Some(Value::Int(min)), Some(Value::Int(max))) => {
                        if v < min {
                            meta.min_value = Some(value.clone());
                        }
                        if v > max {
                            meta.max_value = Some(value.clone());
                        }
                    }
                    (Value::Float(v), Some(Value::Float(min)), Some(Value::Float(max))) => {
                        if v < min {
                            meta.min_value = Some(value.clone());
                        }
                        if v > max {
                            meta.max_value = Some(value.clone());
                        }
                    }
                    (Value::Int(_) | Value::Float(_), None, None) => {
                        meta.min_value = Some(value.clone());
                        meta.max_value = Some(value.clone());
                    }
                    _ => {}
                }
            }
            Entry::Vacant(entry) => {
                let mut meta = SignalMetadata::default();
                meta.last_value = Some(value.clone());
                if matches!(value, Value::Int(_) | Value::Float(_)) {
                    meta.min_value = Some(value.clone());
                    meta.max_value = Some(value.clone());
                }
                entry.insert(meta);
            }
        }
    }
    
    #[cfg(feature = "enhanced")]
    pub fn get_metadata(&self, name: &str) -> Option<SignalMetadata> {
        self.metadata.get(name).map(|m| m.clone())
    }
    
    #[cfg(feature = "enhanced")]
    pub fn get_total_updates(&self) -> u64 {
        self.total_updates.load(Ordering::Relaxed)
    }
    
    #[cfg(feature = "enhanced")]
    pub fn get_statistics(&self) -> SignalBusStats {
        let signal_count = self.len();
        let total_updates = self.get_total_updates();
        
        let metadata_stats: Vec<_> = self.metadata.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        
        SignalBusStats {
            signal_count,
            total_updates,
            metadata: metadata_stats,
        }
    }
    
    #[cfg(all(feature = "enhanced", feature = "history"))]
    pub fn get_recent_history(&self, limit: usize) -> Vec<(String, Value, DateTime<Utc>)> {
        if let Ok(history) = self.history_buffer.read() {
            let start = history.len().saturating_sub(limit);
            history[start..].to_vec()
        } else {
            Vec::new()
        }
    }
    
    // Optimized batch operations
    #[cfg(feature = "optimized")]
    pub fn batch_update<I>(&self, updates: I) -> Result<()>
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        let mut signals = self.signals.write();
        
        #[cfg(all(feature = "enhanced", feature = "metrics"))]
        let start = Instant::now();
        
        let mut _count = 0;
        for (name, value) in updates {
            signals.insert(name.clone(), value.clone());
            
            #[cfg(feature = "enhanced")]
            if self.enable_tracking {
                drop(signals); // Release lock temporarily
                self.update_metadata(&name, &value);
                signals = self.signals.write();
            }
            
            count += 1;
        }
        
        #[cfg(feature = "enhanced")]
        if self.enable_tracking {
            self.total_updates.fetch_add(count, Ordering::Relaxed);
        }
        
        #[cfg(all(feature = "enhanced", feature = "metrics"))]
        {
            histogram!("petra_batch_update_duration_us")
                .record(start.elapsed().as_micros() as f64);
            counter!("petra_batch_updates_total").increment(count);
        }
        
        Ok(())
    }
    
    #[cfg(feature = "optimized")]
    pub fn batch_read<I, C>(&self, names: I) -> HashMap<String, Value>
    where
        I: IntoIterator<Item = String>,
        C: FromIterator<(String, Value)>,
    {
        let signals = self.signals.read();
        names.into_iter()
            .filter_map(|name| {
                signals.get(&name).map(|v| (name, v.clone()))
            })
            .collect()
    }
}

#[cfg(feature = "enhanced")]
#[derive(Debug, Clone)]
pub struct SignalBusStats {
    pub signal_count: usize,
    pub total_updates: u64,
    pub metadata: Vec<(String, SignalMetadata)>,
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
    fn test_basic_operations() {
        let bus = SignalBus::new();
        
        // Test set and get
        bus.set("test_bool", Value::Bool(true)).unwrap();
        assert_eq!(bus.get("test_bool"), Some(Value::Bool(true)));
        
        // Test typed getters
        assert_eq!(bus.get_bool("test_bool").unwrap(), true);
        
        // Test missing signal
        assert!(bus.get("missing").is_none());
        assert!(bus.get_bool("missing").is_err());
    }
    
    #[test]
    fn test_type_conversions() {
        let bus = SignalBus::new();
        
        // Int to float conversion
        bus.set("test_int", Value::Int(42)).unwrap();
        assert_eq!(bus.get_float("test_int").unwrap(), 42.0);
        
        // Type mismatch
        bus.set("test_string", Value::Bool(false)).unwrap();
        assert!(bus.get_int("test_string").is_err());
    }
    
    #[test]
    fn test_snapshot() {
        let bus = SignalBus::new();
        bus.set("sig1", Value::Int(1)).unwrap();
        bus.set("sig2", Value::Bool(true)).unwrap();
        
        let snapshot = bus.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert_eq!(snapshot.get("sig1"), Some(&Value::Int(1)));
        assert_eq!(snapshot.get("sig2"), Some(&Value::Bool(true)));
    }
    
    #[cfg(feature = "enhanced")]
    #[test]
    fn test_metadata_tracking() {
        let bus = SignalBus::new();
        
        // Set initial value
        bus.set("tracked", Value::Int(10)).unwrap();
        
        // Update multiple times
        bus.set("tracked", Value::Int(5)).unwrap();
        bus.set("tracked", Value::Int(15)).unwrap();
        
        let meta = bus.get_metadata("tracked").unwrap();
        assert_eq!(meta.update_count, 3);
        assert_eq!(meta.min_value, Some(Value::Int(5)));
        assert_eq!(meta.max_value, Some(Value::Int(15)));
        assert_eq!(meta.last_value, Some(Value::Int(15)));
    }
    
    #[cfg(feature = "optimized")]
    #[test]
    fn test_batch_operations() {
        let bus = SignalBus::new();
        
        // Batch update
        let updates = vec![
            ("sig1".to_string(), Value::Int(1)),
            ("sig2".to_string(), Value::Bool(true)),
            ("sig3".to_string(), Value::Float(3.14)),
        ];
        bus.batch_update(updates).unwrap();
        
        // Verify all were set
        assert_eq!(bus.get("sig1"), Some(Value::Int(1)));
        assert_eq!(bus.get("sig2"), Some(Value::Bool(true)));
        assert_eq!(bus.get("sig3"), Some(Value::Float(3.14)));
        
        // Batch read
        let names = vec!["sig1".to_string(), "sig3".to_string(), "missing".to_string()];
        let results: HashMap<String, Value> = bus.batch_read(names);
        assert_eq!(results.len(), 2);
        assert_eq!(results.get("sig1"), Some(&Value::Int(1)));
        assert_eq!(results.get("sig3"), Some(&Value::Float(3.14)));
        assert!(!results.contains_key("missing"));
    }
    
    #[cfg(all(feature = "enhanced", feature = "history"))]
    #[test]
    fn test_history_buffer() {
        let bus = SignalBus::new();
        
        // Add some history
        for i in 0..5 {
            bus.set("counter", Value::Int(i)).unwrap();
        }
        
        let history = bus.get_recent_history(3);
        assert_eq!(history.len(), 3);
        
        // Check the values are the most recent ones
        let values: Vec<i32> = history.iter()
            .filter_map(|(name, value, _)| {
                if name == "counter" {
                    match value {
                        Value::Int(i) => Some(*i),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(values, vec![2, 3, 4]);
    }
}

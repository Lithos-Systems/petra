use crate::{error::*, value::Value};
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;
use tracing::{trace, info}; // Added info import
use metrics::{counter, gauge, histogram};

#[derive(Clone)]
pub struct OptimizedSignalBus {
    signals: Arc<RwLock<HashMap<String, Value>>>,
    // Cache for frequently accessed signals
    hot_cache: Arc<dashmap::DashMap<String, Value>>,
    access_counts: Arc<dashmap::DashMap<String, u64>>,
}

impl OptimizedSignalBus {
    pub fn new() -> Self {
        Self {
            signals: Arc::new(RwLock::new(HashMap::new())),
            hot_cache: Arc::new(dashmap::DashMap::new()),
            access_counts: Arc::new(dashmap::DashMap::new()),
        }
    }

    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        trace!("Set {} = {}", name, value);
        
        // Update metrics
        counter!("petra_signal_updates_total", "signal" => name.to_string()).increment(1);
        
        // Update main storage
        self.signals.write().insert(name.to_string(), value.clone());
        
        // Update hot cache if frequently accessed
        if let Some(count) = self.access_counts.get(name) {
            if *count > 100 {
                self.hot_cache.insert(name.to_string(), value.clone());
            }
        }
        
        // Update gauge metric - Fixed bool cast
        match &value {
            Value::Bool(b) => gauge!("petra_signal_value", "signal" => name.to_string()).set(if *b { 1.0 } else { 0.0 }),
            Value::Int(i) => gauge!("petra_signal_value", "signal" => name.to_string()).set(*i as f64),
            Value::Float(f) => gauge!("petra_signal_value", "signal" => name.to_string()).set(*f),
        }
        
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        // Track access patterns
        self.access_counts
            .entry(name.to_string())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        
        // Check hot cache first
        if let Some(value) = self.hot_cache.get(name) {
            counter!("petra_cache_hits_total").increment(1);
            return Ok(value.clone());
        }
        
        counter!("petra_cache_misses_total").increment(1);
        
        // Rest of the implementation...
        self.signals
            .read()
            .get(name)
            .cloned()
            .ok_or_else(|| PlcError::SignalNotFound(name.to_string()))
    }
    
    pub fn optimize_cache(&self) {
        info!("Optimizing signal cache based on access patterns");
        
        // Promote frequently accessed signals to hot cache
        for entry in self.access_counts.iter() {
            let (name, count) = entry.pair();
            if *count > 100 && !self.hot_cache.contains_key(name) {
                if let Some(value) = self.signals.read().get(name) {
                    self.hot_cache.insert(name.clone(), value.clone());
                }
            }
        }
    }
}

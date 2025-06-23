use crate::{error::*, value::Value};
use parking_lot::RwLock;
use std::sync::Arc;
use std::collections::HashMap;
use std::fmt;
use tracing::trace;
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
        
        // Update gauge metric
        match &value {
            Value::Bool(b) => gauge!("petra_signal_value", "signal" => name.to_string()).set(*b as f64),
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
        
        self.signals
            .read()
            .get(name)
            .cloned()
            .ok_or_else(|| PlcError::SignalNotFound(name.to_string()))
    }

    pub fn batch_get(&self, names: &[&str]) -> Result<Vec<(String, Value)>> {
        let signals = self.signals.read();
        names.iter()
            .map(|&name| {
                signals.get(name)
                    .map(|v| (name.to_string(), v.clone()))
                    .ok_or_else(|| PlcError::SignalNotFound(name.to_string()))
            })
            .collect()
    }

    pub fn batch_set(&self, updates: Vec<(&str, Value)>) -> Result<()> {
        let mut signals = self.signals.write();
        for (name, value) in updates {
            signals.insert(name.to_string(), value.clone());
            
            // Update hot cache if needed
            if let Some(count) = self.access_counts.get(name) {
                if *count > 100 {
                    self.hot_cache.insert(name.to_string(), value);
                }
            }
        }
        Ok(())
    }

    pub fn optimize_cache(&self) {
        // Move top 10% most accessed signals to hot cache
        let mut access_vec: Vec<_> = self.access_counts
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        
        access_vec.sort_by(|a, b| b.1.cmp(&a.1));
        
        let cache_size = (access_vec.len() / 10).max(10);
        self.hot_cache.clear();
        
        let signals = self.signals.read();
        for (name, _) in access_vec.iter().take(cache_size) {
            if let Some(value) = signals.get(name) {
                self.hot_cache.insert(name.clone(), value.clone());
            }
        }
        
        info!("Optimized cache with {} hot signals", self.hot_cache.len());
    }
}

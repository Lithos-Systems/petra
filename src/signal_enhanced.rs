use crate::{error::*, value::Value};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use crossbeam::atomic::AtomicCell;
use std::collections::HashMap;
use tracing::{trace, warn};

pub struct EnhancedSignalBus {
    signals: Arc<DashMap<String, SignalEntry>>,
    hot_cache: Arc<DashMap<String, Arc<AtomicCell<Value>>>>,
    config: SignalBusConfig,
    stats: Arc<SignalStats>,
}

#[derive(Debug, Clone)]
pub struct SignalEntry {
    pub value: Value,
    pub timestamp: Instant,
    pub update_count: u64,
}

#[derive(Debug, Clone)]
pub struct SignalBusConfig {
    pub max_signals: usize,
    pub signal_ttl: Duration,
    pub cleanup_interval: Duration,
    pub hot_signal_threshold: u64,
}

impl Default for SignalBusConfig {
    fn default() -> Self {
        Self {
            max_signals: 10_000,
            signal_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(60),
            hot_signal_threshold: 100, // Updates per cleanup interval
        }
    }
}

#[derive(Debug, Default)]
pub struct SignalStats {
    pub total_updates: AtomicCell<u64>,
    pub cache_hits: AtomicCell<u64>,
    pub cache_misses: AtomicCell<u64>,
    pub evictions: AtomicCell<u64>,
}

impl EnhancedSignalBus {
    pub fn new(config: SignalBusConfig) -> Self {
        let bus = Self {
            signals: Arc::new(DashMap::new()),
            hot_cache: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(SignalStats::default()),
        };
        
        // Start cleanup task
        let bus_clone = bus.clone();
        tokio::spawn(async move {
            bus_clone.cleanup_task().await;
        });
        
        bus
    }
    
    pub fn set(&self, name: &str, value: Value) -> Result<()> {
        trace!("Set {} = {}", name, value);
        
        // Update hot cache if exists
        if let Some(cell) = self.hot_cache.get(name) {
            cell.store(value.clone());
            self.stats.cache_hits.fetch_add(1);
        } else {
            self.stats.cache_misses.fetch_add(1);
        }
        
        // Update main storage
        let mut entry = self.signals.entry(name.to_string()).or_insert(SignalEntry {
            value: value.clone(),
            timestamp: Instant::now(),
            update_count: 0,
        });
        
        entry.value = value.clone();
        entry.timestamp = Instant::now();
        entry.update_count += 1;
        
        self.stats.total_updates.fetch_add(1);
        
        // Check if should be promoted to hot cache
        if entry.update_count > self.config.hot_signal_threshold {
            self.promote_to_hot_cache(name, value.clone());
        }
        
        Ok(())
    }
    
    pub fn get(&self, name: &str) -> Result<Value> {
        // Check hot cache first
        if let Some(cell) = self.hot_cache.get(name) {
            self.stats.cache_hits.fetch_add(1);
            return Ok(cell.value().load());
        }
        
        self.stats.cache_misses.fetch_add(1);
        
        self.signals
            .get(name)
            .map(|entry| entry.value.clone())
            .ok_or_else(|| PlcError::SignalNotFound(name.to_string()))
    }
    
    fn promote_to_hot_cache(&self, name: &str, value: Value) {
        if self.hot_cache.len() < 1000 { // Limit hot cache size
            self.hot_cache.insert(
                name.to_string(),
                Arc::new(AtomicCell::new(value))
            );
            trace!("Promoted {} to hot cache", name);
        }
    }
    
    async fn cleanup_task(&self) {
        let mut interval = tokio::time::interval(self.config.cleanup_interval);
        
        loop {
            interval.tick().await;
            self.cleanup_stale_signals();
            self.cleanup_cold_cache();
        }
    }
    
    fn cleanup_stale_signals(&self) {
        let now = Instant::now();
        let mut evicted = 0;
        
        self.signals.retain(|_, entry| {
            let should_keep = now.duration_since(entry.timestamp) < self.config.signal_ttl;
            if !should_keep {
                evicted += 1;
            }
            should_keep
        });
        
        if evicted > 0 {
            self.stats.evictions.fetch_add(evicted);
            warn!("Evicted {} stale signals", evicted);
        }
        
        // Enforce max signals limit
        if self.signals.len() > self.config.max_signals {
            let to_remove = self.signals.len() - self.config.max_signals;
            let mut entries: Vec<_> = self.signals.iter()
                .map(|e| (e.key().clone(), e.timestamp))
                .collect();
            
            entries.sort_by_key(|(_, ts)| *ts);
            
            for (key, _) in entries.into_iter().take(to_remove) {
                self.signals.remove(&key);
                self.hot_cache.remove(&key);
                evicted += 1;
            }
            
            self.stats.evictions.fetch_add(evicted as u64);
        }
    }
    
    fn cleanup_cold_cache(&self) {
        // Reset update counts and demote cold signals
        for mut entry in self.signals.iter_mut() {
            if entry.update_count < self.config.hot_signal_threshold / 2 {
                self.hot_cache.remove(entry.key());
            }
            entry.update_count = 0;
        }
    }
    
    pub fn stats(&self) -> SignalBusStats {
        SignalBusStats {
            total_signals: self.signals.len(),
            hot_cache_size: self.hot_cache.len(),
            total_updates: self.stats.total_updates.load(),
            cache_hits: self.stats.cache_hits.load(),
            cache_misses: self.stats.cache_misses.load(),
            evictions: self.stats.evictions.load(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SignalBusStats {
    pub total_signals: usize,
    pub hot_cache_size: usize,
    pub total_updates: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
}

impl Clone for EnhancedSignalBus {
    fn clone(&self) -> Self {
        Self {
            signals: self.signals.clone(),
            hot_cache: self.hot_cache.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

// src/engine.rs - Fixed engine implementation
use crate::{
    blocks::{Block, create_block},
    config::Config,
    error::{PlcError, Result},
    signal::SignalBus,
    value::Value,
};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

#[cfg(feature = "metrics")]
use metrics::{gauge, counter, histogram};

/// Main PETRA engine that orchestrates block execution
pub struct Engine {
    /// Engine configuration
    config: Config,
    /// Signal bus for data exchange
    signal_bus: SignalBus,
    /// Loaded blocks
    blocks: Vec<Box<dyn Block>>,
    /// Running state
    running: Arc<AtomicBool>,
    /// Statistics
    stats: Arc<RwLock<EngineStats>>,
}

/// Engine statistics
#[derive(Debug, Default)]
pub struct EngineStats {
    pub scan_count: u64,
    pub total_scan_time: Duration,
    pub min_scan_time: Duration,
    pub max_scan_time: Duration,
    pub last_scan_time: Duration,
    pub block_errors: u64,
}

impl Engine {
    /// Create a new engine from configuration
    pub fn new(config: Config) -> Result<Self> {
        let signal_bus = SignalBus::new();
        
        // Initialize signals
        for signal in &config.signals {
            if let Some(initial) = &signal.initial {
                let value = match signal.signal_type.as_str() {
                    "bool" => match initial {
                        serde_yaml::Value::Bool(b) => Value::Bool(*b),
                        _ => return Err(PlcError::Config(format!(
                            "Signal '{}' initial value type mismatch", signal.name
                        ))),
                    },
                    "int" => match initial {
                        serde_yaml::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Value::Int(i)
                            } else {
                                return Err(PlcError::Config(format!(
                                    "Signal '{}' initial value not a valid integer", signal.name
                                )));
                            }
                        },
                        _ => return Err(PlcError::Config(format!(
                            "Signal '{}' initial value type mismatch", signal.name
                        ))),
                    },
                    "float" => match initial {
                        serde_yaml::Value::Number(n) => {
                            if let Some(f) = n.as_f64() {
                                Value::Float(f)
                            } else {
                                return Err(PlcError::Config(format!(
                                    "Signal '{}' initial value not a valid float", signal.name
                                )));
                            }
                        },
                        _ => return Err(PlcError::Config(format!(
                            "Signal '{}' initial value type mismatch", signal.name
                        ))),
                    },
                    _ => return Err(PlcError::Config(format!(
                        "Unknown signal type: {}", signal.signal_type
                    ))),
                };
                
                signal_bus.set(&signal.name, value)?;
            }
        }
        
        // Create blocks
        let mut blocks = Vec::new();
        for block_config in &config.blocks {
            let block = create_block(block_config)?;
            blocks.push(block);
        }
        
        Ok(Self {
            config,
            signal_bus,
            blocks,
            running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RwLock::new(EngineStats::default())),
        })
    }
    
    /// Get reference to the signal bus
    pub fn signal_bus(&self) -> &SignalBus {
        &self.signal_bus
    }
    
    /// Get engine statistics
    pub async fn stats(&self) -> EngineStats {
        self.stats.read().await.clone()
    }
    
    /// Run the engine
    pub async fn run(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(PlcError::Runtime("Engine already running".to_string()));
        }
        
        self.running.store(true, Ordering::Relaxed);
        info!("Starting PETRA engine with scan time: {}ms", self.config.scan_time_ms);
        
        // Initialize metrics if enabled
        #[cfg(feature = "metrics")]
        {
            gauge!("petra_engine_scan_time_ms").set(self.config.scan_time_ms as f64);
            counter!("petra_engine_starts").increment(1);
        }
        
        let target_scan_time = Duration::from_millis(self.config.scan_time_ms);
        
        while self.running.load(Ordering::Relaxed) {
            let scan_start = Instant::now();
            
            // Execute all blocks
            let mut block_errors = 0;
            for block in &self.blocks {
                if let Err(e) = block.execute(&self.signal_bus) {
                    error!("Block '{}' execution failed: {}", block.name(), e);
                    block_errors += 1;
                    
                    #[cfg(feature = "metrics")]
                    counter!("petra_block_errors", "block" => block.name()).increment(1);
                }
            }
            
            let scan_duration = scan_start.elapsed();
            
            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.scan_count += 1;
                stats.total_scan_time += scan_duration;
                stats.last_scan_time = scan_duration;
                stats.block_errors += block_errors;
                
                if stats.scan_count == 1 || scan_duration < stats.min_scan_time {
                    stats.min_scan_time = scan_duration;
                }
                if scan_duration > stats.max_scan_time {
                    stats.max_scan_time = scan_duration;
                }
            }
            
            // Update metrics
            #[cfg(feature = "metrics")]
            {
                histogram!("petra_scan_duration_us").record(scan_duration.as_micros() as f64);
                gauge!("petra_last_scan_duration_ms").set(scan_duration.as_millis() as f64);
                counter!("petra_scan_count").increment(1);
            }
            
            // Check for scan time overrun
            if scan_duration > target_scan_time {
                warn!(
                    "Scan time overrun: {:?} > {:?}",
                    scan_duration, target_scan_time
                );
                
                #[cfg(feature = "metrics")]
                counter!("petra_scan_overruns").increment(1);
            }
            
            // Sleep for remaining time
            if let Some(sleep_time) = target_scan_time.checked_sub(scan_duration) {
                tokio::time::sleep(sleep_time).await;
            }
        }
        
        info!("PETRA engine stopped");
        Ok(())
    }
    
    /// Stop the engine
    pub fn stop(&self) {
        info!("Stopping PETRA engine");
        self.running.store(false, Ordering::Relaxed);
    }
    
    /// Check if engine is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Clone for EngineStats {
    fn clone(&self) -> Self {
        Self {
            scan_count: self.scan_count,
            total_scan_time: self.total_scan_time,
            min_scan_time: self.min_scan_time,
            max_scan_time: self.max_scan_time,
            last_scan_time: self.last_scan_time,
            block_errors: self.block_errors,
        }
    }
}

// Additional helper functions and test utilities can go here...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SignalConfig, BlockConfig};
    use std::collections::HashMap;
    
    fn create_test_config() -> Config {
        Config {
            scan_time_ms: 100,
            signals: vec![
                SignalConfig {
                    name: "test_signal".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: None,
                    tags: vec![],
                    #[cfg(feature = "engineering-types")]
                    units: None,
                    #[cfg(feature = "quality-codes")]
                    quality_enabled: false,
                    #[cfg(feature = "validation")]
                    validation: None,
                    metadata: HashMap::new(),
                },
            ],
            blocks: vec![],
            mqtt: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "web")]
            web: None,
            protocols: None,
        }
    }
    
    #[test]
    fn test_engine_creation() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        assert!(!engine.is_running());
    }
    
    #[tokio::test]
    async fn test_engine_start_stop() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Start engine in background
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            engine_clone.run().await
        });
        
        // Wait a bit
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Check it's running
        assert!(engine.is_running());
        
        // Stop it
        engine.stop();
        
        // Wait for task to complete
        let _ = handle.await;
        
        assert!(!engine.is_running());
    }
}

// Make Engine cloneable for testing
impl Clone for Engine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            signal_bus: self.signal_bus.clone(),
            blocks: vec![], // Don't clone blocks for testing
            running: self.running.clone(),
            stats: self.stats.clone(),
        }
    }
}

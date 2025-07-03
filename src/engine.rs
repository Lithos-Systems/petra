// src/engine.rs - Fixed engine implementation with Arc<Mutex<>> for async safety
use crate::{
    blocks::{Block, create_block},
    config::{Config, EngineConfig},
    error::{PlcError, Result},
    signal::SignalBus,
    value::Value,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, RwLock, mpsc},
    time::{interval, MissedTickBehavior},
};
use tracing::{info, warn, error, debug, trace};

#[cfg(feature = "metrics")]
use metrics::{counter, gauge, histogram};

#[cfg(feature = "enhanced-monitoring")]
use ringbuffer::{AllocRingBuffer, RingBuffer};

#[cfg(feature = "circuit-breaker")]
use crate::blocks::circuit_breaker::BlockExecutor;

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub enhanced_monitoring: bool,
    pub metrics_enabled: bool,
    pub max_consecutive_errors: u64,
    pub error_recovery_enabled: bool,
    pub performance_logging: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
            metrics_enabled: cfg!(feature = "metrics"),
            max_consecutive_errors: 10,
            error_recovery_enabled: true,
            performance_logging: false,
        }
    }
}

/// Engine statistics
#[derive(Debug, Default, Clone)]
pub struct EngineStats {
    pub scan_count: u64,
    pub total_scan_time: Duration,
    pub min_scan_time: Duration,
    pub max_scan_time: Duration,
    pub last_scan_time: Duration,
    pub block_errors: u64,
    pub avg_scan_time: Duration,
    pub jitter: Duration,
}

/// Main PETRA execution engine
pub struct Engine {
    // Core components
    bus: SignalBus,
    blocks: Arc<Mutex<Vec<Box<dyn Block>>>>, // Changed to Arc<Mutex<>> for async safety
    config: Config,
    engine_config: EngineConfig,
    
    // Runtime state
    running: Arc<AtomicBool>,
    scan_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    consecutive_errors: Arc<AtomicU64>,
    start_time: Instant,
    target_scan_time: Duration,
    
    // Statistics
    stats: Arc<RwLock<EngineStats>>,
    
    // Enhanced monitoring data
    #[cfg(feature = "enhanced-monitoring")]
    scan_times: Arc<RwLock<VecDeque<Duration>>>,
    #[cfg(feature = "enhanced-monitoring")]
    execution_order: Arc<RwLock<Vec<String>>>,
    #[cfg(feature = "enhanced-monitoring")]
    failed_blocks: Arc<RwLock<Vec<String>>>,
    #[cfg(feature = "enhanced-monitoring")]
    block_execution_times: Arc<RwLock<HashMap<String, Duration>>>,
    
    // Circuit breaker integration
    #[cfg(feature = "circuit-breaker")]
    block_executor: Option<BlockExecutor>,
}

impl Engine {
    /// Create a new engine instance
    pub fn new(config: Config) -> Result<Self> {
        Self::new_with_engine_config(config, EngineConfig::default())
    }
    
    /// Create engine with custom engine configuration
    pub fn new_with_engine_config(config: Config, engine_config: EngineConfig) -> Result<Self> {
        info!("Initializing PETRA engine v{}", env!("CARGO_PKG_VERSION"));
        
        // Initialize signal bus
        let bus = SignalBus::new();
        
        // Initialize signals
        for signal_config in &config.signals {
            let value = match signal_config.signal_type.as_str() {
                "bool" => {
                    if let Some(initial) = &signal_config.initial {
                        match initial {
                            serde_yaml::Value::Bool(b) => Value::Bool(*b),
                            _ => return Err(PlcError::Config(format!(
                                "Signal '{}' initial value type mismatch", signal_config.name
                            ))),
                        }
                    } else {
                        Value::Bool(false)
                    }
                },
                "int" => {
                    if let Some(initial) = &signal_config.initial {
                        match initial {
                            serde_yaml::Value::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    Value::Int(i)
                                } else {
                                    return Err(PlcError::Config(format!(
                                        "Signal '{}' initial value not a valid integer", signal_config.name
                                    )));
                                }
                            },
                            _ => return Err(PlcError::Config(format!(
                                "Signal '{}' initial value type mismatch", signal_config.name
                            ))),
                        }
                    } else {
                        Value::Int(0)
                    }
                },
                "float" => {
                    if let Some(initial) = &signal_config.initial {
                        match initial {
                            serde_yaml::Value::Number(n) => {
                                if let Some(f) = n.as_f64() {
                                    Value::Float(f)
                                } else {
                                    return Err(PlcError::Config(format!(
                                        "Signal '{}' initial value not a valid float", signal_config.name
                                    )));
                                }
                            },
                            _ => return Err(PlcError::Config(format!(
                                "Signal '{}' initial value type mismatch", signal_config.name
                            ))),
                        }
                    } else {
                        Value::Float(0.0)
                    }
                },
                _ => {
                    return Err(PlcError::Config(format!(
                        "Unknown signal type: {}", signal_config.signal_type
                    )));
                }
            };
            bus.set(&signal_config.name, value)?;
        }
        
        // Create blocks
        let mut blocks = Vec::new();
        for block_config in &config.blocks {
            match create_block(block_config) {
                Ok(block) => {
                    debug!("Created block '{}' of type '{}'", 
                        block_config.name, block_config.block_type);
                    blocks.push(block);
                }
                Err(e) => {
                    error!("Failed to create block '{}': {}", block_config.name, e);
                    return Err(PlcError::Config(format!(
                        "Failed to create block '{}': {}", block_config.name, e
                    )));
                }
            }
        }
        
        let target_scan_time = Duration::from_millis(config.engine.scan_time_ms);
        let start_time = Instant::now();
        
        // Initialize stats
        let stats = EngineStats {
            min_scan_time: Duration::MAX,
            max_scan_time: Duration::ZERO,
            ..Default::default()
        };
        
        Ok(Self {
            bus,
            blocks: Arc::new(Mutex::new(blocks)), // Wrap in Arc<Mutex<>>
            config,
            engine_config,
            running: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            consecutive_errors: Arc::new(AtomicU64::new(0)),
            start_time,
            target_scan_time,
            stats: Arc::new(RwLock::new(stats)),
            
            #[cfg(feature = "enhanced-monitoring")]
            scan_times: Arc::new(RwLock::new(VecDeque::with_capacity(1000))),
            #[cfg(feature = "enhanced-monitoring")]
            execution_order: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "enhanced-monitoring")]
            failed_blocks: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "enhanced-monitoring")]
            block_execution_times: Arc::new(RwLock::new(HashMap::new())),
            
            #[cfg(feature = "circuit-breaker")]
            block_executor: None,
        })
    }

    /// Start the engine main loop
    pub async fn run(&self) -> Result<()> {
        self.running.store(true, Ordering::Relaxed);
        info!("Starting PETRA engine with scan time: {}ms", self.config.engine.scan_time_ms);
        
        let mut interval = interval(self.target_scan_time);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        // Initialize metrics if enabled
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_engine_scan_time_ms").set(self.config.engine.scan_time_ms as f64);
            counter!("petra_engine_starts").increment(1);
        }
        
        while self.running.load(Ordering::Relaxed) {
            interval.tick().await;
            let scan_start = Instant::now();
            
            match self.execute_scan_cycle().await {
                Ok(_) => {
                    self.scan_count.fetch_add(1, Ordering::Relaxed);
                    self.consecutive_errors.store(0, Ordering::Relaxed);
                }
                Err(e) => {
                    error!("Scan cycle error: {}", e);
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    let consecutive_errors = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    if consecutive_errors >= self.engine_config.max_consecutive_errors {
                        error!("Too many consecutive errors ({}), stopping engine", consecutive_errors);
                        self.stop();
                        return Err(PlcError::Runtime(format!(
                            "Too many consecutive errors: {}", consecutive_errors
                        )));
                    }
                }
            }

            let scan_duration = scan_start.elapsed();
            self.update_timing_stats(scan_duration).await;
        }

        info!("PETRA engine stopped");
        Ok(())
    }

    /// Execute a single scan cycle
    pub async fn execute_scan_cycle(&self) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let mut execution_order = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut failed_blocks = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut block_times = HashMap::new();

        // Lock blocks for the duration of the scan cycle
        let mut blocks = self.blocks.lock().await;
        
        // Execute all blocks in order
        for block in blocks.iter_mut() {
            let block_name = block.name().to_string();
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                execution_order.push(block_name.clone());
            }
            
            let block_start = Instant::now();
            
            // Execute block
            let result = block.execute(&self.bus);
            
            let block_duration = block_start.elapsed();
            
            match result {
                Ok(_) => {
                    trace!("Block '{}' executed successfully in {:?}", block_name, block_duration);
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        histogram!("petra_block_execution_time_us")
                            .record(block_duration.as_micros() as f64);
                    }
                }
                Err(e) => {
                    warn!("Block '{}' failed: {}", block_name, e);
                    
                    #[cfg(feature = "enhanced-monitoring")]
                    if self.engine_config.enhanced_monitoring {
                        failed_blocks.push(block_name.clone());
                    }
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_block_errors").increment(1);
                    }
                    
                    // Continue executing other blocks even if one fails
                    // This ensures partial functionality during errors
                }
            }
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                block_times.insert(block_name, block_duration);
            }
        }

        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            // Update monitoring data
            *self.execution_order.write().await = execution_order;
            *self.failed_blocks.write().await = failed_blocks;
            *self.block_execution_times.write().await = block_times;
        }

        Ok(())
    }
    
    /// Update timing statistics
    async fn update_timing_stats(&self, scan_duration: Duration) {
        let mut stats = self.stats.write().await;
        
        stats.scan_count += 1;
        stats.total_scan_time += scan_duration;
        stats.last_scan_time = scan_duration;
        
        if scan_duration < stats.min_scan_time {
            stats.min_scan_time = scan_duration;
        }
        if scan_duration > stats.max_scan_time {
            stats.max_scan_time = scan_duration;
        }
        
        stats.avg_scan_time = stats.total_scan_time / stats.scan_count as u32;
        
        // Calculate jitter (deviation from target)
        let target_nanos = self.target_scan_time.as_nanos() as i128;
        let actual_nanos = scan_duration.as_nanos() as i128;
        let jitter_nanos = (actual_nanos - target_nanos).abs() as u64;
        stats.jitter = Duration::from_nanos(jitter_nanos);
        
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            let mut scan_times = self.scan_times.write().await;
            scan_times.push_back(scan_duration);
            
            // Keep only last 1000 scan times
            if scan_times.len() > 1000 {
                scan_times.pop_front();
            }
        }
        
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_scan_time_us").set(scan_duration.as_micros() as f64);
            gauge!("petra_jitter_us").set(stats.jitter.as_micros() as f64);
            gauge!("petra_scan_count").set(stats.scan_count as f64);
        }
        
        // Log performance warnings
        if self.engine_config.performance_logging {
            let jitter_threshold = self.target_scan_time / 10; // 10% threshold
            if stats.jitter > jitter_threshold {
                warn!(
                    "High jitter detected: {:?} (target: {:?}, actual: {:?})",
                    stats.jitter, self.target_scan_time, scan_duration
                );
            }
        }
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
    
    /// Get reference to signal bus
    pub fn signal_bus(&self) -> &SignalBus {
        &self.bus
    }
    
    /// Get engine statistics
    pub async fn stats(&self) -> EngineStats {
        self.stats.read().await.clone()
    }
    
    /// Reset all blocks
    pub async fn reset_blocks(&self) -> Result<()> {
        let mut blocks = self.blocks.lock().await;
        for block in blocks.iter_mut() {
            block.reset()?;
        }
        
        // Reset statistics
        self.scan_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
        
        let mut stats = self.stats.write().await;
        *stats = EngineStats {
            min_scan_time: Duration::MAX,
            max_scan_time: Duration::ZERO,
            ..Default::default()
        };
        
        Ok(())
    }
    
    /// Get current scan count
    pub fn scan_count(&self) -> u64 {
        self.scan_count.load(Ordering::Relaxed)
    }
    
    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
    
    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get target scan time
    pub fn target_scan_time(&self) -> Duration {
        self.target_scan_time
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get execution order from last scan
    pub async fn execution_order(&self) -> Vec<String> {
        self.execution_order.read().await.clone()
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get failed blocks from last scan
    pub async fn failed_blocks(&self) -> Vec<String> {
        self.failed_blocks.read().await.clone()
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get block execution times from last scan
    pub async fn block_execution_times(&self) -> HashMap<String, Duration> {
        self.block_execution_times.read().await.clone()
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Get scan time history
    pub async fn scan_time_history(&self) -> Vec<Duration> {
        self.scan_times.read().await.iter().cloned().collect()
    }
}

// Make Engine cloneable for testing
#[cfg(test)]
impl Clone for Engine {
    fn clone(&self) -> Self {
        Self {
            bus: self.bus.clone(),
            blocks: Arc::new(Mutex::new(Vec::new())), // Empty blocks for testing
            config: self.config.clone(),
            engine_config: self.engine_config.clone(),
            running: self.running.clone(),
            scan_count: self.scan_count.clone(),
            error_count: self.error_count.clone(),
            consecutive_errors: self.consecutive_errors.clone(),
            start_time: self.start_time,
            target_scan_time: self.target_scan_time,
            stats: self.stats.clone(),
            
            #[cfg(feature = "enhanced-monitoring")]
            scan_times: self.scan_times.clone(),
            #[cfg(feature = "enhanced-monitoring")]
            execution_order: self.execution_order.clone(),
            #[cfg(feature = "enhanced-monitoring")]
            failed_blocks: self.failed_blocks.clone(),
            #[cfg(feature = "enhanced-monitoring")]
            block_execution_times: self.block_execution_times.clone(),
            
            #[cfg(feature = "circuit-breaker")]
            block_executor: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SignalConfig, EngineConfig as ConfigEngineConfig};
    use std::collections::HashMap;
    
    fn create_test_config() -> Config {
        Config {
            engine: ConfigEngineConfig {
                scan_time_ms: 100,
                max_scan_jitter_ms: 50,
                error_recovery: true,
            },
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
    
    #[tokio::test]
    async fn test_reset_blocks() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        engine.reset_blocks().await.unwrap();
        assert_eq!(engine.scan_count(), 0);
        assert_eq!(engine.error_count(), 0);
    }
}

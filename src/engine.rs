//! PETRA execution engine with feature-organized monitoring and optimization
//!
//! This module provides the real-time execution engine with support for:
//! - Standard monitoring: Basic performance tracking and jitter detection
//! - Enhanced monitoring: Detailed statistics and per-block timing
//! - Metrics integration: Prometheus metrics export
//! - Real-time scheduling: Priority and affinity management
//! - Circuit breakers: Enhanced error handling and fault tolerance

use crate::{
    error::*,
    signal::SignalBus,
    value::Value,
    config::Config,
    blocks::{create_block, Block},
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use tokio::time::interval;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn, debug, error, trace};
use serde::{Serialize, Deserialize};

#[cfg(feature = "metrics")]
use metrics::{counter, gauge, histogram, register_counter, register_gauge, register_histogram};

#[cfg(feature = "enhanced-monitoring")]
use ringbuffer::{AllocRingBuffer, RingBuffer};

#[cfg(feature = "realtime")]
use std::thread;

#[cfg(feature = "circuit-breaker")]
use crate::blocks::BlockExecutor;

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// ============================================================================
// ENGINE STATISTICS STRUCTURES
// ============================================================================

/// Basic engine statistics (always available)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct EngineStats {
    /// Engine running state
    pub running: bool,
    /// Total scan cycles completed
    pub scan_count: u64,
    /// Total errors encountered
    pub error_count: u64,
    /// Engine uptime in seconds
    pub uptime_secs: u64,
    /// Number of configured signals
    pub signal_count: usize,
    /// Number of configured blocks
    pub block_count: usize,
    /// Target scan time in milliseconds
    pub target_scan_time_ms: u64,
    /// Last scan duration in microseconds
    pub last_scan_time_us: Option<u64>,
    
    // Enhanced monitoring fields (feature-gated)
    #[cfg(feature = "enhanced-monitoring")]
    /// Average scan time in microseconds
    pub avg_scan_time_us: Option<f64>,
    #[cfg(feature = "enhanced-monitoring")]
    /// Maximum scan time in microseconds
    pub max_scan_time_us: Option<f64>,
    #[cfg(feature = "enhanced-monitoring")]
    /// Per-block execution times in microseconds
    pub block_times_us: Option<HashMap<String, f64>>,
    #[cfg(feature = "enhanced-monitoring")]
    /// Jitter statistics
    pub jitter_stats: Option<JitterStats>,
    #[cfg(feature = "enhanced-monitoring")]
    /// Memory usage information
    pub memory_stats: Option<MemoryStats>,
}

/// Detailed engine statistics (requires enhanced-monitoring)
#[cfg(feature = "enhanced-monitoring")]
#[derive(Clone, Debug, Serialize)]
pub struct DetailedStats {
    /// Basic statistics
    pub basic: EngineStats,
    /// Recent scan time history
    pub scan_times: Vec<Duration>,
    /// Per-block execution times
    pub block_execution_times: HashMap<String, Duration>,
    /// Jitter analysis
    pub jitter_stats: JitterStats,
    /// Memory usage tracking
    pub memory_stats: MemoryStats,
    /// Block execution order
    pub execution_order: Vec<String>,
    /// Failed blocks in last cycle
    pub failed_blocks: Vec<String>,
}

/// Jitter statistics for timing analysis
#[cfg(feature = "enhanced-monitoring")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct JitterStats {
    /// Average jitter in microseconds
    pub avg_us: f64,
    /// Maximum jitter in microseconds
    pub max_us: f64,
    /// Jitter variance in microseconds
    pub variance_us: f64,
    /// Number of scan overruns
    pub overrun_count: u64,
    /// Percentage of scans that overran target time
    pub overrun_percentage: f64,
}

/// Memory usage statistics
#[cfg(feature = "enhanced-monitoring")]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MemoryStats {
    /// Heap memory usage in bytes
    pub heap_bytes: u64,
    /// Stack memory usage in bytes (estimate)
    pub stack_bytes: u64,
    /// Signal bus memory usage in bytes
    pub signal_bus_bytes: u64,
    /// Block memory usage in bytes
    pub block_memory_bytes: u64,
}

// ============================================================================
// ENGINE CONFIGURATION
// ============================================================================

/// Engine runtime configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable enhanced monitoring
    pub enhanced_monitoring: bool,
    /// Enable metrics collection
    pub metrics_enabled: bool,
    /// Enable real-time scheduling
    pub realtime_enabled: bool,
    /// CPU affinity mask (for real-time)
    pub cpu_affinity: Option<u64>,
    /// Thread priority (for real-time)
    pub thread_priority: Option<i32>,
    /// Jitter threshold for warnings (microseconds)
    pub jitter_threshold_us: u64,
    /// Maximum consecutive errors before stopping
    pub max_consecutive_errors: u32,
    /// Circuit breaker configuration
    #[cfg(feature = "circuit-breaker")]
    pub circuit_breaker_enabled: bool,
    #[cfg(feature = "circuit-breaker")]
    pub circuit_breaker_threshold: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
            metrics_enabled: cfg!(feature = "metrics"),
            realtime_enabled: cfg!(feature = "realtime"),
            cpu_affinity: None,
            thread_priority: None,
            jitter_threshold_us: 1000, // 1ms
            max_consecutive_errors: 10,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker_enabled: true,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker_threshold: 5,
        }
    }
}

// ============================================================================
// MAIN ENGINE STRUCTURE
// ============================================================================

/// Main PETRA execution engine
pub struct Engine {
    // Core components
    bus: SignalBus,
    blocks: Vec<Box<dyn Block>>,
    config: Config,
    engine_config: EngineConfig,
    
    // Runtime state
    running: Arc<AtomicBool>,
    scan_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    consecutive_errors: Arc<AtomicU64>,
    start_time: Instant,
    target_scan_time: Duration,
    
    // Signal change notifications
    signal_change_tx: Option<mpsc::Sender<(String, Value)>>,
    
    // Standard monitoring (always available)
    stats_handle: Arc<RwLock<EngineStats>>,
    scan_jitter_buffer: Arc<Mutex<VecDeque<Duration>>>,
    
    // Enhanced monitoring (feature-gated)
    #[cfg(feature = "enhanced-monitoring")]
    scan_times: Arc<RwLock<AllocRingBuffer<Duration>>>,
    #[cfg(feature = "enhanced-monitoring")]
    block_execution_times: Arc<RwLock<HashMap<String, Duration>>>,
    #[cfg(feature = "enhanced-monitoring")]
    execution_order: Arc<RwLock<Vec<String>>>,
    #[cfg(feature = "enhanced-monitoring")]
    failed_blocks: Arc<RwLock<Vec<String>>>,
    
    // Circuit breakers (feature-gated)
    #[cfg(feature = "circuit-breaker")]
    block_executors: HashMap<String, BlockExecutor>,
    
    // Real-time scheduling (feature-gated)
    #[cfg(feature = "realtime")]
    rt_handle: Option<thread::JoinHandle<()>>,
}

// Safety: Engine is designed to be used in a single-threaded async context
// All shared state is properly synchronized with Arc<Mutex<>> or Arc<RwLock<>>
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

// ============================================================================
// ENGINE IMPLEMENTATION
// ============================================================================

impl Engine {
    /// Create a new engine instance
    pub fn new(config: Config) -> Result<Self> {
        Self::new_with_engine_config(config, EngineConfig::default())
    }
    
    /// Create engine with custom engine configuration
    pub fn new_with_engine_config(config: Config, engine_config: EngineConfig) -> Result<Self> {
        info!("Initializing PETRA engine with {} signals and {} blocks", 
              config.signals.len(), config.blocks.len());
        
        // Initialize metrics if enabled
        #[cfg(feature = "metrics")]
        if engine_config.metrics_enabled {
            Self::initialize_metrics(&config)?;
        }
        
        // Initialize signal bus
        let bus = SignalBus::new();
        
        // Initialize all signals with their configured initial values
        for signal in &config.signals {
            let initial_value = signal.initial.as_ref()
                .map(|v| Self::parse_initial_value(v, &signal.signal_type))
                .transpose()?
                .unwrap_or_else(|| Self::default_value_for_type(&signal.signal_type));
            
            bus.set(&signal.name, initial_value.clone())?;
            
            debug!("Initialized signal '{}' as {} = {}", 
                   signal.name, signal.signal_type.as_str(), initial_value);
        }

        // Create all blocks
        let mut blocks = Vec::with_capacity(config.blocks.len());
        #[cfg(feature = "circuit-breaker")]
        let mut block_executors = HashMap::new();
        
        for block_config in &config.blocks {
            match create_block(block_config) {
                Ok(mut block) => {
                    // Initialize the block
                    block.initialize(block_config)?;
                    
                    info!("Created block '{}' of type '{}'", 
                          block_config.name, block_config.block_type);
                    
                    #[cfg(feature = "circuit-breaker")]
                    if engine_config.circuit_breaker_enabled {
                        let executor = BlockExecutor::new(
                            engine_config.circuit_breaker_threshold,
                            Duration::from_millis(5000), // 5 second reset timeout
                        );
                        block_executors.insert(block_config.name.clone(), executor);
                    }
                    
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

        let target_scan_time = Duration::from_millis(config.scan_time_ms);
        let start_time = Instant::now();
        
        // Initialize stats
        let stats = EngineStats {
            running: false,
            scan_count: 0,
            error_count: 0,
            uptime_secs: 0,
            signal_count: config.signals.len(),
            block_count: config.blocks.len(),
            target_scan_time_ms: config.scan_time_ms,
            last_scan_time_us: None,
            
            #[cfg(feature = "enhanced-monitoring")]
            avg_scan_time_us: None,
            #[cfg(feature = "enhanced-monitoring")]
            max_scan_time_us: None,
            #[cfg(feature = "enhanced-monitoring")]
            block_times_us: None,
            #[cfg(feature = "enhanced-monitoring")]
            jitter_stats: None,
            #[cfg(feature = "enhanced-monitoring")]
            memory_stats: None,
        };

        info!("PETRA engine initialized successfully");

        Ok(Self {
            bus,
            blocks,
            config,
            engine_config,
            running: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            consecutive_errors: Arc::new(AtomicU64::new(0)),
            start_time,
            target_scan_time,
            signal_change_tx: None,
            stats_handle: Arc::new(RwLock::new(stats)),
            scan_jitter_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            
            #[cfg(feature = "enhanced-monitoring")]
            scan_times: Arc::new(RwLock::new(AllocRingBuffer::with_capacity(1000))),
            #[cfg(feature = "enhanced-monitoring")]
            block_execution_times: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "enhanced-monitoring")]
            execution_order: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "enhanced-monitoring")]
            failed_blocks: Arc::new(RwLock::new(Vec::new())),
            
            #[cfg(feature = "circuit-breaker")]
            block_executors,
            
            #[cfg(feature = "realtime")]
            rt_handle: None,
        })
    }

    /// Start the engine main loop
    pub async fn run(&mut self) -> Result<()> {
        self.running.store(true, Ordering::Relaxed);
        info!("Starting PETRA engine with scan time: {}ms", self.config.engine.scan_time_ms);
        
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_engine_running").set(1.0);
            gauge!("petra_engine_scan_time_ms").set(self.config.scan_time_ms as f64);
            gauge!("petra_engine_signal_count").set(self.config.signals.len() as f64);
            gauge!("petra_engine_block_count").set(self.config.blocks.len() as f64);
        }

        // Update stats to running
        {
            let mut stats = self.stats_handle.write().await;
            stats.running = true;
        }

        // Apply real-time scheduling if enabled
        #[cfg(feature = "realtime")]
        if self.engine_config.realtime_enabled {
            self.apply_realtime_scheduling()?;
        }

        let mut interval = interval(self.target_scan_time);
        
        while self.running.load(Ordering::Relaxed) {
            let scan_start = Instant::now();
            
            // Execute scan cycle
            match self.execute_scan_cycle().await {
                Ok(_) => {
                    let scan_count = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;
                    self.consecutive_errors.store(0, Ordering::Relaxed);
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_scan_count_total").increment(1);
                    }
                    
                    trace!("Scan cycle {} completed", scan_count);
                }
                Err(e) => {
                    let error_count = self.error_count.fetch_add(1, Ordering::Relaxed) + 1;
                    let consecutive_errors = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    error!("Scan cycle error #{}: {}", error_count, e);
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_engine_errors_total").increment(1);
                    }
                    
                    // Check if we should stop due to too many consecutive errors
                    if consecutive_errors >= self.engine_config.max_consecutive_errors as u64 {
                        error!("Too many consecutive errors ({}), stopping engine", consecutive_errors);
                        self.stop();
                        return Err(PlcError::Engine(format!(
                            "Too many consecutive errors: {}", consecutive_errors
                        )));
                    }
                }
            }

            let scan_duration = scan_start.elapsed();
            self.update_timing_stats(scan_duration).await;

            // Wait for next scan cycle
            interval.tick().await;
        }

        info!("PETRA engine stopped");
        
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_engine_running").set(0.0);
        }

        // Update stats to stopped
        {
            let mut stats = self.stats_handle.write().await;
            stats.running = false;
        }

        Ok(())
    }

    /// Execute a single scan cycle
    pub async fn execute_scan_cycle(&mut self) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let mut execution_order = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut failed_blocks = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut block_times = HashMap::new();

        // Execute all blocks in order
        for block in &mut self.blocks {
            let block_name = block.name().to_string();
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                execution_order.push(block_name.clone());
            }
            
            let block_start = Instant::now();
            
            // Execute block with or without circuit breaker
            let result = {
                #[cfg(feature = "circuit-breaker")]
                if self.engine_config.circuit_breaker_enabled {
                    if let Some(executor) = self.block_executors.get(&block_name) {
                        executor.execute_with_circuit_breaker(block.as_mut(), &self.bus)
                    } else {
                        block.execute(&self.bus)
                    }
                } else {
                    block.execute(&self.bus)
                }
                
                #[cfg(not(feature = "circuit-breaker"))]
                block.execute(&self.bus)
            };
            
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
                    warn!("Block '{}' execution failed: {}", block_name, e);
                    
                    #[cfg(feature = "enhanced-monitoring")]
                    if self.engine_config.enhanced_monitoring {
                        failed_blocks.push(block_name.clone());
                    }
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_block_errors_total").increment(1);
                    }
                    
                    return Err(e);
                }
            }
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                block_times.insert(block_name, block_duration);
            }
        }

        // Update enhanced monitoring data
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            {
                let mut exec_order = self.execution_order.write().await;
                *exec_order = execution_order;
            }
            
            {
                let mut failed = self.failed_blocks.write().await;
                *failed = failed_blocks;
            }
            
            {
                let mut times = self.block_execution_times.write().await;
                *times = block_times;
            }
        }

        Ok(())
    }

    /// Update timing statistics
    async fn update_timing_stats(&self, scan_duration: Duration) {
        // Update basic jitter buffer
        if let Ok(mut jitter_buffer) = self.scan_jitter_buffer.try_lock() {
            jitter_buffer.push_back(scan_duration);
            if jitter_buffer.len() > 100 {
                jitter_buffer.pop_front();
            }
        }

        // Update enhanced monitoring
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            if let Ok(mut scan_times) = self.scan_times.try_write() {
                scan_times.push(scan_duration);
            }
        }

        // Update metrics
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            histogram!("petra_scan_duration_us").record(scan_duration.as_micros() as f64);
            gauge!("petra_scan_duration_ms").set(scan_duration.as_millis() as f64);
            
            // Calculate jitter metrics
            if let Ok(jitter_buffer) = self.scan_jitter_buffer.try_lock() {
                if jitter_buffer.len() > 1 {
                    let avg: Duration = jitter_buffer.iter().sum::<Duration>() / jitter_buffer.len() as u32;
                    let max = *jitter_buffer.iter().max().unwrap();
                    let jitter = if scan_duration > self.target_scan_time {
                        scan_duration - self.target_scan_time
                    } else {
                        self.target_scan_time - scan_duration
                    };
                    
                    gauge!("petra_scan_jitter_avg_us").set(avg.as_micros() as f64);
                    gauge!("petra_scan_jitter_max_us").set(max.as_micros() as f64);
                    gauge!("petra_scan_variance_us").set(jitter.as_micros() as f64);
                    
                    if scan_duration > self.target_scan_time {
                        counter!("petra_scan_overruns_total").increment(1);
                        
                        if jitter.as_micros() > self.engine_config.jitter_threshold_us as u128 {
                            warn!("Scan overrun detected: {:?} > target {:?} (jitter: {:?})",
                                  scan_duration, self.target_scan_time, jitter);
                        }
                    }
                }
            }
        }

        // Update stats handle
        if let Ok(mut stats) = self.stats_handle.try_write() {
            stats.scan_count = self.scan_count.load(Ordering::Relaxed);
            stats.error_count = self.error_count.load(Ordering::Relaxed);
            stats.uptime_secs = self.start_time.elapsed().as_secs();
            stats.last_scan_time_us = Some(scan_duration.as_micros() as u64);

            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                // Update enhanced stats
                if let Ok(scan_times) = self.scan_times.try_read() {
                    if !scan_times.is_empty() {
                        let avg_us = scan_times.iter().sum::<Duration>().as_micros() as f64 / scan_times.len() as f64;
                        let max_us = scan_times.iter().max().map(|d| d.as_micros() as f64);
                        
                        stats.avg_scan_time_us = Some(avg_us);
                        stats.max_scan_time_us = max_us;
                    }
                }

                if let Ok(block_times) = self.block_execution_times.try_read() {
                    let block_times_us: HashMap<String, f64> = block_times
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_micros() as f64))
                        .collect();
                    stats.block_times_us = Some(block_times_us);
                }

                // Calculate jitter stats
                if let Ok(jitter_buffer) = self.scan_jitter_buffer.try_lock() {
                    if jitter_buffer.len() > 1 {
                        let avg_us = jitter_buffer.iter().sum::<Duration>().as_micros() as f64 / jitter_buffer.len() as f64;
                        let max_us = jitter_buffer.iter().max().unwrap().as_micros() as f64;
                        let variance = jitter_buffer.iter()
                            .map(|d| {
                                let diff = d.as_micros() as f64 - avg_us;
                                diff * diff
                            })
                            .sum::<f64>() / jitter_buffer.len() as f64;
                        
                        let overrun_count = jitter_buffer.iter()
                            .filter(|&d| *d > self.target_scan_time)
                            .count() as u64;
                        let overrun_percentage = (overrun_count as f64 / jitter_buffer.len() as f64) * 100.0;
                        
                        stats.jitter_stats = Some(JitterStats {
                            avg_us,
                            max_us,
                            variance_us: variance.sqrt(),
                            overrun_count,
                            overrun_percentage,
                        });
                    }
                }

                // Update memory stats
                stats.memory_stats = Some(self.get_memory_stats());
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

    /// Get current engine statistics
    pub async fn get_stats(&self) -> EngineStats {
        self.stats_handle.read().await.clone()
    }

    /// Get detailed statistics (requires enhanced-monitoring)
    #[cfg(feature = "enhanced-monitoring")]
    pub async fn get_detailed_stats(&self) -> DetailedStats {
        let basic = self.get_stats().await;
        let scan_times = self.scan_times.read().await.to_vec();
        let block_execution_times = self.block_execution_times.read().await.clone();
        let execution_order = self.execution_order.read().await.clone();
        let failed_blocks = self.failed_blocks.read().await.clone();
        
        // Calculate jitter stats
        let jitter_stats = if !scan_times.is_empty() {
            let avg_us = scan_times.iter().sum::<Duration>().as_micros() as f64 / scan_times.len() as f64;
            let max_us = scan_times.iter().max().unwrap().as_micros() as f64;
            let variance = scan_times.iter()
                .map(|d| {
                    let diff = d.as_micros() as f64 - avg_us;
                    diff * diff
                })
                .sum::<f64>() / scan_times.len() as f64;
            
            let overrun_count = scan_times.iter()
                .filter(|&d| *d > self.target_scan_time)
                .count() as u64;
            let overrun_percentage = (overrun_count as f64 / scan_times.len() as f64) * 100.0;
            
            JitterStats {
                avg_us,
                max_us,
                variance_us: variance.sqrt(),
                overrun_count,
                overrun_percentage,
            }
        } else {
            JitterStats {
                avg_us: 0.0,
                max_us: 0.0,
                variance_us: 0.0,
                overrun_count: 0,
                overrun_percentage: 0.0,
            }
        };

        DetailedStats {
            basic,
            scan_times,
            block_execution_times,
            jitter_stats,
            memory_stats: self.get_memory_stats(),
            execution_order,
            failed_blocks,
        }
    }

    /// Get access to the signal bus
    pub fn signal_bus(&self) -> &SignalBus {
        &self.bus
    }

    /// Enable signal change notifications
    pub fn enable_signal_notifications(&mut self) -> mpsc::Receiver<(String, Value)> {
        let (tx, rx) = mpsc::channel(1000);
        self.signal_change_tx = Some(tx);
        rx
    }

    /// Reset all blocks
    pub async fn reset_blocks(&mut self) -> Result<()> {
        info!("Resetting all blocks");
        
        for block in &mut self.blocks {
            if let Err(e) = block.reset() {
                warn!("Failed to reset block '{}': {}", block.name(), e);
            }
        }
        
        // Reset circuit breakers
        #[cfg(feature = "circuit-breaker")]
        if self.engine_config.circuit_breaker_enabled {
            for executor in self.block_executors.values() {
                executor.reset();
            }
        }
        
        // Reset statistics
        self.scan_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
        
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            self.scan_times.write().await.clear();
            self.block_execution_times.write().await.clear();
            self.execution_order.write().await.clear();
            self.failed_blocks.write().await.clear();
        }
        
        info!("All blocks reset successfully");
        Ok(())
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    /// Initialize metrics registration
    #[cfg(feature = "metrics")]
    fn initialize_metrics(_config: &Config) -> Result<()> {
        register_counter!("petra_scan_count_total", "Total number of scan cycles completed");
        register_counter!("petra_engine_errors_total", "Total number of engine errors");
        register_counter!("petra_block_errors_total", "Total number of block execution errors");
        register_counter!("petra_scan_overruns_total", "Total number of scan time overruns");
        
        register_gauge!("petra_engine_running", "Engine running state (1=running, 0=stopped)");
        register_gauge!("petra_engine_scan_time_ms", "Configured scan time in milliseconds");
        register_gauge!("petra_engine_signal_count", "Number of configured signals");
        register_gauge!("petra_engine_block_count", "Number of configured blocks");
        register_gauge!("petra_scan_duration_ms", "Last scan duration in milliseconds");
        register_gauge!("petra_scan_jitter_avg_us", "Average scan jitter in microseconds");
        register_gauge!("petra_scan_jitter_max_us", "Maximum scan jitter in microseconds");
        register_gauge!("petra_scan_variance_us", "Scan time variance in microseconds");
        
        register_histogram!("petra_scan_duration_us", "Scan cycle duration in microseconds");
        register_histogram!("petra_block_execution_time_us", "Block execution time in microseconds");
        
        debug!("Metrics initialized");
        Ok(())
    }

    /// Parse initial value from configuration
    fn parse_initial_value(value: &serde_yaml::Value, signal_type: &str) -> Result<Value> {
        match signal_type {
            "bool" => {
                value.as_bool()
                    .map(Value::Bool)
                    .ok_or_else(|| PlcError::Config(format!("Invalid bool value: {:?}", value)))
            }
            "int" => {
                value.as_i64()
                    .map(|i| Value::Int(i as i32))
                    .ok_or_else(|| PlcError::Config(format!("Invalid int value: {:?}", value)))
            }
            "float" => {
                value.as_f64()
                    .map(Value::Float)
                    .ok_or_else(|| PlcError::Config(format!("Invalid float value: {:?}", value)))
            }
            #[cfg(feature = "extended-types")]
            "string" => {
                value.as_str()
                    .map(|s| Value::String(s.to_string()))
                    .ok_or_else(|| PlcError::Config(format!("Invalid string value: {:?}", value)))
            }
            _ => Err(PlcError::Config(format!("Unknown signal type: {}", signal_type)))
        }
    }

    /// Get default value for signal type
    fn default_value_for_type(signal_type: &str) -> Value {
        match signal_type {
            "bool" => Value::Bool(false),
            "int" => Value::Int(0),
            "float" => Value::Float(0.0),
            #[cfg(feature = "extended-types")]
            "string" => Value::String(String::new()),
            _ => Value::Bool(false), // fallback
        }
    }

    /// Apply real-time scheduling
    #[cfg(feature = "realtime")]
    fn apply_realtime_scheduling(&self) -> Result<()> {
        use std::thread;
        
        if let Some(priority) = self.engine_config.thread_priority {
            // Set thread priority (platform-specific)
            #[cfg(target_os = "linux")]
            {
                use libc::{pthread_self, pthread_setschedparam, sched_param, SCHED_FIFO};
                use std::mem;
                
                unsafe {
                    let mut param: sched_param = mem::zeroed();
                    param.sched_priority = priority;
                    
                    let result = pthread_setschedparam(pthread_self(), SCHED_FIFO, &param);
                    if result != 0 {
                        warn!("Failed to set thread priority: {}", result);
                    } else {
                        info!("Set real-time thread priority: {}", priority);
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadPriority};
                use winapi::um::winbase::THREAD_PRIORITY_TIME_CRITICAL;
                
                unsafe {
                    let result = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
                    if result == 0 {
                        warn!("Failed to set thread priority");
                    } else {
                        info!("Set real-time thread priority");
                    }
                }
            }
        }

        if let Some(affinity) = self.engine_config.cpu_affinity {
            // Set CPU affinity (platform-specific)
            #[cfg(target_os = "linux")]
            {
                use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
                use std::mem;
                
                unsafe {
                    let mut cpuset: cpu_set_t = mem::zeroed();
                    CPU_ZERO(&mut cpuset);
                    
                    for i in 0..64 {
                        if (affinity & (1 << i)) != 0 {
                            CPU_SET(i, &mut cpuset);
                        }
                    }
                    
                    let result = sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpuset);
                    if result != 0 {
                        warn!("Failed to set CPU affinity: {}", result);
                    } else {
                        info!("Set CPU affinity: 0x{:x}", affinity);
                    }
                }
            }
            
            #[cfg(target_os = "windows")]
            {
                use winapi::um::processthreadsapi::{GetCurrentThread, SetThreadAffinityMask};
                
                unsafe {
                    let result = SetThreadAffinityMask(GetCurrentThread(), affinity);
                    if result == 0 {
                        warn!("Failed to set CPU affinity");
                    } else {
                        info!("Set CPU affinity: 0x{:x}", affinity);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get memory usage statistics
    #[cfg(feature = "enhanced-monitoring")]
    fn get_memory_stats(&self) -> MemoryStats {
        // This is a simplified implementation
        // In a real implementation, you would use platform-specific APIs
        // to get accurate memory usage information
        
        let heap_bytes = self.estimate_heap_usage();
        let stack_bytes = self.estimate_stack_usage();
        let signal_bus_bytes = self.bus.memory_usage();
        let block_memory_bytes = self.estimate_block_memory();
        
        MemoryStats {
            heap_bytes,
            stack_bytes,
            signal_bus_bytes,
            block_memory_bytes,
        }
    }

    #[cfg(feature = "enhanced-monitoring")]
    fn estimate_heap_usage(&self) -> u64 {
        // Rough estimate based on data structures
        let mut total = 0u64;
        
        // Signal bus estimate
        total += self.config.signals.len() as u64 * 64; // rough estimate per signal
        
        // Block estimate  
        total += self.config.blocks.len() as u64 * 256; // rough estimate per block
        
        // Ring buffers and collections
        total += 1000 * 16; // scan times buffer
        total += 100 * 16;  // jitter buffer
        
        total
    }

    #[cfg(feature = "enhanced-monitoring")]
    fn estimate_stack_usage(&self) -> u64 {
        // Very rough estimate - in practice this would need platform-specific code
        4096 * 16 // assume ~64KB stack usage
    }

    #[cfg(feature = "enhanced-monitoring")]
    fn estimate_block_memory(&self) -> u64 {
        // Estimate memory used by blocks themselves
        self.blocks.len() as u64 * 512 // rough estimate
    }
}

// ============================================================================
// DROP IMPLEMENTATION
// ============================================================================

impl Drop for Engine {
    fn drop(&mut self) {
        if self.is_running() {
            info!("Engine dropped while running, stopping...");
            self.stop();
        }
    }
}

// ============================================================================
// TESTING UTILITIES
// ============================================================================

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::config::{SignalConfig, BlockConfig};
    
    /// Create a test engine configuration
    pub fn create_test_config() -> Config {
        Config {
            scan_time_ms: 10,
            signals: vec![
                SignalConfig {
                    name: "test_input".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: Some("Test input signal".to_string()),
                    tags: vec!["test".to_string()],
                },
                SignalConfig {
                    name: "test_output".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: Some("Test output signal".to_string()),
                    tags: vec!["test".to_string()],
                },
            ],
            blocks: vec![
                BlockConfig {
                    name: "test_not".to_string(),
                    block_type: "NOT".to_string(),
                    inputs: {
                        let mut map = HashMap::new();
                        map.insert("input".to_string(), "test_input".to_string());
                        map
                    },
                    outputs: {
                        let mut map = HashMap::new();
                        map.insert("output".to_string(), "test_output".to_string());
                        map
                    },
                    params: HashMap::new(),
                    description: Some("Test NOT block".to_string()),
                    tags: vec!["test".to_string()],
                    #[cfg(feature = "enhanced-errors")]
                    error_handling: None,
                    #[cfg(feature = "circuit-breaker")]
                    circuit_breaker: None,
                },
            ],
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "validation")]
            validation: None,
        }
    }
    
    /// Create a test engine with minimal configuration
    pub fn create_test_engine() -> Result<Engine> {
        Engine::new(create_test_config())
    }
    
    /// Create a benchmark configuration with specified number of signals and blocks
    pub fn create_benchmark_config(signal_count: usize, block_count: usize) -> Config {
        let mut signals = Vec::with_capacity(signal_count);
        let mut blocks = Vec::with_capacity(block_count);
        
        // Create signals
        for i in 0..signal_count {
            signals.push(SignalConfig {
                name: format!("signal_{}", i),
                signal_type: if i % 3 == 0 { "bool" } else if i % 3 == 1 { "int" } else { "float" }.to_string(),
                initial: None,
                description: Some(format!("Benchmark signal {}", i)),
                tags: vec!["benchmark".to_string()],
            });
        }
        
        // Create blocks (simple NOT blocks for performance testing)
        for i in 0..block_count.min(signal_count / 2) {
            let input_idx = i * 2;
            let output_idx = i * 2 + 1;
            
            blocks.push(BlockConfig {
                name: format!("block_{}", i),
                block_type: "NOT".to_string(),
                inputs: {
                    let mut map = HashMap::new();
                    map.insert("input".to_string(), format!("signal_{}", input_idx));
                    map
                },
                outputs: {
                    let mut map = HashMap::new();
                    map.insert("output".to_string(), format!("signal_{}", output_idx));
                    map
                },
                params: HashMap::new(),
                description: Some(format!("Benchmark block {}", i)),
                tags: vec!["benchmark".to_string()],
                #[cfg(feature = "enhanced-errors")]
                error_handling: None,
                #[cfg(feature = "circuit-breaker")]
                circuit_breaker: None,
            });
        }
        
        Config {
            scan_time_ms: 1, // 1ms for high-performance benchmarking
            signals,
            blocks,
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "validation")]
            validation: None,
        }
    }
}

// ============================================================================
// MODULE TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout};
    
    #[tokio::test]
    async fn test_engine_creation() {
        let config = test_utils::create_test_config();
        let engine = Engine::new(config).unwrap();
        
        assert!(!engine.is_running());
        assert_eq!(engine.signal_bus().get("test_input").unwrap(), Value::Bool(false));
        assert_eq!(engine.signal_bus().get("test_output").unwrap(), Value::Bool(false));
    }
    
    #[tokio::test]
    async fn test_single_scan_cycle() {
        let config = test_utils::create_test_config();
        let mut engine = Engine::new(config).unwrap();
        
        // Set input signal
        engine.signal_bus().set("test_input", Value::Bool(true)).unwrap();
        
        // Execute one scan cycle
        engine.execute_scan_cycle().await.unwrap();
        
        // Check that NOT gate worked correctly
        assert_eq!(engine.signal_bus().get("test_output").unwrap(), Value::Bool(false));
        
        // Change input and execute again
        engine.signal_bus().set("test_input", Value::Bool(false)).unwrap();
        engine.execute_scan_cycle().await.unwrap();
        
        assert_eq!(engine.signal_bus().get("test_output").unwrap(), Value::Bool(true));
    }
    
    #[tokio::test]
    async fn test_engine_start_stop() {
        let config = test_utils::create_test_config();
        let mut engine = Engine::new(config).unwrap();
        
        assert!(!engine.is_running());
        
        // Start engine in background
        let engine_handle = tokio::spawn(async move {
            engine.run().await
        });
        
        // Give it a moment to start
        sleep(Duration::from_millis(50)).await;
        
        // Stop the engine by dropping the handle (this is a simplified test)
        engine_handle.abort();
    }
    
    #[tokio::test]
    async fn test_engine_stats() {
        let config = test_utils::create_test_config();
        let engine = Engine::new(config).unwrap();
        
        let stats = engine.get_stats().await;
        
        assert!(!stats.running);
        assert_eq!(stats.scan_count, 0);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.signal_count, 2);
        assert_eq!(stats.block_count, 1);
        assert_eq!(stats.target_scan_time_ms, 10);
    }
    
    #[cfg(feature = "enhanced-monitoring")]
    #[tokio::test]
    async fn test_detailed_stats() {
        let config = test_utils::create_test_config();
        let engine_config = EngineConfig {
            enhanced_monitoring: true,
            ..Default::default()
        };
        let engine = Engine::new_with_engine_config(config, engine_config).unwrap();
        
        let detailed_stats = engine.get_detailed_stats().await;
        
        assert!(detailed_stats.scan_times.is_empty());
        assert!(detailed_stats.block_execution_times.is_empty());
        assert!(detailed_stats.execution_order.is_empty());
        assert!(detailed_stats.failed_blocks.is_empty());
    }
    
    #[tokio::test]
    async fn test_reset_blocks() {
        let config = test_utils::create_test_config();
        let mut engine = Engine::new(config).unwrap();
        
        // Execute a few cycles to build up stats
        engine.execute_scan_cycle().await.unwrap();
        engine.execute_scan_cycle().await.unwrap();
        
        let stats_before = engine.get_stats().await;
        assert!(stats_before.scan_count > 0);
        
        // Reset blocks
        engine.reset_blocks().await.unwrap();
        
        let stats_after = engine.get_stats().await;
        assert_eq!(stats_after.scan_count, 0);
        assert_eq!(stats_after.error_count, 0);
    }
    
    #[tokio::test]
    async fn test_benchmark_config() {
        let config = test_utils::create_benchmark_config(1000, 100);
        
        assert_eq!(config.signals.len(), 1000);
        assert_eq!(config.blocks.len(), 100);
        assert_eq!(config.scan_time_ms, 1);
        
        // Verify we can create an engine with this config
        let engine = Engine::new(config).unwrap();
        assert_eq!(engine.signal_bus().signal_count(), 1000);
    }
}

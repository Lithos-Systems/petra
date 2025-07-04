// src/engine.rs
//! PETRA Engine - Real-time Automation Execution Core
//! 
//! This module implements the heart of PETRA: a deterministic, real-time capable
//! automation engine that executes logic blocks in precise scan cycles.
//! It provides real-time execution with precise timing control, comprehensive 
//! error handling, and detailed performance monitoring.
//!
//! # Architecture Overview
//!
//! The engine operates as the central coordinator for all PETRA components:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           PETRA Engine                                  │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
//! │  │ Signal Bus   │◄──►│ Scan Engine  │◄──►│ Block System │               │
//! │  │              │    │              │    │              │               │
//! │  │ • DashMap    │    │ • Timing     │    │ • Logic      │               │
//! │  │ • Atomic Ops │    │ • Scheduling │    │ • Math       │               │
//! │  │ • Events     │    │ • Error Mgmt │    │ • Control    │               │
//! │  └──────────────┘    └──────────────┘    └──────────────┘               │
//! │         ▲                    │                    │                     │
//! │         │                    ▼                    ▼                     │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐               │
//! │  │ Protocol     │    │ Performance  │    │ Persistence  │               │
//! │  │ Drivers      │    │ Monitoring   │    │ & History    │               │
//! │  │              │    │              │    │              │               │
//! │  │ • MQTT       │    │ • Metrics    │    │ • Parquet    │               │
//! │  │ • Modbus     │    │ • Jitter     │    │ • WAL        │               │
//! │  │ • S7/OPC-UA  │    │ • Profiling  │    │ • S3/Cloud   │               │
//! │  └──────────────┘    └──────────────┘    └──────────────┘               │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Deterministic Execution**: Fixed scan cycles with configurable timing
//! - **Real-time Support**: Optional real-time scheduling with `realtime` feature
//! - **Hot Reload**: Dynamic block and configuration updates without restart
//! - **Error Recovery**: Comprehensive error handling with automatic recovery
//! - **Performance Monitoring**: Detailed metrics including jitter analysis
//! - **Thread Safety**: Safe concurrent access using Arc<Mutex<>> patterns

use crate::{
    blocks::{create_block, Block},
    config::{BlockConfig, Config},
    value::from_yaml_value,
    error::PlcError,
    signal::SignalBus,
    value::Value,
};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};
use tokio::{
    sync::{Mutex, RwLock},
    task::JoinHandle,
    time::{interval, sleep, MissedTickBehavior},
};
use tracing::{debug, error, info, warn, span, Level};

#[cfg(feature = "enhanced-monitoring")]
use crate::metrics::EngineMetrics;

#[cfg(feature = "parallel-execution")]
mod parallel_executor;

#[cfg(feature = "realtime")]
use libc::{sched_param, sched_setscheduler, SCHED_FIFO};

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Engine-specific configuration parameters
/// 
/// These settings control the runtime behavior of the engine, separate from
/// the system configuration loaded from YAML files.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineConfig {
    /// Enable enhanced performance monitoring
    pub enhanced_monitoring: bool,
    
    /// Enable automatic error recovery
    pub error_recovery: bool,
    
    /// Maximum consecutive errors before shutdown
    pub max_consecutive_errors: u64,
    
    /// Delay between recovery attempts (ms)
    pub recovery_delay_ms: u64,
    
    /// Enable real-time thread priority (requires realtime feature)
    pub realtime_priority: Option<i32>,
    
    /// CPU affinity mask for the engine thread
    pub cpu_affinity: Option<Vec<usize>>,
    
    /// Enable detailed performance profiling
    pub profiling: bool,

    /// Enable parallel block execution
    pub parallel_execution: bool,

    /// Enable SIMD optimizations
    pub simd_enabled: bool,

    /// Enable memory pooling
    pub use_memory_pools: bool,

    /// Memory pool prewarm size
    pub pool_prewarm_size: usize,

    /// Enable cache optimization
    pub cache_optimized: bool,
    
    /// Watchdog timeout (0 = disabled)
    pub watchdog_timeout_ms: u64,
    
    /// Behavior when scan cycle is missed
    #[serde(skip)]
    pub missed_tick_behavior: MissedTickBehavior,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enhanced_monitoring: false,
            error_recovery: true,
            max_consecutive_errors: 10,
            recovery_delay_ms: 1000,
            realtime_priority: None,
            cpu_affinity: None,
            profiling: false,
            parallel_execution: false,
            simd_enabled: false,
            use_memory_pools: false,
            pool_prewarm_size: 0,
            cache_optimized: false,
            watchdog_timeout_ms: 0,
            missed_tick_behavior: MissedTickBehavior::Burst,
        }
    }
}

impl EngineConfig {
    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            enhanced_monitoring: false,
            error_recovery: true,
            max_consecutive_errors: 5,
            recovery_delay_ms: 100,
            realtime_priority: Some(50),
            cpu_affinity: Some(vec![0]), // Pin to first CPU
            profiling: false,
            parallel_execution: true,
            simd_enabled: true,
            use_memory_pools: true,
            pool_prewarm_size: 10_000,
            cache_optimized: true,
            watchdog_timeout_ms: 0,
            missed_tick_behavior: MissedTickBehavior::Skip,
        }
    }
    
    /// Create a development/debug configuration
    pub fn development() -> Self {
        Self {
            enhanced_monitoring: true,
            error_recovery: false,
            max_consecutive_errors: 1,
            recovery_delay_ms: 5000,
            realtime_priority: None,
            cpu_affinity: None,
            profiling: true,
            parallel_execution: false,
            simd_enabled: false,
            use_memory_pools: false,
            pool_prewarm_size: 0,
            cache_optimized: false,
            watchdog_timeout_ms: 30000,
            missed_tick_behavior: MissedTickBehavior::Burst,
        }
    }
    
    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enhanced_monitoring: true,
            error_recovery: true,
            max_consecutive_errors: 10,
            recovery_delay_ms: 1000,
            realtime_priority: None,
            cpu_affinity: None,
            profiling: false,
            parallel_execution: true,
            simd_enabled: false,
            use_memory_pools: false,
            pool_prewarm_size: 0,
            cache_optimized: false,
            watchdog_timeout_ms: 60000,
            missed_tick_behavior: MissedTickBehavior::Delay,
        }
    }
}

// ============================================================================
// ENGINE STATE AND STATISTICS
// ============================================================================

/// Engine lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineState {
    /// Engine is initialized but not started
    Stopped,
    /// Engine is starting up
    Starting,
    /// Engine is running normally
    Running,
    /// Engine is shutting down
    Stopping,
    /// Engine encountered a fatal error
    Error,
    /// Engine is in recovery mode
    Recovering,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState::Stopped
    }
}

/// Detailed engine performance statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineStats {
    /// Total scan cycles completed
    pub scan_count: u64,
    
    /// Total errors encountered
    pub error_count: u64,
    
    /// Errors by block name
    pub block_errors: HashMap<String, u64>,
    
    /// Minimum scan time observed
    pub min_scan_time: Duration,
    
    /// Maximum scan time observed
    pub max_scan_time: Duration,
    
    /// Average scan time (exponential moving average)
    pub avg_scan_time: Duration,
    
    /// Current scan time jitter
    pub jitter: Duration,
    
    /// Maximum jitter observed
    pub max_jitter: Duration,
    
    /// Number of scan overruns
    pub scan_overruns: u64,
    
    /// Engine uptime
    pub uptime: Duration,
    
    /// Current engine state
    pub state: EngineState,
    
    /// Last error message
    pub last_error: Option<String>,
    
    /// Timestamp of last successful scan
    pub last_scan_time: Option<SystemTime>,
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Per-block execution times
    pub block_execution_times: HashMap<String, Duration>,
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Signal update frequencies
    pub signal_update_rates: HashMap<String, f64>,
}

// ============================================================================
// MAIN ENGINE STRUCTURE
// ============================================================================

/// The PETRA automation engine
/// 
/// This is the core execution engine that manages the scan cycle, executes blocks,
/// and coordinates all system components. It provides real-time execution with precise
/// timing control, comprehensive error handling, and detailed performance monitoring.
/// 
/// # Thread Safety
/// 
/// The engine is designed to be safely shared across threads using Arc. All
/// internal state is protected by appropriate synchronization primitives:
/// 
/// - `blocks`: Protected by `Arc<Mutex<>>` for safe async access
/// - `stats`: Protected by `Arc<RwLock<>>` for concurrent reads with exclusive writes
/// - Atomic counters for lock-free metrics updates
pub struct Engine {
    // ========================================================================
    // CORE COMPONENTS
    // ========================================================================
    
    /// Signal bus for data flow between components
    bus: SignalBus,
    
    /// Logic blocks for automation processing
    /// 
    /// Protected by Arc<Mutex<>> to allow safe async access for hot-reload
    /// and runtime block management.
    blocks: Arc<Mutex<Vec<Box<dyn Block>>>>,
    
    /// System configuration
    config: Config,
    
    /// Engine-specific configuration
    engine_config: EngineConfig,
    
    // ========================================================================
    // RUNTIME STATE
    // ========================================================================
    
    /// Engine running state (atomic for lock-free access)
    running: Arc<AtomicBool>,
    
    /// Current engine state for lifecycle management
    state: Arc<RwLock<EngineState>>,
    
    /// Total scan cycles completed
    scan_count: Arc<AtomicU64>,
    
    /// Total errors encountered
    error_count: Arc<AtomicU64>,
    
    /// Consecutive errors without successful scan
    consecutive_errors: Arc<AtomicU64>,
    
    /// Engine start timestamp
    start_time: Instant,
    
    /// Target scan cycle duration
    target_scan_time: Duration,
    
    // ========================================================================
    // PERFORMANCE MONITORING
    // ========================================================================
    
    /// Detailed performance statistics
    stats: Arc<RwLock<EngineStats>>,
    
    /// Last scan cycle start time for jitter calculation
    last_scan_start: Arc<RwLock<Instant>>,
    
    /// Exponential moving average alpha for scan time
    ema_alpha: f64,
    
    // ========================================================================
    // OPTIONAL FEATURES
    // ========================================================================
    
    #[cfg(feature = "enhanced-monitoring")]
    /// Enhanced monitoring metrics collector
    metrics: Arc<EngineMetrics>,
    
    /// Watchdog timer handle
    watchdog_handle: Option<JoinHandle<()>>,
    
    /// Last watchdog ping time
    last_watchdog_ping: Arc<RwLock<Instant>>,

    #[cfg(feature = "parallel-execution")]
    parallel_executor: Option<Arc<parallel_executor::ParallelExecutor>>,
}

// ============================================================================
// ENGINE CONSTRUCTION AND INITIALIZATION
// ============================================================================

impl Engine {
    /// Create a new engine with default configuration
    /// 
    /// # Arguments
    /// 
    /// * `config` - System configuration loaded from YAML
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::{Config, Engine};
    /// 
    /// let config = Config::from_file("petra.yaml")?;
    /// let engine = Engine::new(config)?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn new(config: Config) -> Result<Self, PlcError> {
        Self::new_with_config(config, EngineConfig::default())
    }
    
    /// Create a new engine with an existing signal bus
    /// 
    /// This constructor is useful for integration tests where you want to
    /// pre-populate the signal bus or share it between components.
    /// 
    /// # Arguments
    /// 
    /// * `config` - System configuration
    /// * `bus` - Pre-initialized signal bus
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::{Config, Engine, SignalBus, Value};
    /// 
    /// let bus = SignalBus::new();
    /// bus.set("test_signal", Value::Float(42.0))?;
    /// 
    /// let config = Config::from_file("petra.yaml")?;
    /// let engine = Engine::new_with_bus(config, bus)?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn new_with_bus(config: Config, bus: SignalBus) -> Result<Self, PlcError> {
        let engine_config = EngineConfig::default();
        Self::new_with_bus_and_config(config, bus, engine_config)
    }
    
    /// Create a new engine with custom engine configuration
    /// 
    /// # Arguments
    /// 
    /// * `config` - System configuration
    /// * `engine_config` - Engine-specific configuration parameters
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::{Config, Engine, EngineConfig};
    /// 
    /// let config = Config::from_file("petra.yaml")?;
    /// let engine_config = EngineConfig::high_performance();
    /// let engine = Engine::new_with_config(config, engine_config)?;
    /// # Ok::<(), petra::PlcError>(())
    /// ```
    pub fn new_with_config(config: Config, engine_config: EngineConfig) -> Result<Self, PlcError> {
        let bus = SignalBus::new();
        Self::new_with_bus_and_config(config, bus, engine_config)
    }
    
    /// Create a new engine with both custom bus and engine configuration
    /// 
    /// This is the most flexible constructor, allowing full control over
    /// all engine components.
    /// 
    /// # Arguments
    /// 
    /// * `config` - System configuration
    /// * `bus` - Pre-initialized signal bus
    /// * `engine_config` - Engine-specific configuration
    pub fn new_with_bus_and_config(
        config: Config,
        bus: SignalBus,
        engine_config: EngineConfig,
    ) -> Result<Self, PlcError> {
        let _span = span!(Level::INFO, "engine_init").entered();
        info!("Initializing PETRA engine v{}", env!("CARGO_PKG_VERSION"));
        
        // Validate configuration
        config.validate()?;
        
        // Initialize signals from configuration
        Self::initialize_signals(&bus, &config)?;
        
        // Create and initialize blocks
        let blocks = Self::create_blocks(&config)?;

        #[cfg(feature = "parallel-execution")]
        let parallel_executor = if engine_config.parallel_execution {
            Some(Arc::new(parallel_executor::ParallelExecutor::new(&blocks)))
        } else {
            None
        };
        
        // Calculate EMA alpha based on scan time
        let ema_alpha = 2.0 / (10.0 + 1.0); // 10-period EMA
        
        let engine = Self {
            bus,
            blocks: Arc::new(Mutex::new(blocks)),
            target_scan_time: Duration::from_millis(config.scan_time_ms),
            config,
            engine_config,
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(EngineState::Stopped)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            consecutive_errors: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            stats: Arc::new(RwLock::new(EngineStats {
                min_scan_time: Duration::MAX,
                max_scan_time: Duration::ZERO,
                ..Default::default()
            })),
            last_scan_start: Arc::new(RwLock::new(Instant::now())),
            ema_alpha,
            #[cfg(feature = "enhanced-monitoring")]
            metrics: Arc::new(EngineMetrics::new()),
            watchdog_handle: None,
            last_watchdog_ping: Arc::new(RwLock::new(Instant::now())),
            #[cfg(feature = "parallel-execution")]
            parallel_executor,
        };
        
        info!(
            "Engine initialized: {} signals, {} blocks, scan_time={}ms", 
            engine.config.signals.len(),
            engine.blocks.try_lock().map_or(0, |b| b.len()),
            engine.config.scan_time_ms
        );
        
        Ok(engine)
    }
    
    /// Initialize signals in the signal bus from configuration
    fn initialize_signals(bus: &SignalBus, config: &Config) -> Result<(), PlcError> {
        let _span = span!(Level::DEBUG, "init_signals").entered();
        
        for signal_config in &config.signals {
            // Convert initial value from configuration
            let value = if let Some(initial_yaml) = &signal_config.initial {
                from_yaml_value(initial_yaml.clone())
                    .map_err(|e| PlcError::Config(format!(
                        "Signal '{}' initial value conversion failed: {}",
                        signal_config.name, e
                    )))?
            } else {
                // Use default value for signal type
                match signal_config.signal_type.as_str() {
                    "bool" => Value::Bool(false),
                    "int" | "integer" => Value::Integer(0),
                    "float" => Value::Float(0.0),
                    #[cfg(feature = "extended-types")]
                    "string" => Value::String(String::new()),
                    _ => return Err(PlcError::Config(format!(
                        "Unknown signal type: {}", signal_config.signal_type
                    ))),
                }
            };
            
            // Set initial value in signal bus
            bus.set(&signal_config.name, value)?;
            
            debug!("Initialized signal '{}' with type '{}'", 
                signal_config.name, signal_config.signal_type);
        }
        
        debug!("Initialized {} signals", config.signals.len());
        Ok(())
    }
    
    /// Create and initialize all blocks from configuration
    fn create_blocks(config: &Config) -> Result<Vec<Box<dyn Block>>, PlcError> {
        let _span = span!(Level::DEBUG, "create_blocks").entered();
        
        let mut blocks = Vec::with_capacity(config.blocks.len());
        
        // Sort blocks by priority (higher priority = earlier execution)
        let mut sorted_configs = config.blocks.clone();
        sorted_configs.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        for block_config in sorted_configs {
            if !block_config.enabled {
                debug!("Skipping disabled block '{}'", block_config.name);
                continue;
            }
            
            match create_block(&block_config) {
                Ok(block) => {
                    debug!("Created block '{}' of type '{}' with priority {}", 
                        block_config.name, block_config.block_type, block_config.priority);
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
        
        debug!("Created {} blocks", blocks.len());
        Ok(blocks)
    }
    
    /// Start the watchdog timer
    fn start_watchdog(&mut self, timeout: Duration) {
        let running = Arc::clone(&self.running);
        let last_ping = Arc::clone(&self.last_watchdog_ping);
        
        let handle = tokio::spawn(async move {
            let mut interval = interval(timeout / 4); // Check 4x per timeout period
            
            while running.load(Ordering::Relaxed) {
                interval.tick().await;
                
                let last_ping_time = *last_ping.read().await;
                if last_ping_time.elapsed() > timeout {
                    error!("Engine watchdog timeout! Last ping: {:?} ago", last_ping_time.elapsed());
                    // In a production system, this might trigger a restart
                    break;
                }
            }
        });
        
        self.watchdog_handle = Some(handle);
        debug!("Started engine watchdog with timeout: {:?}", timeout);
    }
    
    /// Update watchdog ping timestamp
    async fn ping_watchdog(&self) {
        if self.watchdog_handle.is_some() {
            *self.last_watchdog_ping.write().await = Instant::now();
        }
    }
}

// ============================================================================
// ENGINE EXECUTION CONTROL
// ============================================================================

impl Engine {
    /// Start the engine main loop
    /// 
    /// This method starts the engine's main execution loop and runs until stopped.
    /// The loop executes scan cycles at the configured interval, handling errors
    /// gracefully and maintaining precise timing.
    /// 
    /// # Real-Time Configuration
    /// 
    /// When the "realtime" feature is enabled and real-time priority is configured,
    /// the engine will attempt to set the thread priority and CPU affinity for
    /// deterministic performance.
    /// 
    /// # Error Handling
    /// 
    /// The engine uses a multi-level error handling strategy:
    /// 1. Individual block errors are isolated and logged
    /// 2. Scan cycle errors increment the consecutive error counter
    /// 3. Too many consecutive errors trigger a shutdown or recovery
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use petra::{Config, Engine};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let config = Config::from_file("petra.yaml")?;
    ///     let engine = Engine::new(config)?;
    ///     
    ///     // Run until stopped (Ctrl+C)
    ///     engine.run().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(&mut self) -> Result<(), PlcError> {
        let _span = span!(Level::INFO, "engine_run").entered();
        
        // Configure real-time scheduling if requested
        #[cfg(feature = "realtime")]
        if let Some(priority) = self.engine_config.realtime_priority {
            self.set_realtime_priority(priority)?;
        }
        
        // Set CPU affinity if configured
        #[cfg(feature = "realtime")]
        if let Some(cpus) = &self.engine_config.cpu_affinity {
            self.set_cpu_affinity(cpus)?;
        }
        
        // Start watchdog if configured
        if self.engine_config.watchdog_timeout_ms > 0 {
            self.start_watchdog(Duration::from_millis(self.engine_config.watchdog_timeout_ms));
        }
        
        // Update state
        *self.state.write().await = EngineState::Starting;
        self.running.store(true, Ordering::Release);
        
        info!("Engine starting with scan time: {:?}", self.target_scan_time);
        
        // Create scan interval with configured behavior
        let mut scan_interval = interval(self.target_scan_time);
        scan_interval.set_missed_tick_behavior(self.engine_config.missed_tick_behavior);
        
        // Update state to running
        *self.state.write().await = EngineState::Running;
        self.start_time = Instant::now();
        
        // Main scan loop
        while self.running.load(Ordering::Acquire) {
            scan_interval.tick().await;
            
            match self.execute_scan_cycle().await {
                Ok(()) => {
                    // Reset consecutive error counter on success
                    self.consecutive_errors.store(0, Ordering::Relaxed);
                    
                    // Ping watchdog
                    self.ping_watchdog().await;
                }
                Err(e) => {
                    error!("Scan cycle error: {}", e);
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    
                    let consecutive = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    // Update stats with error
                    let mut stats = self.stats.write().await;
                    stats.error_count += 1;
                    stats.last_error = Some(e.to_string());
                    drop(stats);
                    
                    // Check if we should shutdown or recover
                    if consecutive >= self.engine_config.max_consecutive_errors {
                        error!(
                            "Maximum consecutive errors ({}) reached, initiating shutdown",
                            self.engine_config.max_consecutive_errors
                        );
                        
                        if self.engine_config.error_recovery {
                            *self.state.write().await = EngineState::Recovering;
                            warn!("Attempting recovery in {}ms", self.engine_config.recovery_delay_ms);
                            sleep(Duration::from_millis(self.engine_config.recovery_delay_ms)).await;
                            
                            // Reset error counter for recovery attempt
                            self.consecutive_errors.store(0, Ordering::Relaxed);
                            *self.state.write().await = EngineState::Running;
                        } else {
                            *self.state.write().await = EngineState::Error;
                            return Err(PlcError::Runtime(format!(
                                "Engine shutdown due to {} consecutive errors", consecutive
                            )));
                        }
                    }
                }
            }
        }
        
        // Graceful shutdown
        *self.state.write().await = EngineState::Stopping;
        info!("Engine shutting down gracefully");
        
        // Stop watchdog
        if let Some(handle) = self.watchdog_handle.take() {
            handle.abort();
        }
        
        *self.state.write().await = EngineState::Stopped;
        Ok(())
    }
    
    /// Execute a single scan cycle
    /// 
    /// This method executes all enabled blocks in priority order, updating
    /// performance statistics and handling errors for individual blocks.
    pub async fn execute_scan_cycle(&self) -> Result<(), PlcError> {
        let scan_start = Instant::now();
        let _span = span!(Level::TRACE, "scan_cycle", scan = %self.scan_count()).entered();
        
        // Execute all blocks
        #[cfg(feature = "parallel-execution")]
        if let Some(executor) = &self.parallel_executor {
            executor.execute_parallel(Arc::clone(&self.blocks), &self.bus).await?;
        } else {
            let mut blocks = self.blocks.lock().await;
            let mut block_errors = Vec::new();

            for block in blocks.iter_mut() {
                let block_start = Instant::now();

                match block.execute(&self.bus) {
                    Ok(()) => {
                        let block_elapsed = block_start.elapsed();

                        #[cfg(feature = "enhanced-monitoring")]
                        {
                            let mut stats = self.stats.write().await;
                            stats.block_execution_times.insert(
                                block.name().to_string(),
                                block_elapsed,
                            );
                        }

                        if block_elapsed > self.target_scan_time / 10 {
                            warn!(
                                "Slow block '{}' took {:?} (>10% of scan time)",
                                block.name(),
                                block_elapsed
                            );
                        }
                    }
                    Err(e) => {
                        error!("Block '{}' execution failed: {}", block.name(), e);
                        block_errors.push((block.name().to_string(), e));

                        let mut stats = self.stats.write().await;
                        *stats.block_errors.entry(block.name().to_string()).or_insert(0) += 1;
                    }
                }
            }

            drop(blocks);

            if !block_errors.is_empty() {
                let error_msg = block_errors
                    .iter()
                    .map(|(name, err)| format!("{}: {}", name, err))
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(PlcError::Runtime(format!(
                    "Block execution errors: {}",
                    error_msg
                )));
            }
        }

        #[cfg(not(feature = "parallel-execution"))]
        {
            let mut blocks = self.blocks.lock().await;
            let mut block_errors = Vec::new();

            for block in blocks.iter_mut() {
                let block_start = Instant::now();

                match block.execute(&self.bus) {
                    Ok(()) => {
                        let block_elapsed = block_start.elapsed();

                        #[cfg(feature = "enhanced-monitoring")]
                        {
                            let mut stats = self.stats.write().await;
                            stats.block_execution_times.insert(
                                block.name().to_string(),
                                block_elapsed,
                            );
                        }

                        if block_elapsed > self.target_scan_time / 10 {
                            warn!(
                                "Slow block '{}' took {:?} (>10% of scan time)",
                                block.name(),
                                block_elapsed
                            );
                        }
                    }
                    Err(e) => {
                        error!("Block '{}' execution failed: {}", block.name(), e);
                        block_errors.push((block.name().to_string(), e));

                        let mut stats = self.stats.write().await;
                        *stats.block_errors.entry(block.name().to_string()).or_insert(0) += 1;
                    }
                }
            }

            drop(blocks);

            if !block_errors.is_empty() {
                let error_msg = block_errors
                    .iter()
                    .map(|(name, err)| format!("{}: {}", name, err))
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(PlcError::Runtime(format!(
                    "Block execution errors: {}",
                    error_msg
                )));
            }
        }
        
        // Update scan statistics
        let scan_elapsed = scan_start.elapsed();
        self.update_statistics(scan_elapsed).await;
        
        // Increment scan counter
        self.scan_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Update performance statistics after a scan cycle
    async fn update_statistics(&self, scan_elapsed: Duration) {
        let mut stats = self.stats.write().await;
        
        // Update scan count
        stats.scan_count = self.scan_count.load(Ordering::Relaxed);
        
        // Update min/max scan times
        if scan_elapsed < stats.min_scan_time {
            stats.min_scan_time = scan_elapsed;
        }
        if scan_elapsed > stats.max_scan_time {
            stats.max_scan_time = scan_elapsed;
        }
        
        // Calculate exponential moving average
        if stats.avg_scan_time == Duration::ZERO {
            stats.avg_scan_time = scan_elapsed;
        } else {
            let avg_nanos = stats.avg_scan_time.as_nanos() as f64;
            let scan_nanos = scan_elapsed.as_nanos() as f64;
            let new_avg_nanos = self.ema_alpha * scan_nanos + (1.0 - self.ema_alpha) * avg_nanos;
            stats.avg_scan_time = Duration::from_nanos(new_avg_nanos as u64);
        }
        
        // Calculate jitter
        let last_start = *self.last_scan_start.read().await;
        let actual_interval = last_start.elapsed();
        let jitter = if actual_interval > self.target_scan_time {
            actual_interval - self.target_scan_time
        } else {
            self.target_scan_time - actual_interval
        };
        
        stats.jitter = jitter;
        if jitter > stats.max_jitter {
            stats.max_jitter = jitter;
        }
        
        // Check for scan overrun
        if scan_elapsed > self.target_scan_time {
            stats.scan_overruns += 1;
            warn!(
                "Scan overrun: {:?} > {:?} (target)",
                scan_elapsed, self.target_scan_time
            );
        }
        
        // Update timestamps
        stats.uptime = self.start_time.elapsed();
        stats.last_scan_time = Some(SystemTime::now());
        
        // Update last scan start for next jitter calculation
        drop(stats);
        *self.last_scan_start.write().await = Instant::now();
        
        // Log performance warnings
        if jitter > self.target_scan_time / 5 {
            warn!("High jitter detected: {:?} (>20% of scan time)", jitter);
        }
    }
    
    /// Stop the engine gracefully
    /// 
    /// This method signals the engine to stop and waits for the current
    /// scan cycle to complete before returning.
    pub async fn stop(&self) {
        info!("Engine stop requested");
        self.running.store(false, Ordering::Release);
        
        // Give the engine time to complete current scan
        sleep(self.target_scan_time * 2).await;
    }
    
    /// Force an immediate engine stop
    /// 
    /// This method immediately stops the engine without waiting for the
    /// current scan cycle to complete. Use with caution.
    pub fn force_stop(&self) {
        warn!("Engine force stop requested");
        self.running.store(false, Ordering::Release);
        *self.state.try_write().unwrap() = EngineState::Stopped;
    }
}

// ============================================================================
// REAL-TIME CONFIGURATION (Linux only)
// ============================================================================

#[cfg(feature = "realtime")]
impl Engine {
    /// Set real-time thread priority (requires root/CAP_SYS_NICE)
    fn set_realtime_priority(&self, priority: i32) -> Result<(), PlcError> {
        unsafe {
            let param = sched_param {
                sched_priority: priority,
            };
            
            let result = sched_setscheduler(0, SCHED_FIFO, &param);
            if result != 0 {
                warn!("Failed to set real-time priority: {}", std::io::Error::last_os_error());
                // Don't fail, just warn
            } else {
                info!("Set real-time priority to {}", priority);
            }
        }
        Ok(())
    }
    
    /// Set CPU affinity mask
    fn set_cpu_affinity(&self, cpus: &[usize]) -> Result<(), PlcError> {
        use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
        use std::mem;
        
        unsafe {
            let mut cpu_set: cpu_set_t = mem::zeroed();
            CPU_ZERO(&mut cpu_set);
            
            for &cpu in cpus {
                CPU_SET(cpu, &mut cpu_set);
            }
            
            let result = sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set);
            if result != 0 {
                warn!("Failed to set CPU affinity: {}", std::io::Error::last_os_error());
            } else {
                info!("Set CPU affinity to {:?}", cpus);
            }
        }
        Ok(())
    }
}

// ============================================================================
// ENGINE STATE QUERIES
// ============================================================================

impl Engine {
    /// Check if the engine is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }
    
    /// Get the current engine state
    pub async fn state(&self) -> EngineState {
        *self.state.read().await
    }
    
    /// Get total scan cycles completed
    pub fn scan_count(&self) -> u64 {
        self.scan_count.load(Ordering::Relaxed)
    }
    
    /// Get total errors encountered
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
    
    /// Get current consecutive error count
    pub fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }
    
    /// Get engine uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get a copy of current statistics
    pub async fn stats(&self) -> EngineStats {
        self.stats.read().await.clone()
    }
    
    /// Get basic performance metrics as a formatted string
    pub async fn performance_summary(&self) -> String {
        let stats = self.stats().await;
        
        format!(
            "Engine Performance Summary:\n\
             • Scans: {} (overruns: {})\n\
             • Timing: avg={:?}, min={:?}, max={:?}\n\
             • Jitter: current={:?}, max={:?}\n\
             • Errors: {} blocks, {} total\n\
             • Uptime: {:?}",
            stats.scan_count,
            stats.scan_overruns,
            stats.avg_scan_time,
            stats.min_scan_time,
            stats.max_scan_time,
            stats.jitter,
            stats.max_jitter,
            stats.block_errors.len(),
            self.error_count(),
            stats.uptime
        )
    }
}

// ============================================================================
// EXTERNAL ACCESS AND UTILITIES
// ============================================================================

impl Engine {
    /// Get reference to signal bus
    /// 
    /// Provides access to the signal bus for external components like
    /// protocol drivers, web interfaces, and monitoring systems.
    pub fn signal_bus(&self) -> &SignalBus {
        &self.bus
    }
    
    /// Alias for signal_bus() for backwards compatibility
    pub fn get_bus(&self) -> &SignalBus {
        self.signal_bus()
    }
    
    /// Reset all blocks to their initial state
    /// 
    /// This method resets all blocks and clears performance statistics.
    /// Useful for testing and system recovery scenarios.
    pub async fn reset_blocks(&self) -> Result<(), PlcError> {
        let _span = span!(Level::INFO, "reset_blocks").entered();
        
        let mut blocks = self.blocks.lock().await;
        for block in blocks.iter_mut() {
            if let Err(e) = block.reset() {
                error!("Failed to reset block '{}': {}", block.name(), e);
                return Err(e);
            }
        }
        
        // Reset statistics
        self.scan_count.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
        self.consecutive_errors.store(0, Ordering::Relaxed);
        
        let mut stats = self.stats.write().await;
        *stats = EngineStats {
            min_scan_time: Duration::MAX,
            max_scan_time: Duration::ZERO,
            state: *self.state.read().await,
            ..Default::default()
        };
        
        info!("Reset {} blocks and cleared statistics", blocks.len());
        Ok(())
    }
    
    /// Reload configuration and blocks without stopping
    /// 
    /// This method allows hot-reloading of configuration and blocks
    /// while the engine continues running. New blocks are created and
    /// swapped atomically.
    #[cfg(feature = "hot-reload")]
    pub async fn reload_config(&self, new_config: Config) -> Result<(), PlcError> {
        let _span = span!(Level::INFO, "reload_config").entered();
        info!("Starting configuration reload");
        
        // Validate new configuration
        new_config.validate()?;
        
        // Create new blocks
        let new_blocks = Self::create_blocks(&new_config)?;
        
        // Atomically swap blocks
        let mut blocks = self.blocks.lock().await;
        *blocks = new_blocks;
        
        info!("Configuration reloaded successfully");
        Ok(())
    }
    
    /// Add a new block to the running engine
    /// 
    /// This method allows adding new blocks dynamically without restart.
    pub async fn add_block(&self, block: Box<dyn Block>) -> Result<(), PlcError> {
        let mut blocks = self.blocks.lock().await;
        let block_name = block.name().to_string();
        
        // Check for duplicate names
        if blocks.iter().any(|b| b.name() == block_name) {
            return Err(PlcError::Config(format!(
                "Block with name '{}' already exists", block_name
            )));
        }
        
        blocks.push(block);
        info!("Added new block '{}'", block_name);
        Ok(())
    }
    
    /// Remove a block from the running engine
    /// 
    /// This method allows removing blocks dynamically without restart.
    pub async fn remove_block(&self, block_name: &str) -> Result<(), PlcError> {
        let mut blocks = self.blocks.lock().await;
        let initial_count = blocks.len();
        
        blocks.retain(|b| b.name() != block_name);
        
        if blocks.len() == initial_count {
            return Err(PlcError::Config(format!(
                "Block '{}' not found", block_name
            )));
        }
        
        info!("Removed block '{}'", block_name);
        Ok(())
    }
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SignalConfig, BlockConfig};
    use std::collections::HashMap;
    
    fn create_test_config() -> Config {
        Config {
            scan_time_ms: 100,
            max_scan_jitter_ms: 10,
            error_recovery: true,
            max_consecutive_errors: 5,
            restart_delay_ms: 1000,
            
            signals: vec![
                SignalConfig {
                    name: "test_signal".to_string(),
                    signal_type: "bool".to_string(),
                    initial: Some(serde_yaml::Value::Bool(false)),
                    description: Some("Test signal".to_string()),
                    unit: None,
                    min: None,
                    max: None,
                    access: None,
                    #[cfg(feature = "extended-types")]
                    metadata: HashMap::new(),
                },
                SignalConfig {
                    name: "output_signal".to_string(),
                    signal_type: "bool".to_string(),
                    initial: None,
                    description: Some("Output signal".to_string()),
                    unit: None,
                    min: None,
                    max: None,
                    access: None,
                    #[cfg(feature = "extended-types")]
                    metadata: HashMap::new(),
                },
            ],
            
            blocks: vec![
                BlockConfig {
                    name: "test_block".to_string(),
                    block_type: "NOT".to_string(),
                    inputs: {
                        let mut inputs = HashMap::new();
                        inputs.insert("input".to_string(), "test_signal".to_string());
                        inputs
                    },
                    outputs: {
                        let mut outputs = HashMap::new();
                        outputs.insert("output".to_string(), "output_signal".to_string());
                        outputs
                    },
                    parameters: HashMap::new(),
                    priority: 0,
                    enabled: true,
                    description: Some("Test NOT block".to_string()),
                    category: Some("Test".to_string()),
                    tags: vec!["test".to_string()],
                    #[cfg(feature = "circuit-breaker")]
                    circuit_breaker: None,
                    #[cfg(feature = "enhanced-monitoring")]
                    enhanced_monitoring: false,
                    metadata: HashMap::new(),
                },
            ],
            
            protocols: None,
            version: "1.0".to_string(),
            description: Some("Test configuration".to_string()),
            author: Some("Test".to_string()),
            created: None,
            modified: None,
            metadata: HashMap::new(),
            
            #[cfg(feature = "mqtt")]
            mqtt: None,
            #[cfg(feature = "security")]
            security: None,
            #[cfg(feature = "history")]
            history: None,
            #[cfg(feature = "alarms")]
            alarms: None,
            #[cfg(feature = "web")]
            web: None,
            #[cfg(feature = "advanced-storage")]
            storage: None,
            #[cfg(feature = "s7-support")]
            s7: None,
            #[cfg(feature = "modbus-support")]
            modbus: None,
            #[cfg(feature = "opcua-support")]
            opcua: None,
            #[cfg(feature = "twilio")]
            twilio: None,
            #[cfg(feature = "email")]
            email: None,
        }
    }
    
    #[test]
    fn test_engine_creation() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        assert!(!engine.is_running());
        assert_eq!(engine.scan_count(), 0);
        assert_eq!(engine.error_count(), 0);
    }
    
    #[test]
    fn test_engine_with_bus() {
        let config = create_test_config();
        let bus = SignalBus::new();
        
        // Pre-populate bus
        bus.set("pre_existing", Value::Float(42.0)).unwrap();
        
        let engine = Engine::new_with_bus(config, bus).unwrap();
        
        // Verify pre-existing signal
        let value = engine.signal_bus().get("pre_existing").unwrap();
        assert_eq!(value, Some(Value::Float(42.0)));
        
        // Verify configured signals were initialized
        let test_signal = engine.signal_bus().get("test_signal").unwrap();
        assert_eq!(test_signal, Some(Value::Bool(false)));
    }
    
    #[test]
    fn test_engine_config_variants() {
        let config = create_test_config();
        
        // Test high performance config
        let hp_config = EngineConfig::high_performance();
        let engine = Engine::new_with_config(config.clone(), hp_config).unwrap();
        assert!(!engine.engine_config.enhanced_monitoring);
        assert_eq!(engine.engine_config.realtime_priority, Some(50));
        
        // Test development config
        let dev_config = EngineConfig::development();
        let engine = Engine::new_with_config(config.clone(), dev_config).unwrap();
        assert!(engine.engine_config.enhanced_monitoring);
        assert!(!engine.engine_config.error_recovery);
        
        // Test production config
        let prod_config = EngineConfig::production();
        let engine = Engine::new_with_config(config, prod_config).unwrap();
        assert!(engine.engine_config.enhanced_monitoring);
        assert!(engine.engine_config.error_recovery);
    }
    
    #[tokio::test]
    async fn test_engine_lifecycle() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Initial state
        assert_eq!(engine.state().await, EngineState::Stopped);
        assert!(!engine.is_running());
        
        // Start and immediately stop
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let mut engine = engine_clone;
            engine.run().await
        });
        
        // Give it time to start
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(engine.is_running());
        assert_eq!(engine.state().await, EngineState::Running);
        
        // Stop the engine
        engine.stop().await;
        handle.abort();
        
        assert!(!engine.is_running());
    }
    
    #[tokio::test]
    async fn test_scan_cycle_execution() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Set initial signal value
        engine.signal_bus().set("test_signal", Value::Bool(true)).unwrap();
        
        // Execute one scan cycle
        engine.execute_scan_cycle().await.unwrap();
        
        // Check that NOT block inverted the signal
        let output = engine.signal_bus().get("output_signal").unwrap();
        assert_eq!(output, Some(Value::Bool(false)));
        
        // Verify statistics
        assert_eq!(engine.scan_count(), 1);
        assert_eq!(engine.error_count(), 0);
        
        let stats = engine.stats().await;
        assert_eq!(stats.scan_count, 1);
        assert!(stats.avg_scan_time > Duration::ZERO);
    }
    
    #[tokio::test]
    async fn test_block_management() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Create a simple test block
        use crate::blocks::{Block, BlockConfig};
        
        struct TestBlock {
            name: String,
        }
        
        impl Block for TestBlock {
            fn execute(&mut self, _bus: &SignalBus) -> Result<(), PlcError> {
                Ok(())
            }
            
            fn name(&self) -> &str {
                &self.name
            }
            
            fn block_type(&self) -> &str {
                "TEST"
            }
            
            fn reset(&mut self) -> Result<(), PlcError> {
                Ok(())
            }
        }
        
        // Add a new block
        let new_block = Box::new(TestBlock {
            name: "dynamic_block".to_string(),
        });
        
        engine.add_block(new_block).await.unwrap();
        
        // Try to add duplicate
        let duplicate = Box::new(TestBlock {
            name: "dynamic_block".to_string(),
        });
        
        assert!(engine.add_block(duplicate).await.is_err());
        
        // Remove the block
        engine.remove_block("dynamic_block").await.unwrap();
        
        // Try to remove non-existent block
        assert!(engine.remove_block("non_existent").await.is_err());
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Simulate a block error by setting invalid signal reference
        // This would normally happen if a block references a non-existent signal
        
        // For this test, we'll just verify error counting works
        engine.error_count.fetch_add(1, Ordering::Relaxed);
        engine.consecutive_errors.fetch_add(1, Ordering::Relaxed);
        
        assert_eq!(engine.error_count(), 1);
        assert_eq!(engine.consecutive_errors(), 1);
        
        // Reset blocks should clear error counters
        engine.reset_blocks().await.unwrap();
        assert_eq!(engine.error_count(), 0);
        assert_eq!(engine.consecutive_errors(), 0);
    }
    
    #[test]
    fn test_performance_summary() {
        let config = create_test_config();
        let engine = Engine::new(config).unwrap();
        
        // Get performance summary (should work even with no scans)
        let rt = tokio::runtime::Runtime::new().unwrap();
        let summary = rt.block_on(engine.performance_summary());
        
        assert!(summary.contains("Engine Performance Summary"));
        assert!(summary.contains("Scans: 0"));
        assert!(summary.contains("Errors: 0"));
    }
}

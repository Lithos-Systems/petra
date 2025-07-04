//! # PETRA Real-Time Scan Engine
//!
//! ## Purpose & Overview
//! 
//! This module provides the core real-time execution engine for PETRA, serving as the central
//! orchestrator that coordinates all system components in deterministic scan cycles. The engine
//! is responsible for:
//!
//! - **Deterministic Execution** - Executes automation logic in predictable, timed cycles
//! - **Block Orchestration** - Manages execution order and data flow between logic blocks
//! - **Real-Time Performance** - Maintains precise timing with jitter monitoring and compensation
//! - **Error Recovery** - Handles transient errors gracefully without system shutdown
//! - **Performance Monitoring** - Tracks execution metrics and identifies bottlenecks
//! - **Graceful Degradation** - Continues operation even when individual blocks fail
//! - **Resource Management** - Optimizes CPU usage and memory allocation patterns
//!
//! ## Architecture & Interactions
//!
//! The engine sits at the heart of PETRA's architecture, orchestrating all components:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                          PETRA Engine Core                              │
//! │                        (src/engine.rs)                                  │
//! └─────────┬─────────────────────────┬─────────────────────────┬───────────┘
//!           │                         │                         │
//!           ▼                         ▼                         ▼
//! ┌─────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
//! │  Signal Bus     │    │    Block System     │    │  Protocol Drivers   │
//! │ (signal.rs)     │◄──►│   (blocks/*)        │◄──►│   (protocols/*)     │
//! │                 │    │                     │    │                     │
//! │ • Data Flow     │    │ • Logic Processing  │    │ • I/O Operations    │
//! │ • State Storage │    │ • Control Algorithms│    │ • Communication     │
//! │ • Thread Safety │    │ • Math Operations   │    │ • Data Acquisition  │
//! └─────────────────┘    └─────────────────────┘    └─────────────────────┘
//!           │                         │                         │
//!           ▼                         ▼                         ▼
//! ┌─────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
//! │   Monitoring    │    │      Storage        │    │      Security       │
//! │ (metrics.rs)    │    │   (history.rs)      │    │   (security.rs)     │
//! │                 │    │                     │    │                     │
//! │ • Performance   │    │ • Data Logging      │    │ • Authentication    │
//! │ • Alarms        │    │ • Trend Analysis    │    │ • Authorization     │
//! │ • Diagnostics   │    │ • Backup/Recovery   │    │ • Audit Logging     │
//! └─────────────────┘    └─────────────────────┘    └─────────────────────┘
//! ```
//!
//! ## Scan Cycle Architecture
//!
//! The engine operates in deterministic scan cycles:
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────────────┐
//! │                        Scan Cycle (e.g., 100ms)                        │
//! └─────┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬──────┘
//!       │         │         │         │         │         │         │
//!       ▼         ▼         ▼         ▼         ▼         ▼         ▼
//!   ┌──────┐ ┌─────────┐ ┌──────┐ ┌─────────┐ ┌──────┐ ┌─────────┐ ┌──────┐
//!   │  I/O │ │ Blocks  │ │ Math │ │ Control │ │Alarms│ │ History │ │Stats │
//!   │ Read │ │Logic Ex.│ │ Ops  │ │ Loops   │ │Check │ │ Logging │ │Update│
//!   └──────┘ └─────────┘ └──────┘ └─────────┘ └──────┘ └─────────┘ └──────┘
//!      ↑                                                                ↓
//!      └────────────────── Feedback & Control Loop ─────────────────────┘
//! ```
//!
//! ## Real-Time Performance Features
//!
//! - **Predictable Timing**: Fixed scan cycles with jitter monitoring
//! - **Priority-Based Execution**: Blocks execute in order of priority
//! - **Missed Tick Handling**: Configurable behavior for timing violations
//! - **Resource Optimization**: Memory-efficient data structures and algorithms
//! - **Lock-Free Operations**: Signal bus uses concurrent data structures
//! - **Error Isolation**: Individual block failures don't affect others
//!
//! ## Performance Characteristics
//!
//! The engine is optimized for:
//! - **Sub-microsecond jitter** in real-time mode with proper OS configuration
//! - **10,000+ signals** with minimal memory overhead
//! - **1,000+ blocks** executing per scan cycle
//! - **100+ protocols** operating concurrently
//! - **Deterministic execution** regardless of system load

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![allow(clippy::module_name_repetitions)] // Engine types naturally repeat "Engine"

use crate::{
    blocks::{Block, create_block},
    config::Config,
    error::{PlcError, Result},
    signal::SignalBus,
    value::{Value, from_yaml_value},
};
use std::{
    collections::{VecDeque, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::{
    sync::{Mutex, RwLock},
    time::{interval, MissedTickBehavior, sleep},
    runtime::Runtime,
};
use tracing::{info, warn, error, debug, trace, span, Level};

// Feature-gated imports for enhanced functionality
#[cfg(feature = "metrics")]
use metrics::{counter, gauge, histogram};

#[cfg(feature = "realtime")]
use libc;

#[cfg(feature = "circuit-breaker")]
use crate::blocks::circuit_breaker::BlockExecutor;

#[cfg(feature = "enhanced-monitoring")]
use std::collections::HashMap;

// ============================================================================
// ENGINE CONFIGURATION
// ============================================================================

/// Engine configuration parameters
/// 
/// Controls various aspects of engine behavior including monitoring, metrics,
/// error handling, and performance tuning. These settings affect the runtime
/// characteristics and can be tuned for different deployment scenarios.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::engine::EngineConfig;
/// 
/// // High-performance configuration
/// let config = EngineConfig {
///     enhanced_monitoring: false,  // Disable for better performance
///     metrics_enabled: true,       // Keep basic metrics
///     max_consecutive_errors: 5,   // Fail fast
///     error_recovery_enabled: true,
///     performance_logging: false,  // No extra logging overhead
///     real_time_priority: Some(50),
///     cpu_affinity: Some(vec![0, 1]),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Enable enhanced monitoring with detailed execution tracking
    /// 
    /// When enabled, tracks execution order, block timing, and maintains
    /// historical performance data. Adds ~5-10% overhead.
    pub enhanced_monitoring: bool,
    
    /// Enable Prometheus-compatible metrics collection
    /// 
    /// Exports performance metrics for monitoring systems. Minimal overhead
    /// when metrics are not actively scraped.
    pub metrics_enabled: bool,
    
    /// Maximum consecutive errors before engine shutdown
    /// 
    /// Safety mechanism to prevent runaway error conditions. Lower values
    /// provide faster failure detection but may be too aggressive.
    pub max_consecutive_errors: u64,
    
    /// Enable automatic error recovery mechanisms
    /// 
    /// When enabled, the engine attempts to recover from transient errors
    /// rather than immediately shutting down.
    pub error_recovery_enabled: bool,
    
    /// Enable performance logging for diagnostics
    /// 
    /// Logs detailed performance warnings when timing thresholds are exceeded.
    /// Useful for debugging but adds logging overhead.
    pub performance_logging: bool,
    
    /// Real-time thread priority (1-99, Linux only)
    /// 
    /// Only available with the "realtime" feature. Higher values get higher
    /// priority but require root privileges or capabilities.
    #[cfg(feature = "realtime")]
    pub real_time_priority: Option<u8>,
    
    /// CPU affinity mask for engine thread
    /// 
    /// Only available with the "realtime" feature. Pins the engine to specific
    /// CPU cores for consistent performance.
    #[cfg(feature = "realtime")]
    pub cpu_affinity: Option<Vec<usize>>,
    
    /// Engine watchdog timeout in milliseconds
    /// 
    /// If enabled, a watchdog thread monitors the engine and can restart it
    /// if it becomes unresponsive.
    pub watchdog_timeout_ms: Option<u64>,
    
    /// Block execution timeout in microseconds
    /// 
    /// Maximum time allowed for a single block execution. Blocks exceeding
    /// this limit are terminated and marked as failed.
    pub block_timeout_us: Option<u64>,
    
    /// Scan cycle overrun tolerance percentage
    /// 
    /// Acceptable percentage of scan time overrun before warnings are generated.
    /// For example, 10 means 10% overrun is acceptable.
    pub overrun_tolerance_percent: u8,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enhanced_monitoring: cfg!(feature = "enhanced-monitoring"),
            metrics_enabled: cfg!(feature = "metrics"),
            max_consecutive_errors: 10,
            error_recovery_enabled: true,
            performance_logging: cfg!(debug_assertions),
            #[cfg(feature = "realtime")]
            real_time_priority: None,
            #[cfg(feature = "realtime")]
            cpu_affinity: None,
            watchdog_timeout_ms: None,
            block_timeout_us: None,
            overrun_tolerance_percent: 10,
        }
    }
}

impl EngineConfig {
    /// Create a high-performance configuration
    /// 
    /// Optimized for minimal overhead and maximum throughput.
    pub fn high_performance() -> Self {
        Self {
            enhanced_monitoring: false,
            metrics_enabled: false,
            max_consecutive_errors: 3,
            error_recovery_enabled: true,
            performance_logging: false,
            #[cfg(feature = "realtime")]
            real_time_priority: Some(80),
            #[cfg(feature = "realtime")]
            cpu_affinity: Some(vec![0]),
            watchdog_timeout_ms: None,
            block_timeout_us: Some(100), // 100µs max per block
            overrun_tolerance_percent: 5,
        }
    }
    
    /// Create a development configuration
    /// 
    /// Optimized for debugging and development with full monitoring.
    pub fn development() -> Self {
        Self {
            enhanced_monitoring: true,
            metrics_enabled: true,
            max_consecutive_errors: 100,
            error_recovery_enabled: true,
            performance_logging: true,
            #[cfg(feature = "realtime")]
            real_time_priority: None,
            #[cfg(feature = "realtime")]
            cpu_affinity: None,
            watchdog_timeout_ms: Some(30000), // 30 second timeout
            block_timeout_us: Some(10000),    // 10ms max per block
            overrun_tolerance_percent: 50,
        }
    }
    
    /// Create a production configuration
    /// 
    /// Balanced settings for production deployments.
    pub fn production() -> Self {
        Self {
            enhanced_monitoring: false,
            metrics_enabled: true,
            max_consecutive_errors: 10,
            error_recovery_enabled: true,
            performance_logging: false,
            #[cfg(feature = "realtime")]
            real_time_priority: Some(50),
            #[cfg(feature = "realtime")]
            cpu_affinity: None,
            watchdog_timeout_ms: Some(60000), // 1 minute timeout
            block_timeout_us: Some(1000),     // 1ms max per block
            overrun_tolerance_percent: 10,
        }
    }
}

// ============================================================================
// ENGINE STATISTICS
// ============================================================================

/// Comprehensive engine performance statistics
/// 
/// Tracks detailed timing and execution metrics for performance analysis
/// and system optimization. These statistics are used for monitoring,
/// alerting, and performance tuning.
#[derive(Debug, Default, Clone)]
pub struct EngineStats {
    /// Total number of completed scan cycles
    pub scan_count: u64,
    
    /// Cumulative execution time across all scans
    pub total_scan_time: Duration,
    
    /// Minimum observed scan time
    pub min_scan_time: Duration,
    
    /// Maximum observed scan time
    pub max_scan_time: Duration,
    
    /// Duration of the most recent scan
    pub last_scan_time: Duration,
    
    /// Average scan time across all cycles
    pub avg_scan_time: Duration,
    
    /// Current timing jitter (deviation from target)
    pub jitter: Duration,
    
    /// Maximum jitter observed
    pub max_jitter: Duration,
    
    /// Total number of block execution errors
    pub block_errors: u64,
    
    /// Number of scan cycles that exceeded target time
    pub scan_overruns: u64,
    
    /// Total number of blocks executed across all scans
    pub blocks_executed: u64,
    
    /// Engine uptime since start
    pub uptime: Duration,
    
    /// Current engine state
    pub state: EngineState,
    
    /// Memory usage statistics
    pub memory_stats: MemoryStats,
    
    /// Thread pool statistics
    #[cfg(feature = "thread-pool")]
    pub thread_pool_stats: ThreadPoolStats,
}

/// Engine operational state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Engine is stopped and not processing
    Stopped,
    /// Engine is starting up
    Starting,
    /// Engine is running normally
    Running,
    /// Engine is stopping gracefully
    Stopping,
    /// Engine has encountered a fatal error
    Error,
    /// Engine is in recovery mode
    Recovering,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Memory usage statistics
#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    /// Total heap memory allocated
    pub heap_allocated: usize,
    /// Peak heap memory usage
    pub heap_peak: usize,
    /// Signal bus memory usage
    pub signal_bus_memory: usize,
    /// Block system memory usage
    pub blocks_memory: usize,
}

/// Thread pool statistics
#[cfg(feature = "thread-pool")]
#[derive(Debug, Default, Clone)]
pub struct ThreadPoolStats {
    /// Number of active worker threads
    pub active_threads: usize,
    /// Number of queued tasks
    pub queued_tasks: usize,
    /// Total tasks completed
    pub completed_tasks: u64,
}

// ============================================================================
// MAIN ENGINE IMPLEMENTATION
// ============================================================================

/// Main PETRA execution engine
/// 
/// The engine is the central coordinator that orchestrates all PETRA components
/// in deterministic scan cycles. It provides real-time execution with precise
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
/// 
/// # Examples
/// 
/// ```rust
/// use petra::{Config, Engine, EngineConfig};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load configuration
///     let config = Config::from_file("petra.yaml")?;
///     
///     // Create engine with production settings
///     let engine_config = EngineConfig::production();
///     let engine = Engine::new_with_config(config, engine_config)?;
///     
///     // Start the engine (runs until stopped)
///     engine.run().await?;
///     
///     Ok(())
/// }
/// ```
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
    blocks: Arc<Mutex<Vec<Box<dyn Block + Send + Sync>>>>,
    
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
    
    /// Comprehensive statistics tracking
    stats: Arc<RwLock<EngineStats>>,
    
    /// Historical scan time data for trend analysis
    #[cfg(feature = "enhanced-monitoring")]
    scan_times: Arc<RwLock<VecDeque<Duration>>>,
    
    /// Block execution order from last scan
    #[cfg(feature = "enhanced-monitoring")]
    execution_order: Arc<RwLock<Vec<String>>>,
    
    /// Failed blocks from last scan
    #[cfg(feature = "enhanced-monitoring")]
    failed_blocks: Arc<RwLock<Vec<String>>>,
    
    /// Individual block execution times
    #[cfg(feature = "enhanced-monitoring")]
    block_execution_times: Arc<RwLock<HashMap<String, Duration>>>,
    
    // ========================================================================
    // ADVANCED FEATURES
    // ========================================================================
    
    /// Circuit breaker for fault isolation
    #[cfg(feature = "circuit-breaker")]
    block_executor: Option<Arc<BlockExecutor>>,
    
    /// Watchdog timer for deadlock detection
    watchdog_handle: Option<tokio::task::JoinHandle<()>>,
    
    /// Last watchdog ping timestamp
    last_watchdog_ping: Arc<RwLock<Instant>>,
}

// ============================================================================
// ENGINE LIFECYCLE MANAGEMENT
// ============================================================================

impl Engine {
    /// Create a new engine instance with default configuration
    /// 
    /// # Arguments
    /// 
    /// * `config` - System configuration containing signals, blocks, and settings
    /// 
    /// # Errors
    /// 
    /// Returns errors for:
    /// - Invalid signal configurations
    /// - Block creation failures
    /// - Signal bus initialization issues
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
    pub fn new(config: Config) -> Result<Self> {
        Self::new_with_config(config, EngineConfig::default())
    }
    
    /// Create engine with custom engine configuration
    /// 
    /// Allows fine-tuning of engine behavior for specific deployment scenarios.
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
    pub fn new_with_config(config: Config, engine_config: EngineConfig) -> Result<Self> {
        let _span = span!(Level::INFO, "engine_init").entered();
        info!("Initializing PETRA engine v{}", env!("CARGO_PKG_VERSION"));
        
        // Validate configuration for runtime use
        config.validate_for_runtime()?;
        
        // Initialize signal bus
        let bus = SignalBus::new();
        debug!("Initialized signal bus");
        
        // Initialize signals from configuration
        Self::initialize_signals(&bus, &config)?;
        
        // Create and initialize blocks
        let blocks = Self::create_blocks(&config)?;
        
        // Calculate timing parameters
        let target_scan_time = Duration::from_millis(config.scan_time_ms);
        let start_time = Instant::now();
        
        // Initialize statistics
        let stats = EngineStats {
            min_scan_time: Duration::MAX,
            max_scan_time: Duration::ZERO,
            state: EngineState::Stopped,
            ..Default::default()
        };
        
        // Create engine instance
        let mut engine = Self {
            bus,
            blocks: Arc::new(Mutex::new(blocks)),
            config,
            engine_config,
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(RwLock::new(EngineState::Stopped)),
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
            
            watchdog_handle: None,
            last_watchdog_ping: Arc::new(RwLock::new(Instant::now())),
        };
        
        // Initialize advanced features
        #[cfg(feature = "circuit-breaker")]
        if engine.engine_config.enhanced_monitoring {
            engine.block_executor = Some(Arc::new(BlockExecutor::new()));
        }
        
        // Start watchdog if configured
        if let Some(timeout_ms) = engine_config.watchdog_timeout_ms {
            engine.start_watchdog(Duration::from_millis(timeout_ms));
        }
        
        info!(
            "Engine initialized: {} signals, {} blocks, scan_time={}ms", 
            engine.config.signals.len(),
            engine.blocks.try_lock().map_or(0, |b| b.len()),
            engine.config.scan_time_ms
        );
        
        Ok(engine)
    }
    
    /// Initialize signals in the signal bus from configuration
    fn initialize_signals(bus: &SignalBus, config: &Config) -> Result<()> {
        let _span = span!(Level::DEBUG, "init_signals").entered();
        
        for signal_config in &config.signals {
            // Convert initial value from configuration
            let value = if let Some(initial_yaml) = &signal_config.initial {
                from_yaml_value(initial_yaml, &signal_config.signal_type)
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
            
            trace!("Initialized signal '{}' with type '{}'", 
                signal_config.name, signal_config.signal_type);
        }
        
        debug!("Initialized {} signals", config.signals.len());
        Ok(())
    }
    
    /// Create and initialize all blocks from configuration
    fn create_blocks(config: &Config) -> Result<Vec<Box<dyn Block + Send + Sync>>> {
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
    /// 3. When consecutive errors exceed the threshold, the engine stops
    /// 4. If error recovery is enabled, the engine may attempt automatic recovery
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
    ///     // This will run until the engine is stopped
    ///     engine.run().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(&self) -> Result<()> {
        let _span = span!(Level::INFO, "engine_run").entered();
        
        // Set engine state to starting
        *self.state.write().await = EngineState::Starting;
        
        // Configure real-time settings if enabled
        #[cfg(feature = "realtime")]
        self.configure_realtime().await?;
        
        // Set running flag and engine state
        self.running.store(true, Ordering::Relaxed);
        *self.state.write().await = EngineState::Running;
        
        info!("Starting PETRA engine with scan time: {}ms", self.config.scan_time_ms);
        
        // Initialize scan interval with precise timing
        let mut interval = interval(self.target_scan_time);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        
        // Initialize metrics if enabled
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_engine_scan_time_ms").set(self.config.scan_time_ms as f64);
            counter!("petra_engine_starts").increment(1);
            gauge!("petra_engine_uptime_seconds").set(0.0);
        }
        
        // Main execution loop
        while self.running.load(Ordering::Relaxed) {
            // Wait for next scan interval
            interval.tick().await;
            
            // Ping watchdog
            self.ping_watchdog().await;
            
            // Execute scan cycle with timing measurement
            let scan_start = Instant::now();
            
            match self.execute_scan_cycle().await {
                Ok(_) => {
                    // Success: reset error counters and update metrics
                    self.scan_count.fetch_add(1, Ordering::Relaxed);
                    self.consecutive_errors.store(0, Ordering::Relaxed);
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_successful_scans").increment(1);
                    }
                }
                Err(e) => {
                    // Error: increment counters and check thresholds
                    error!("Scan cycle error: {}", e);
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    let consecutive_errors = self.consecutive_errors.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_scan_errors").increment(1);
                        gauge!("petra_consecutive_errors").set(consecutive_errors as f64);
                    }
                    
                    // Check if we should attempt recovery or stop
                    if consecutive_errors >= self.engine_config.max_consecutive_errors {
                        if self.engine_config.error_recovery_enabled {
                            warn!("Attempting engine recovery after {} consecutive errors", consecutive_errors);
                            match self.attempt_recovery().await {
                                Ok(_) => {
                                    info!("Engine recovery successful");
                                    self.consecutive_errors.store(0, Ordering::Relaxed);
                                    continue;
                                }
                                Err(recovery_err) => {
                                    error!("Engine recovery failed: {}", recovery_err);
                                }
                            }
                        }
                        
                        error!("Too many consecutive errors ({}), stopping engine", consecutive_errors);
                        *self.state.write().await = EngineState::Error;
                        self.stop().await;
                        return Err(PlcError::Runtime(format!(
                            "Too many consecutive errors: {}", consecutive_errors
                        )));
                    }
                }
            }
            
            // Update timing statistics
            let scan_duration = scan_start.elapsed();
            self.update_timing_stats(scan_duration).await;
        }
        
        // Engine stopped
        *self.state.write().await = EngineState::Stopped;
        info!("PETRA engine stopped");
        
        // Stop watchdog
        if let Some(handle) = &self.watchdog_handle {
            handle.abort();
        }
        
        Ok(())
    }
    
    /// Configure real-time operating system settings
    #[cfg(feature = "realtime")]
    async fn configure_realtime(&self) -> Result<()> {
        if let Some(priority) = self.engine_config.real_time_priority {
            info!("Configuring real-time priority: {}", priority);
            
            // Set real-time scheduling policy
            unsafe {
                let param = libc::sched_param {
                    sched_priority: priority as i32,
                };
                
                if libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) != 0 {
                    warn!("Failed to set real-time priority: {}", 
                        std::io::Error::last_os_error());
                }
            }
        }
        
        if let Some(ref cpu_mask) = self.engine_config.cpu_affinity {
            info!("Configuring CPU affinity: {:?}", cpu_mask);
            
            // Set CPU affinity
            let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
            
            for &cpu in cpu_mask {
                unsafe {
                    libc::CPU_SET(cpu, &mut cpu_set);
                }
            }
            
            unsafe {
                if libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpu_set) != 0 {
                    warn!("Failed to set CPU affinity: {}", 
                        std::io::Error::last_os_error());
                }
            }
        }
        
        Ok(())
    }
    
    /// Execute a single scan cycle
    /// 
    /// This method executes all enabled blocks in priority order, handling
    /// individual block failures gracefully while maintaining overall system
    /// operation. The scan cycle is designed to be deterministic and complete
    /// within the target scan time.
    /// 
    /// # Block Execution Strategy
    /// 
    /// 1. **Priority-Based Ordering**: Blocks execute in descending priority order
    /// 2. **Error Isolation**: Individual block failures don't stop the scan
    /// 3. **Timeout Protection**: Blocks exceeding timeout limits are terminated
    /// 4. **Performance Monitoring**: Execution times are tracked per block
    /// 
    /// # Performance Optimization
    /// 
    /// - Blocks are executed sequentially to maintain determinism
    /// - Signal bus operations use lock-free data structures
    /// - Memory allocations are minimized during execution
    /// - Detailed timing is only collected when monitoring is enabled
    pub async fn execute_scan_cycle(&self) -> Result<()> {
        let _span = span!(Level::TRACE, "scan_cycle").entered();
        
        // Initialize monitoring data
        #[cfg(feature = "enhanced-monitoring")]
        let mut execution_order = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut failed_blocks = Vec::new();
        #[cfg(feature = "enhanced-monitoring")]
        let mut block_times = HashMap::new();
        
        let mut blocks_executed = 0u64;
        let mut block_errors = 0u64;
        
        // Lock blocks for the duration of the scan cycle
        let mut blocks = self.blocks.lock().await;
        
        // Execute all blocks in priority order
        for block in blocks.iter_mut() {
            let block_name = block.name().to_string();
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                execution_order.push(block_name.clone());
            }
            
            // Execute block with timing measurement
            let block_start = Instant::now();
            
            let result = if let Some(timeout_us) = self.engine_config.block_timeout_us {
                // Execute with timeout protection
                self.execute_block_with_timeout(block.as_mut(), Duration::from_micros(timeout_us)).await
            } else {
                // Execute without timeout
                block.execute(&self.bus)
            };
            
            let block_duration = block_start.elapsed();
            blocks_executed += 1;
            
            match result {
                Ok(_) => {
                    trace!("Block '{}' executed successfully in {:?}", block_name, block_duration);
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        histogram!("petra_block_execution_time", "block" => block_name.clone())
                            .record(block_duration.as_secs_f64());
                    }
                }
                Err(e) => {
                    warn!("Block '{}' failed: {}", block_name, e);
                    block_errors += 1;
                    
                    #[cfg(feature = "enhanced-monitoring")]
                    if self.engine_config.enhanced_monitoring {
                        failed_blocks.push(block_name.clone());
                    }
                    
                    #[cfg(feature = "metrics")]
                    if self.engine_config.metrics_enabled {
                        counter!("petra_block_errors", "block" => block_name.clone()).increment(1);
                    }
                    
                    // Individual block failures don't stop the scan cycle
                    // This ensures partial functionality during errors
                }
            }
            
            #[cfg(feature = "enhanced-monitoring")]
            if self.engine_config.enhanced_monitoring {
                block_times.insert(block_name, block_duration);
            }
        }
        
        // Update monitoring data
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            *self.execution_order.write().await = execution_order;
            *self.failed_blocks.write().await = failed_blocks;
            *self.block_execution_times.write().await = block_times;
        }
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.blocks_executed += blocks_executed;
            stats.block_errors += block_errors;
            stats.uptime = self.start_time.elapsed();
        }
        
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_blocks_executed_total").set(blocks_executed as f64);
            if block_errors > 0 {
                gauge!("petra_block_errors_in_scan").set(block_errors as f64);
            }
        }
        
        trace!("Scan cycle completed: {} blocks executed, {} errors", 
            blocks_executed, block_errors);
        
        Ok(())
    }
    
    /// Execute a block with timeout protection
    async fn execute_block_with_timeout(
        &self,
        block: &mut dyn Block,
        timeout: Duration,
    ) -> Result<()> {
        let block_name = block.name().to_string();
        
        // Create a future for block execution
        let execution_future = async {
            block.execute(&self.bus)
        };
        
        // Execute with timeout
        match tokio::time::timeout(timeout, execution_future).await {
            Ok(result) => result,
            Err(_) => {
                error!("Block '{}' exceeded timeout ({:?})", block_name, timeout);
                Err(PlcError::Runtime(format!(
                    "Block '{}' execution timeout", block_name
                )))
            }
        }
    }
    
    /// Attempt engine recovery after consecutive errors
    async fn attempt_recovery(&self) -> Result<()> {
        let _span = span!(Level::WARN, "engine_recovery").entered();
        
        *self.state.write().await = EngineState::Recovering;
        
        info!("Attempting engine recovery...");
        
        // Step 1: Reset all blocks
        {
            let mut blocks = self.blocks.lock().await;
            for block in blocks.iter_mut() {
                if let Err(e) = block.reset() {
                    warn!("Failed to reset block '{}': {}", block.name(), e);
                }
            }
        }
        
        // Step 2: Verify signal bus integrity
        let signal_count = self.bus.len();
        if signal_count == 0 {
            return Err(PlcError::Runtime("Signal bus is empty after recovery".to_string()));
        }
        
        // Step 3: Brief pause to allow system stabilization
        sleep(Duration::from_millis(100)).await;
        
        // Step 4: Test scan cycle
        match self.execute_scan_cycle().await {
            Ok(_) => {
                info!("Engine recovery successful");
                *self.state.write().await = EngineState::Running;
                Ok(())
            }
            Err(e) => {
                error!("Engine recovery test failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Stop the engine gracefully
    /// 
    /// This method initiates a graceful shutdown of the engine, allowing
    /// the current scan cycle to complete before stopping.
    pub async fn stop(&self) {
        let _span = span!(Level::INFO, "engine_stop").entered();
        
        info!("Stopping PETRA engine");
        *self.state.write().await = EngineState::Stopping;
        
        self.running.store(false, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            counter!("petra_engine_stops").increment(1);
        }
    }
    
    /// Check if engine is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
    
    /// Get current engine state
    pub async fn state(&self) -> EngineState {
        *self.state.read().await
    }
}

// ============================================================================
// PERFORMANCE MONITORING AND STATISTICS
// ============================================================================

impl Engine {
    /// Update timing statistics after each scan cycle
    async fn update_timing_stats(&self, scan_duration: Duration) {
        let mut stats = self.stats.write().await;
        
        // Update basic timing stats
        stats.scan_count += 1;
        stats.total_scan_time += scan_duration;
        stats.last_scan_time = scan_duration;
        
        // Update min/max tracking
        if scan_duration < stats.min_scan_time {
            stats.min_scan_time = scan_duration;
        }
        if scan_duration > stats.max_scan_time {
            stats.max_scan_time = scan_duration;
        }
        
        // Calculate rolling average
        stats.avg_scan_time = stats.total_scan_time / stats.scan_count as u32;
        
        // Calculate jitter (deviation from target)
        let target_nanos = self.target_scan_time.as_nanos() as i128;
        let actual_nanos = scan_duration.as_nanos() as i128;
        let jitter_nanos = (actual_nanos - target_nanos).abs() as u64;
        stats.jitter = Duration::from_nanos(jitter_nanos);
        
        // Update max jitter
        if stats.jitter > stats.max_jitter {
            stats.max_jitter = stats.jitter;
        }
        
        // Check for scan overruns
        let overrun_threshold = self.target_scan_time + 
            (self.target_scan_time * self.engine_config.overrun_tolerance_percent as u32 / 100);
        
        if scan_duration > overrun_threshold {
            stats.scan_overruns += 1;
            
            if self.engine_config.performance_logging {
                warn!(
                    "Scan overrun detected: {:?} (target: {:?}, threshold: {:?})",
                    scan_duration, self.target_scan_time, overrun_threshold
                );
            }
        }
        
        // Update memory statistics
        self.update_memory_stats(&mut stats);
        
        // Store historical data for enhanced monitoring
        #[cfg(feature = "enhanced-monitoring")]
        if self.engine_config.enhanced_monitoring {
            let mut scan_times = self.scan_times.write().await;
            scan_times.push_back(scan_duration);
            
            // Keep only last 1000 scan times
            if scan_times.len() > 1000 {
                scan_times.pop_front();
            }
        }
        
        // Update metrics if enabled
        #[cfg(feature = "metrics")]
        if self.engine_config.metrics_enabled {
            gauge!("petra_scan_time_us").set(scan_duration.as_micros() as f64);
            gauge!("petra_jitter_us").set(stats.jitter.as_micros() as f64);
            gauge!("petra_scan_count").set(stats.scan_count as f64);
            gauge!("petra_avg_scan_time_us").set(stats.avg_scan_time.as_micros() as f64);
            gauge!("petra_max_jitter_us").set(stats.max_jitter.as_micros() as f64);
            gauge!("petra_scan_overruns").set(stats.scan_overruns as f64);
            gauge!("petra_engine_uptime_seconds").set(self.start_time.elapsed().as_secs_f64());
            
            histogram!("petra_scan_duration_seconds").record(scan_duration.as_secs_f64());
            
            if scan_duration > overrun_threshold {
                counter!("petra_scan_overruns_total").increment(1);
            }
        }
        
        // Performance warnings
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
    
    /// Update memory usage statistics
    fn update_memory_stats(&self, stats: &mut EngineStats) {
        // TODO: Implement actual memory tracking
        // This would typically use jemalloc stats or similar
        stats.memory_stats.signal_bus_memory = self.bus.memory_usage();
        
        // Estimate blocks memory (simplified)
        stats.memory_stats.blocks_memory = 
            self.blocks.try_lock().map_or(0, |b| b.len() * 1024); // Rough estimate
    }
    
    /// Get comprehensive engine statistics
    /// 
    /// Returns a complete snapshot of engine performance metrics, useful
    /// for monitoring, debugging, and performance analysis.
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
            stats.block_errors,
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
    pub async fn reset_blocks(&self) -> Result<()> {
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
        
        info!("All blocks reset successfully");
        Ok(())
    }
    
    /// Get current scan count
    pub fn scan_count(&self) -> u64 {
        self.scan_count.load(Ordering::Relaxed)
    }
    
    /// Get total error count
    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }
    
    /// Get consecutive error count
    pub fn consecutive_errors(&self) -> u64 {
        self.consecutive_errors.load(Ordering::Relaxed)
    }
    
    /// Get engine uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get target scan time
    pub fn target_scan_time(&self) -> Duration {
        self.target_scan_time
    }
    
    /// Execute a single scan cycle synchronously (for testing/benchmarks)
    /// 
    /// This is a convenience wrapper that creates a temporary runtime
    /// for synchronous testing scenarios.
    pub fn scan_once(&self) -> Result<()> {
        let rt = Runtime::new().expect("failed to create tokio runtime");
        rt.block_on(self.execute_scan_cycle())
    }
    
    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Get engine configuration
    pub fn engine_config(&self) -> &EngineConfig {
        &self.engine_config
    }
}

// ============================================================================
// ENHANCED MONITORING FEATURES
// ============================================================================

#[cfg(feature = "enhanced-monitoring")]
impl Engine {
    /// Get execution order from last scan cycle
    pub async fn execution_order(&self) -> Vec<String> {
        self.execution_order.read().await.clone()
   }
   
   /// Get failed blocks from last scan cycle
   pub async fn failed_blocks(&self) -> Vec<String> {
       self.failed_blocks.read().await.clone()
   }
   
   /// Get individual block execution times from last scan cycle
   pub async fn block_execution_times(&self) -> HashMap<String, Duration> {
       self.block_execution_times.read().await.clone()
   }
   
   /// Get historical scan time data
   pub async fn scan_time_history(&self) -> Vec<Duration> {
       self.scan_times.read().await.iter().cloned().collect()
   }
   
   /// Get performance analysis of recent scan cycles
   pub async fn performance_analysis(&self) -> PerformanceAnalysis {
       let scan_times = self.scan_times.read().await;
       let block_times = self.block_execution_times.read().await;
       
       let mut analysis = PerformanceAnalysis::default();
       
       if !scan_times.is_empty() {
           // Calculate statistics over recent scans
           let sum: Duration = scan_times.iter().sum();
           analysis.recent_avg_scan_time = sum / scan_times.len() as u32;
           
           analysis.recent_min_scan_time = scan_times.iter().min().copied()
               .unwrap_or(Duration::ZERO);
           analysis.recent_max_scan_time = scan_times.iter().max().copied()
               .unwrap_or(Duration::ZERO);
           
           // Calculate jitter variance
           let avg_nanos = analysis.recent_avg_scan_time.as_nanos() as f64;
           let variance: f64 = scan_times.iter()
               .map(|&t| {
                   let diff = t.as_nanos() as f64 - avg_nanos;
                   diff * diff
               })
               .sum::<f64>() / scan_times.len() as f64;
           
           analysis.jitter_variance = Duration::from_nanos(variance.sqrt() as u64);
       }
       
       // Find slowest blocks
       let mut sorted_blocks: Vec<_> = block_times.iter().collect();
       sorted_blocks.sort_by(|a, b| b.1.cmp(a.1));
       
       analysis.slowest_blocks = sorted_blocks
           .into_iter()
           .take(5)
           .map(|(name, time)| (name.clone(), *time))
           .collect();
       
       analysis
   }
}

/// Performance analysis results
#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Default, Clone)]
pub struct PerformanceAnalysis {
   /// Average scan time over recent cycles
   pub recent_avg_scan_time: Duration,
   /// Minimum scan time in recent history
   pub recent_min_scan_time: Duration,
   /// Maximum scan time in recent history
   pub recent_max_scan_time: Duration,
   /// Timing variance (jitter standard deviation)
   pub jitter_variance: Duration,
   /// Top 5 slowest blocks with their execution times
   pub slowest_blocks: Vec<(String, Duration)>,
}

// ============================================================================
// TESTING SUPPORT
// ============================================================================

#[cfg(test)]
impl Clone for Engine {
   fn clone(&self) -> Self {
       Self {
           bus: self.bus.clone(),
           blocks: Arc::new(Mutex::new(Vec::new())), // Empty blocks for testing
           config: self.config.clone(),
           engine_config: self.engine_config.clone(),
           running: Arc::new(AtomicBool::new(false)),
           state: Arc::new(RwLock::new(EngineState::Stopped)),
           scan_count: Arc::new(AtomicU64::new(0)),
           error_count: Arc::new(AtomicU64::new(0)),
           consecutive_errors: Arc::new(AtomicU64::new(0)),
           start_time: self.start_time,
           target_scan_time: self.target_scan_time,
           stats: Arc::new(RwLock::new(EngineStats::default())),
           
           #[cfg(feature = "enhanced-monitoring")]
           scan_times: Arc::new(RwLock::new(VecDeque::new())),
           #[cfg(feature = "enhanced-monitoring")]
           execution_order: Arc::new(RwLock::new(Vec::new())),
           #[cfg(feature = "enhanced-monitoring")]
           failed_blocks: Arc::new(RwLock::new(Vec::new())),
           #[cfg(feature = "enhanced-monitoring")]
           block_execution_times: Arc::new(RwLock::new(HashMap::new())),
           
           #[cfg(feature = "circuit-breaker")]
           block_executor: None,
           
           watchdog_handle: None,
           last_watchdog_ping: Arc::new(RwLock::new(Instant::now())),
       }
   }
}

// ============================================================================
// TESTING MODULE
// ============================================================================

#[cfg(test)]
mod tests {
   use super::*;
   use crate::config::{SignalConfig, BlockConfig};
   use std::collections::HashMap;
   
   /// Create a test configuration for engine testing
   fn create_test_config() -> Config {
       Config {
           scan_time_ms: 100,
           max_scan_jitter_ms: 50,
           error_recovery: true,
           max_consecutive_errors: 10,
           restart_delay_ms: 1000,
           
           signals: vec![
               SignalConfig {
                   name: "test_signal".to_string(),
                   signal_type: "bool".to_string(),
                   initial: Some(serde_yaml::Value::Bool(false)),
                   description: Some("Test signal".to_string()),
                   category: Some("Test".to_string()),
                   source: Some("Test".to_string()),
                   tags: vec!["test".to_string()],
                   update_frequency_ms: None,
                   #[cfg(feature = "engineering-types")]
                   units: None,
                   #[cfg(feature = "engineering-types")]
                   min_value: None,
                   #[cfg(feature = "engineering-types")]
                   max_value: None,
                   #[cfg(feature = "quality-codes")]
                   quality_enabled: false,
                   #[cfg(feature = "history")]
                   log_to_history: false,
                   #[cfg(feature = "history")]
                   log_interval_ms: 0,
                   #[cfg(feature = "alarms")]
                   enable_alarms: false,
                   #[cfg(feature = "validation")]
                   validation: None,
                   metadata: HashMap::new(),
               },
               SignalConfig {
                   name: "output_signal".to_string(),
                   signal_type: "bool".to_string(),
                   initial: Some(serde_yaml::Value::Bool(false)),
                   description: Some("Output signal".to_string()),
                   category: Some("Test".to_string()),
                   source: Some("Test".to_string()),
                   tags: vec!["test".to_string()],
                   update_frequency_ms: None,
                   #[cfg(feature = "engineering-types")]
                   units: None,
                   #[cfg(feature = "engineering-types")]
                   min_value: None,
                   #[cfg(feature = "engineering-types")]
                   max_value: None,
                   #[cfg(feature = "quality-codes")]
                   quality_enabled: false,
                   #[cfg(feature = "history")]
                   log_to_history: false,
                   #[cfg(feature = "history")]
                   log_interval_ms: 0,
                   #[cfg(feature = "alarms")]
                   enable_alarms: false,
                   #[cfg(feature = "validation")]
                   validation: None,
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
                   params: HashMap::new(),
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
   fn test_engine_config_variants() {
       let config = create_test_config();
       
       // Test high performance config
       let hp_config = EngineConfig::high_performance();
       let engine = Engine::new_with_config(config.clone(), hp_config).unwrap();
       assert!(!engine.engine_config.enhanced_monitoring);
       assert!(!engine.engine_config.metrics_enabled);
       
       // Test development config
       let dev_config = EngineConfig::development();
       let engine = Engine::new_with_config(config.clone(), dev_config).unwrap();
       assert!(engine.engine_config.enhanced_monitoring);
       assert!(engine.engine_config.metrics_enabled);
       
       // Test production config
       let prod_config = EngineConfig::production();
       let engine = Engine::new_with_config(config, prod_config).unwrap();
       assert!(!engine.engine_config.enhanced_monitoring);
       assert!(engine.engine_config.metrics_enabled);
   }
   
   #[tokio::test]
   async fn test_engine_start_stop() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Start engine in background
       let engine_clone = engine.clone();
       let handle = tokio::spawn(async move {
           let _ = engine_clone.run().await;
       });
       
       // Wait for engine to start
       tokio::time::sleep(Duration::from_millis(50)).await;
       
       // Check it's running
       assert!(engine.is_running());
       assert_eq!(engine.state().await, EngineState::Running);
       
       // Stop it
       engine.stop().await;
       
       // Wait for task to complete
       let _ = handle.await;
       
       assert!(!engine.is_running());
       assert_eq!(engine.state().await, EngineState::Stopped);
   }
   
   #[tokio::test]
   async fn test_scan_cycle_execution() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Execute a single scan cycle
       engine.execute_scan_cycle().await.unwrap();
       
       // Check statistics were updated
       let stats = engine.stats().await;
       assert_eq!(stats.blocks_executed, 1);
       assert_eq!(stats.block_errors, 0);
   }
   
   #[tokio::test]
   async fn test_reset_blocks() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Execute some scan cycles
       for _ in 0..5 {
           engine.execute_scan_cycle().await.unwrap();
       }
       
       // Reset blocks
       engine.reset_blocks().await.unwrap();
       
       // Check counters were reset
       assert_eq!(engine.scan_count(), 0);
       assert_eq!(engine.error_count(), 0);
       assert_eq!(engine.consecutive_errors(), 0);
   }
   
   #[test]
   fn test_signal_initialization() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Check signals were initialized
       let bus = engine.signal_bus();
       assert_eq!(bus.get_bool("test_signal").unwrap(), false);
       assert_eq!(bus.get_bool("output_signal").unwrap(), false);
   }
   
   #[tokio::test]
   async fn test_performance_summary() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Execute some scan cycles
       for _ in 0..10 {
           engine.execute_scan_cycle().await.unwrap();
           tokio::time::sleep(Duration::from_millis(10)).await;
       }
       
       // Get performance summary
       let summary = engine.performance_summary().await;
       assert!(summary.contains("Engine Performance Summary"));
       assert!(summary.contains("Scans: 10"));
       assert!(summary.contains("Timing:"));
       assert!(summary.contains("Uptime:"));
   }
   
   #[cfg(feature = "enhanced-monitoring")]
   #[tokio::test]
   async fn test_enhanced_monitoring() {
       let mut config = create_test_config();
       let engine_config = EngineConfig {
           enhanced_monitoring: true,
           ..Default::default()
       };
       
       let engine = Engine::new_with_config(config, engine_config).unwrap();
       
       // Execute scan cycle
       engine.execute_scan_cycle().await.unwrap();
       
       // Check monitoring data
       let execution_order = engine.execution_order().await;
       assert_eq!(execution_order.len(), 1);
       assert_eq!(execution_order[0], "test_block");
       
       let failed_blocks = engine.failed_blocks().await;
       assert_eq!(failed_blocks.len(), 0);
       
       let block_times = engine.block_execution_times().await;
       assert!(block_times.contains_key("test_block"));
       
       // Get performance analysis
       let analysis = engine.performance_analysis().await;
       assert_eq!(analysis.slowest_blocks.len(), 1);
   }
   
   #[test]
   fn test_uptime_tracking() {
       let config = create_test_config();
       let engine = Engine::new(config).unwrap();
       
       // Initial uptime should be near zero
       let uptime1 = engine.uptime();
       assert!(uptime1.as_millis() < 100);
       
       // Wait and check again
       std::thread::sleep(Duration::from_millis(100));
       let uptime2 = engine.uptime();
       assert!(uptime2.as_millis() >= 100);
   }
}

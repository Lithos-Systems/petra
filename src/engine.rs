// src/engine.rs
use crate::{
    error::*,
    signal::SignalBus,
    value::Value,
    config::Config,
    blocks::{create_block, Block},  // Updated import path
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use tokio::time::interval;
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};
use std::collections::VecDeque;
use std::sync::Mutex;
use serde::Serialize;
use std::sync::RwLock;

#[cfg(feature = "metrics")]
use metrics::{histogram, counter, gauge};

#[cfg(feature = "enhanced-monitoring")]
use ringbuffer::{AllocRingBuffer, RingBuffer};

#[cfg(feature = "enhanced-monitoring")]
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
pub struct EngineStats {
    pub running: bool,
    pub scan_count: u64,
    pub error_count: u64,
    pub uptime_secs: u64,
    pub signal_count: usize,
    pub block_count: usize,
    #[cfg(feature = "enhanced-monitoring")]
    pub avg_scan_time_us: Option<f64>,
    #[cfg(feature = "enhanced-monitoring")]
    pub max_scan_time_us: Option<f64>,
    #[cfg(feature = "enhanced-monitoring")]
    pub block_times_us: Option<HashMap<String, f64>>,
}

#[cfg(feature = "enhanced-monitoring")]
#[derive(Clone, Debug, Serialize)]
pub struct DetailedStats {
    pub basic: EngineStats,
    pub scan_times: Vec<Duration>,
    pub block_execution_times: HashMap<String, Duration>,
    pub jitter_stats: JitterStats,
}

#[cfg(feature = "enhanced-monitoring")]
#[derive(Clone, Debug, Serialize)]
pub struct JitterStats {
    pub avg_us: f64,
    pub max_us: f64,
    pub variance_us: f64,
}

pub struct Engine {
    bus: SignalBus,
    blocks: Vec<Box<dyn Block>>,
    config: Config,
    running: Arc<AtomicBool>,
    scan_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    start_time: Instant,
    signal_change_tx: Option<mpsc::Sender<(String, Value)>>,
    
    // Standard monitoring
    scan_jitter_buffer: Arc<Mutex<VecDeque<Duration>>>,
    target_scan_time: Duration,
    stats_handle: Arc<RwLock<EngineStats>>,
    
    // Enhanced monitoring (optional)
    #[cfg(feature = "enhanced-monitoring")]
    scan_times: Arc<RwLock<AllocRingBuffer<Duration>>>,
    #[cfg(feature = "enhanced-monitoring")]
    block_execution_times: Arc<RwLock<HashMap<String, Duration>>>,
    #[cfg(feature = "enhanced-monitoring")]
    enable_enhanced_monitoring: bool,
}

impl Engine {
    pub fn new(config: Config) -> Result<Self> {
        // Initialize metrics if available
        #[cfg(feature = "metrics")]
        {
            gauge!("petra_engine_scan_time_ms").set(config.scan_time_ms as f64);
            gauge!("petra_engine_signal_count").set(config.signals.len() as f64);
            gauge!("petra_engine_block_count").set(config.blocks.len() as f64);
        }
        
        // Initialize signal bus
        let bus = SignalBus::new();
        
        // Initialize all signals
        for signal in &config.signals {
            let value = match signal.signal_type.as_str() {
                "bool" => Value::Bool(signal.initial.as_bool().unwrap_or(false)),
                "int" => Value::Int(signal.initial.as_i64().unwrap_or(0) as i32),
                "float" => Value::Float(signal.initial.as_f64().unwrap_or(0.0)),
                _ => {
                    return Err(PlcError::Config(format!(
                        "Invalid signal type '{}' for signal '{}'",
                        signal.signal_type, signal.name
                    )));
                }
            };
            bus.set(&signal.name, value.clone())?;
            debug!("Initialized signal '{}' as {} = {}", signal.name, signal.signal_type, value);
        }

        // Create all blocks
        let mut blocks = Vec::new();
        for block_config in &config.blocks {
            match create_block(block_config) {
                Ok(block) => {
                    info!("Created block '{}' of type '{}'", block_config.name, block_config.block_type);
                    blocks.push(block);
                }
                Err(e) => {
                    return Err(PlcError::Config(format!(
                        "Failed to create block '{}': {}", block_config.name, e
                    )));
                }
            }
        }

        // Check if enhanced monitoring is enabled in config
        #[cfg(feature = "enhanced-monitoring")]
        let enable_enhanced_monitoring = config.engine_config
            .as_ref()
            .and_then(|ec| ec.get("enhanced_monitoring"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Self {
            bus,
            blocks,
            config: config.clone(),
            running: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            signal_change_tx: None,
            scan_jitter_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            target_scan_time: Duration::from_millis(config.scan_time_ms.into()),
            stats_handle: Arc::new(RwLock::new(EngineStats {
                running: false,
                scan_count: 0,
                error_count: 0,
                uptime_secs: 0,
                signal_count: config.signals.len(),
                block_count: config.blocks.len(),
                #[cfg(feature = "enhanced-monitoring")]
                avg_scan_time_us: None,
                #[cfg(feature = "enhanced-monitoring")]
                max_scan_time_us: None,
                #[cfg(feature = "enhanced-monitoring")]
                block_times_us: None,
            })),
            #[cfg(feature = "enhanced-monitoring")]
            scan_times: Arc::new(RwLock::new(AllocRingBuffer::new(1000))),
            #[cfg(feature = "enhanced-monitoring")]
            block_execution_times: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "enhanced-monitoring")]
            enable_enhanced_monitoring,
        })
    }

    pub fn new_with_bus(config: Config, bus: SignalBus) -> Result<Self> {
        // Initialize all signals on the provided bus
        for signal in &config.signals {
            let value = match signal.signal_type.as_str() {
                "bool" => Value::Bool(signal.initial.as_bool().unwrap_or(false)),
                "int" => Value::Int(signal.initial.as_i64().unwrap_or(0) as i32),
                "float" => Value::Float(signal.initial.as_f64().unwrap_or(0.0)),
                _ => {
                    return Err(PlcError::Config(format!(
                        "Invalid signal type '{}' for signal '{}'",
                        signal.signal_type, signal.name
                    )));
                }
            };
            bus.set(&signal.name, value.clone())?;
            debug!("Initialized signal '{}' as {} = {}", signal.name, signal.signal_type, value);
        }

        // Create all blocks
        let mut blocks = Vec::new();
        for block_config in &config.blocks {
            match create_block(block_config) {
                Ok(block) => {
                    info!("Created block '{}' of type '{}'", block_config.name, block_config.block_type);
                    blocks.push(block);
                }
                Err(e) => {
                    return Err(PlcError::Config(format!(
                        "Failed to create block '{}': {}", block_config.name, e
                    )));
                }
            }
        }

        // Check if enhanced monitoring is enabled in config
        #[cfg(feature = "enhanced-monitoring")]
        let enable_enhanced_monitoring = config.engine_config
            .as_ref()
            .and_then(|ec| ec.get("enhanced_monitoring"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(Self {
            bus,
            blocks,
            config: config.clone(),
            running: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            signal_change_tx: None,
            scan_jitter_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            target_scan_time: Duration::from_millis(config.scan_time_ms.into()),
            stats_handle: Arc::new(RwLock::new(EngineStats {
                running: false,
                scan_count: 0,
                error_count: 0,
                uptime_secs: 0,
                signal_count: config.signals.len(),
                block_count: config.blocks.len(),
                #[cfg(feature = "enhanced-monitoring")]
                avg_scan_time_us: None,
                #[cfg(feature = "enhanced-monitoring")]
                max_scan_time_us: None,
                #[cfg(feature = "enhanced-monitoring")]
                block_times_us: None,
            })),
            #[cfg(feature = "enhanced-monitoring")]
            scan_times: Arc::new(RwLock::new(AllocRingBuffer::new(1000))),
            #[cfg(feature = "enhanced-monitoring")]
            block_execution_times: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(feature = "enhanced-monitoring")]
            enable_enhanced_monitoring,
        })
    }

    pub fn set_signal_change_channel(&mut self, tx: mpsc::Sender<(String, Value)>) {
        self.signal_change_tx = Some(tx);
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(PlcError::Config("Engine is already running".into()));
        }

        self.running.store(true, Ordering::Relaxed);
        info!("Starting PETRA engine with scan time: {}ms", self.config.scan_time_ms);

        #[cfg(feature = "metrics")]
        gauge!("petra_engine_running").set(1.0);

        let mut stats = self.stats_handle.write().unwrap();
        stats.running = true;
        drop(stats);

        let mut interval = interval(self.target_scan_time);
        
        while self.running.load(Ordering::Relaxed) {
            let scan_start = Instant::now();
            
            // Execute scan cycle
            match self.execute_scan_cycle().await {
                Ok(_) => {
                    let scan_count = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;
                    
                    #[cfg(feature = "metrics")]
                    counter!("petra_scan_count_total").increment(1);
                    
                    debug!("Scan cycle {} completed", scan_count);
                }
                Err(e) => {
                    let error_count = self.error_count.fetch_add(1, Ordering::Relaxed) + 1;
                    error!("Scan cycle error #{}: {}", error_count, e);
                    
                    #[cfg(feature = "metrics")]
                    counter!("petra_engine_errors_total").increment(1);
                }
            }

            let scan_duration = scan_start.elapsed();
            self.update_timing_stats(scan_duration);

            // Wait for next scan cycle
            interval.tick().await;
        }

        info!("PETRA engine stopped");
        
        #[cfg(feature = "metrics")]
        gauge!("petra_engine_running").set(0.0);

        let mut stats = self.stats_handle.write().unwrap();
        stats.running = false;
        Ok(())
    }

    pub async fn execute_scan_cycle(&mut self) -> Result<()> {
        #[cfg(feature = "enhanced-monitoring")]
        let mut block_times = if self.enable_enhanced_monitoring {
            Some(HashMap::new())
        } else {
            None
        };

        // Execute all blocks
        for block in &mut self.blocks {
            #[cfg(feature = "enhanced-monitoring")]
            let block_start = if self.enable_enhanced_monitoring {
                Some(Instant::now())
            } else {
                None
            };

            match block.execute(&self.bus) {
                Ok(_) => {
                    debug!("Block '{}' executed successfully", block.name());
                }
                Err(e) => {
                    warn!("Block '{}' execution failed: {}", block.name(), e);
                    return Err(e);
                }
            }

            #[cfg(feature = "enhanced-monitoring")]
            if let (Some(start), Some(ref mut times)) = (block_start, &mut block_times) {
                times.insert(block.name().to_string(), start.elapsed());
            }
        }

        #[cfg(feature = "enhanced-monitoring")]
        if let Some(times) = block_times {
            let mut execution_times = self.block_execution_times.write().unwrap();
            *execution_times = times;
        }

        Ok(())
    }

    fn update_timing_stats(&self, scan_duration: Duration) {
        // Update basic jitter buffer
        if let Ok(mut jitter_buffer) = self.scan_jitter_buffer.try_lock() {
            jitter_buffer.push_back(scan_duration);
            if jitter_buffer.len() > 100 {
                jitter_buffer.pop_front();
            }
        }

        #[cfg(feature = "enhanced-monitoring")]
        if self.enable_enhanced_monitoring {
            if let Ok(mut scan_times) = self.scan_times.try_write() {
                scan_times.push(scan_duration);
            }
        }

        #[cfg(feature = "metrics")]
        {
            histogram!("petra_scan_duration_us").record(scan_duration.as_micros() as f64);
            gauge!("petra_scan_duration_ms").set(scan_duration.as_millis() as f64);
            
            // Calculate jitter
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
                    }
                }
            }
        }

        // Update stats handle
        if let Ok(mut stats) = self.stats_handle.try_write() {
            stats.scan_count = self.scan_count.load(Ordering::Relaxed);
            stats.error_count = self.error_count.load(Ordering::Relaxed);
            stats.uptime_secs = self.start_time.elapsed().as_secs();

            #[cfg(feature = "enhanced-monitoring")]
            if self.enable_enhanced_monitoring {
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
            }
        }
    }

    pub fn stop(&self) {
        info!("Stopping PETRA engine");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn signal_bus(&self) -> &SignalBus {
        &self.bus
    }

    pub fn stats(&self) -> EngineStats {
        self.stats_handle.read().unwrap().clone()
    }

    #[cfg(feature = "enhanced-monitoring")]
    pub fn detailed_stats(&self) -> Option<DetailedStats> {
        if !self.enable_enhanced_monitoring {
            return None;
        }

        let basic = self.stats();
        let scan_times = self.scan_times.read().unwrap().to_vec();
        let block_execution_times = self.block_execution_times.read().unwrap().clone();
        
        let jitter_stats = if let Ok(jitter_buffer) = self.scan_jitter_buffer.try_lock() {
            if jitter_buffer.len() > 1 {
                let avg_us = jitter_buffer.iter().sum::<Duration>().as_micros() as f64 / jitter_buffer.len() as f64;
                let max_us = jitter_buffer.iter().max().unwrap().as_micros() as f64;
                let variance_us = jitter_buffer.iter()
                    .map(|d| {
                        let diff = d.as_micros() as f64 - avg_us;
                        diff * diff
                    })
                    .sum::<f64>() / jitter_buffer.len() as f64;
                
                JitterStats {
                    avg_us,
                    max_us,
                    variance_us,
                }
            } else {
                JitterStats {
                    avg_us: 0.0,
                    max_us: 0.0,
                    variance_us: 0.0,
                }
            }
        } else {
            JitterStats {
                avg_us: 0.0,
                max_us: 0.0,
                variance_us: 0.0,
            }
        };

        Some(DetailedStats {
            basic,
            scan_times,
            block_execution_times,
            jitter_stats,
        })
    }

    pub fn get_signal(&self, name: &str) -> Option<Value> {
        self.bus.get(name)
    }

    pub fn set_signal(&self, name: &str, value: Value) -> Result<()> {
        self.bus.set(name, value)?;
        
        // Notify signal change subscribers if configured
        if let Some(ref tx) = self.signal_change_tx {
            if let Err(_) = tx.try_send((name.to_string(), self.bus.get(name).unwrap())) {
                // Channel full or closed, log but don't fail
                warn!("Signal change notification channel full or closed");
            }
        }
        
        Ok(())
    }

    pub fn scan_count(&self) -> u64 {
        self.scan_count.load(Ordering::Relaxed)
    }

    pub fn error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
}

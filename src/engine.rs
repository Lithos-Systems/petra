use crate::{error::*, signal::SignalBus, block::*, config::Config, value::Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use tokio::time::interval;
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};
use std::collections::VecDeque;
use std::sync::Mutex;
use serde::Serialize;
use parking_lot::RwLock;

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
            target_scan_time: Duration::from_millis(config.scan_time_ms),
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
            target_scan_time: Duration::from_millis(config.scan_time_ms),
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
        info!("Engine starting with {}ms scan time", self.config.scan_time_ms);

        #[cfg(feature = "enhanced-monitoring")]
        if self.enable_enhanced_monitoring {
            info!("Enhanced monitoring enabled");
        }

        let mut ticker = interval(Duration::from_millis(self.config.scan_time_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;

            let scan_start = Instant::now();
            let scan_num = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;

            // Take snapshot before processing (for enhanced monitoring)
            #[cfg(feature = "enhanced-monitoring")]
            let pre_scan_signals = if self.enable_enhanced_monitoring {
                Some(self.bus.snapshot())
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

                if let Err(e) = block.execute(&self.bus) {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    
                    #[cfg(feature = "metrics")]
                    counter!("petra_block_errors_total", "block" => block.name().to_string()).increment(1);
                    
                    warn!("Block '{}' error: {}", block.name(), e);
                }

                #[cfg(feature = "enhanced-monitoring")]
                if let Some(start) = block_start {
                    if self.enable_enhanced_monitoring {
                        let duration = start.elapsed();
                        self.block_execution_times.write()
                            .insert(block.name().to_string(), duration);
                        
                        #[cfg(feature = "metrics")]
                        histogram!("petra_block_execution_time_us", "block" => block.name().to_string())
                            .record(duration.as_micros() as f64);
                    }
                }
            }

            let scan_duration = scan_start.elapsed();

            // Standard jitter monitoring
            {
                let mut jitter_buffer = self.scan_jitter_buffer.lock().unwrap();
                jitter_buffer.push_back(scan_duration);
                if jitter_buffer.len() > 100 {
                    jitter_buffer.pop_front();
                }
                
                // Check for excessive jitter
                if scan_duration > self.target_scan_time * 5 {
                    warn!("Excessive scan time: {:?} (target: {:?})", 
                          scan_duration, self.target_scan_time);
                }
            }

            // Enhanced monitoring
            #[cfg(feature = "enhanced-monitoring")]
            if self.enable_enhanced_monitoring {
                // Record scan time
                self.scan_times.write().push(scan_duration);
                
                // Detect signal changes and send notifications
                if let Some(pre_scan) = pre_scan_signals {
                    let post_scan = self.bus.snapshot();
                    for (name, pre_value) in &pre_scan {
                        if let Some(post_value) = post_scan.get(name) {
                            if pre_value != post_value {
                                if let Some(tx) = &self.signal_change_tx {
                                    let _ = tx.send((name.clone(), post_value.clone())).await;
                                }
                                
                                #[cfg(feature = "metrics")]
                                counter!("petra_signal_changes_total", "signal" => name.clone()).increment(1);
                            }
                        }
                    }
                }
            }

            // Update metrics
            #[cfg(feature = "metrics")]
            {
                histogram!("petra_scan_duration_us").record(scan_duration.as_micros() as f64);
                counter!("petra_scan_count_total").absolute(scan_num);
                gauge!("petra_engine_running").set(1.0);
            }

            // Log periodically
            if scan_num % 1000 == 0 {
                let error_count = self.error_count.load(Ordering::Relaxed);
                let uptime = self.start_time.elapsed();
                
                info!(
                    "Engine stats - Scans: {}, Errors: {}, Uptime: {:?}, Avg scan: {:?}",
                    scan_num, error_count, uptime,
                    self.get_average_scan_time()
                );
            }

            // Update stats handle
            self.update_stats();
        }

        self.running.store(false, Ordering::Relaxed);
        
        #[cfg(feature = "metrics")]
        gauge!("petra_engine_running").set(0.0);
        
        info!("Engine stopped");
        Ok(())
    }

    pub fn stop(&self) {
        info!("Stopping engine...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn get_stats(&self) -> EngineStats {
        self.stats_handle.read().clone()
    }

    pub fn get_bus(&self) -> &SignalBus {
        &self.bus
    }

    pub fn get_scan_count(&self) -> u64 {
        self.scan_count.load(Ordering::Relaxed)
    }

    pub fn get_error_count(&self) -> u64 {
        self.error_count.load(Ordering::Relaxed)
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn get_average_scan_time(&self) -> Duration {
        let jitter_buffer = self.scan_jitter_buffer.lock().unwrap();
        if jitter_buffer.is_empty() {
            return Duration::from_secs(0);
        }
        
        let sum: Duration = jitter_buffer.iter().sum();
        sum / jitter_buffer.len() as u32
    }

    fn update_stats(&self) {
        let mut stats = self.stats_handle.write();
        stats.running = self.running.load(Ordering::Relaxed);
        stats.scan_count = self.scan_count.load(Ordering::Relaxed);
        stats.error_count = self.error_count.load(Ordering::Relaxed);
        stats.uptime_secs = self.start_time.elapsed().as_secs();
        
        #[cfg(feature = "enhanced-monitoring")]
        if self.enable_enhanced_monitoring {
            // Calculate average and max scan times
            let scan_times = self.scan_times.read();
            if !scan_times.is_empty() {
                let times: Vec<f64> = scan_times.iter()
                    .map(|d| d.as_micros() as f64)
                    .collect();
                
                stats.avg_scan_time_us = Some(times.iter().sum::<f64>() / times.len() as f64);
                stats.max_scan_time_us = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max).into();
            }
            
            // Copy block execution times
            let block_times = self.block_execution_times.read();
            if !block_times.is_empty() {
                let mut times_us = HashMap::new();
                for (name, duration) in block_times.iter() {
                    times_us.insert(name.clone(), duration.as_micros() as f64);
                }
                stats.block_times_us = Some(times_us);
            }
        }
    }

    #[cfg(feature = "enhanced-monitoring")]
    pub fn get_detailed_stats(&self) -> Option<DetailedStats> {
        if !self.enable_enhanced_monitoring {
            return None;
        }

        let scan_times = self.scan_times.read();
        let block_times = self.block_execution_times.read();
        
        Some(DetailedStats {
            scan_time_history: scan_times.iter().map(|d| d.as_micros() as u64).collect(),
            block_execution_times: block_times.iter()
                .map(|(k, v)| (k.clone(), v.as_micros() as u64))
                .collect(),
            memory_usage: self.estimate_memory_usage(),
        })
    }

    #[cfg(feature = "enhanced-monitoring")]
    fn estimate_memory_usage(&self) -> usize {
        // Rough estimation of memory usage
        let signal_count = self.bus.len();
        let block_count = self.blocks.len();
        
        // Assume average sizes
        let signal_memory = signal_count * 64;  // Signal name + value
        let block_memory = block_count * 256;   // Block struct + strings
        let buffer_memory = 1000 * 16;          // Scan time buffer
        
        signal_memory + block_memory + buffer_memory
    }

    #[cfg(feature = "enhanced-monitoring")]
    pub fn enable_enhanced_monitoring(&mut self, enable: bool) {
        self.enable_enhanced_monitoring = enable;
        info!("Enhanced monitoring {}", if enable { "enabled" } else { "disabled" });
    }
}

#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone, Serialize)]
pub struct DetailedStats {
    pub scan_time_history: Vec<u64>,
    pub block_execution_times: HashMap<String, u64>,
    pub memory_usage: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{SignalConfig, BlockConfig};

    #[tokio::test]
    async fn test_engine_creation() {
        let config = Config {
            signals: vec![
                SignalConfig {
                    name: "test_signal".to_string(),
                    signal_type: "bool".to_string(),
                    initial: serde_yaml::Value::Bool(false),
                },
            ],
            blocks: vec![],
            scan_time_ms: 100,
            mqtt: None,
            s7: None,
            alarms: None,
            history: None,
            engine_config: None,
        };

        let engine = Engine::new(config).unwrap();
        assert!(!engine.is_running());
        assert_eq!(engine.get_scan_count(), 0);
        assert_eq!(engine.get_error_count(), 0);
    }

    #[cfg(feature = "enhanced-monitoring")]
    #[tokio::test]
    async fn test_enhanced_monitoring() {
        let mut config = Config {
            signals: vec![],
            blocks: vec![],
            scan_time_ms: 100,
            mqtt: None,
            s7: None,
            alarms: None,
            history: None,
            engine_config: Some(serde_yaml::Value::Mapping({
                let mut map = serde_yaml::Mapping::new();
                map.insert(
                    serde_yaml::Value::String("enhanced_monitoring".to_string()),
                    serde_yaml::Value::Bool(true),
                );
                map
            })),
        };

        let engine = Engine::new(config).unwrap();
        assert!(engine.enable_enhanced_monitoring);
    }
}

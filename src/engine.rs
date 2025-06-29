use crate::{error::*, signal::SignalBus, block::*, config::Config, value::Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use std::collections::VecDeque;
use tokio::time::interval;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn, debug};
#[cfg(feature = "metrics")]
use metrics::{gauge, counter, histogram};

// Provide no-op macros when the `metrics` feature is disabled so that the
// instrumentation calls compile without pulling in the `metrics` crate.
#[cfg(not(feature = "metrics"))]
mod metrics_stub {
    pub struct Metric;
    impl Metric {
        pub fn set(&self, _v: impl Into<f64>) {}
        pub fn record(&self, _v: impl Into<f64>) {}
        pub fn increment(&self, _v: impl Into<i64>) {}
    }
}
use metrics_stub::Metric;

#[cfg(not(feature = "metrics"))]
macro_rules! gauge {
    ($($t:tt)*) => { Metric {} };
}
#[cfg(not(feature = "metrics"))]
macro_rules! counter {
    ($($t:tt)*) => { Metric {} };
}
#[cfg(not(feature = "metrics"))]
macro_rules! histogram {
    ($($t:tt)*) => { Metric {} };
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
    scan_jitter_buffer: Arc<Mutex<VecDeque<Duration>>>,
    target_scan_time: Duration,
    stats_handle: Arc<parking_lot::RwLock<EngineStats>>,
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub running: bool,
    pub scan_count: u64,
    pub error_count: u64,
    pub uptime_secs: u64,
    pub signal_count: usize,
    pub block_count: usize,
}

impl Engine {
    pub fn new(config: Config) -> Result<Self> {
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
            stats_handle: Arc::new(parking_lot::RwLock::new(EngineStats {
                running: false,
                scan_count: 0,
                error_count: 0,
                uptime_secs: 0,
                signal_count: config.signals.len(),
                block_count: config.blocks.len(),
            })),
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
                        "Failed to create block '{}': {}",
                        block_config.name, e
                    )));
                }
            }
        }

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
            stats_handle: Arc::new(parking_lot::RwLock::new(EngineStats {
                running: false,
                scan_count: 0,
                error_count: 0,
                uptime_secs: 0,
                signal_count: config.signals.len(),
                block_count: config.blocks.len(),
            })),
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
        *self.stats_handle.write() = self.stats();

        let mut ticker = interval(Duration::from_millis(self.config.scan_time_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        let mut last_scan_start = Instant::now();

        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;

            let scan_start = Instant::now();
            let actual_interval = scan_start.duration_since(last_scan_start);
            last_scan_start = scan_start;
            
            let scan_num = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;

            // Calculate jitter (deviation from target scan time)
            let jitter = if actual_interval > self.target_scan_time {
                actual_interval - self.target_scan_time
            } else {
                self.target_scan_time - actual_interval
            };

            // Update jitter metrics
            {
                let mut buffer = self.scan_jitter_buffer.lock().await;
                if buffer.len() >= 100 {
                    buffer.pop_front();
                }
                buffer.push_back(jitter);

                // Calculate average and max jitter
                let avg_jitter = buffer.iter().sum::<Duration>() / buffer.len() as u32;
                let max_jitter = buffer.iter().max().copied().unwrap_or(Duration::ZERO);
                
                gauge!("petra_scan_jitter_avg_us").set(avg_jitter.as_micros() as f64);
                gauge!("petra_scan_jitter_max_us").set(max_jitter.as_micros() as f64);
                gauge!("petra_scan_variance_us").set(jitter.as_micros() as f64);

                // Warn if jitter exceeds 5x scan time
                if jitter > self.target_scan_time * 5 {
                    warn!(
                        "Excessive scan jitter detected: {:?} ({}x scan time)",
                        jitter,
                        jitter.as_millis() / self.target_scan_time.as_millis()
                    );
                    counter!("petra_scan_jitter_warnings").increment(1);
                }
            }

            // Take snapshot before processing
            let pre_scan_signals = self.bus.snapshot();
            
            // Execute all blocks
            for block in &mut self.blocks {
                if let Err(e) = block.execute(&self.bus) {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    warn!("Block '{}' error: {}", block.name(), e);
                }
            }

            // Determine which signals changed and send them
            if let Some(tx) = &self.signal_change_tx {
                let post_scan_signals = self.bus.snapshot();
                
                for (name, new_value) in &post_scan_signals {
                    if let Some(old_value) = pre_scan_signals.iter().find(|(n, _)| n == name) {
                        if &old_value.1 != new_value {
                            let _ = tx.send((name.clone(), new_value.clone())).await;
                        }
                    }
                }
            }

            let scan_duration = scan_start.elapsed();

            // Update scan duration metrics
            histogram!("petra_scan_duration_seconds").record(scan_duration.as_secs_f64());
            gauge!("petra_scan_duration_ms").set(scan_duration.as_millis() as f64);

            if scan_duration.as_millis() > self.config.scan_time_ms as u128 {
                warn!(
                    "Scan {} overrun: {:?} > {}ms",
                    scan_num, scan_duration, self.config.scan_time_ms
                );
                counter!("petra_scan_overruns_total").increment(1);
            }

            if scan_num % 1000 == 0 {
                let stats = self.stats();
                info!(
                    "Status: {} scans, {} errors, uptime: {}s",
                    stats.scan_count, stats.error_count, stats.uptime_secs
                );
            }

            *self.stats_handle.write() = self.stats();
        }

        info!("Engine stopped after {} scans", self.scan_count.load(Ordering::Relaxed));
        *self.stats_handle.write() = self.stats();
        Ok(())
    }

    pub fn stop(&self) {
        info!("Stopping engine...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn bus(&self) -> &SignalBus {
        &self.bus
    }

    pub fn stats(&self) -> EngineStats {
        EngineStats {
            running: self.running.load(Ordering::Relaxed),
            scan_count: self.scan_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
            signal_count: self.bus.signal_count(),
            block_count: self.blocks.len(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn stats_handle(&self) -> Arc<parking_lot::RwLock<EngineStats>> {
        self.stats_handle.clone()
    }
}

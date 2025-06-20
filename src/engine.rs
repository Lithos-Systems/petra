use crate::{error::*, signal::SignalBus, block::*, config::Config, value::Value, mqtt::MqttHandler};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;
use tokio::time::{interval, Duration};
use tracing::{info, warn, debug};

pub struct Engine {
    bus: SignalBus,
    blocks: Vec<Box<dyn Block>>,
    config: Config,
    running: Arc<AtomicBool>,
    scan_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    start_time: Instant,
    // Track which signals changed during scan
    changed_signals: Vec<String>,
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
            config,
            running: Arc::new(AtomicBool::new(false)),
            scan_count: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
            changed_signals: Vec::new(),
        })
    }

    pub async fn run(&mut self, mqtt_handler: &mut MqttHandler) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(PlcError::Config("Engine is already running".into()));
        }

        self.running.store(true, Ordering::Relaxed);
        info!("Engine starting with {}ms scan time", self.config.scan_time_ms);

        let mut ticker = interval(Duration::from_millis(self.config.scan_time_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;

            let scan_start = Instant::now();
            let scan_num = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;

            // Take snapshot before processing
            let pre_scan_signals = self.bus.snapshot();
            
            // Execute all blocks
            for block in &mut self.blocks {
                if let Err(e) = block.execute(&self.bus) {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    warn!("Block '{}' error: {}", block.name(), e);
                }
            }

            // Determine which signals changed
            let post_scan_signals = self.bus.snapshot();
            self.changed_signals.clear();
            
            for (name, new_value) in &post_scan_signals {
                if let Some(old_value) = pre_scan_signals.iter().find(|(n, _)| n == name) {
                    if &old_value.1 != new_value {
                        self.changed_signals.push(name.clone());
                    }
                } else {
                    // New signal (shouldn't happen in normal operation)
                    self.changed_signals.push(name.clone());
                }
            }

            // Publish changed signals at end of scan
            if !self.changed_signals.is_empty() {
                debug!("Publishing {} changed signals", self.changed_signals.len());
                for signal_name in &self.changed_signals {
                    if let Ok(value) = self.bus.get(signal_name) {
                        mqtt_handler.publish_signal_change(signal_name, &value).await;
                    }
                }
            }

            let scan_duration = scan_start.elapsed();

            if scan_duration.as_millis() > self.config.scan_time_ms as u128 {
                warn!(
                    "Scan {} overrun: {:?} > {}ms",
                    scan_num, scan_duration, self.config.scan_time_ms
                );
            }

            if scan_num % 1000 == 0 {
                let stats = self.stats();
                info!(
                    "Status: {} scans, {} errors, uptime: {}s",
                    stats.scan_count, stats.error_count, stats.uptime_secs
                );
            }
        }

        info!("Engine stopped after {} scans", self.scan_count.load(Ordering::Relaxed));
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
}

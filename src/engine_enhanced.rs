use crate::{error::*, signal::SignalBus, block::*, config::Config, value::Value};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Instant, Duration};
use tokio::time::{interval};
use tokio::sync::mpsc;
use tracing::{info, warn, debug, error};
use metrics::{histogram, counter, gauge};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use parking_lot::RwLock;

pub struct EnhancedEngine {
    bus: SignalBus,
    blocks: Vec<Box<dyn Block>>,
    config: Config,
    running: Arc<AtomicBool>,
    scan_count: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    start_time: Instant,
    signal_change_tx: Option<mpsc::Sender<(String, Value)>>,
    scan_times: Arc<RwLock<AllocRingBuffer<Duration>>>,
    block_execution_times: Arc<RwLock<HashMap<String, Duration>>>,
}

impl EnhancedEngine {
    pub fn new(config: Config) -> Result<Self> {
        // Initialize metrics
        gauge!("petra_engine_scan_time_ms").set(config.scan_time_ms as f64);
        gauge!("petra_engine_signal_count").set(config.signals.len() as f64);
        gauge!("petra_engine_block_count").set(config.blocks.len() as f64);
        
        // ... existing initialization code ...
        
        Ok(Self {
            // ... existing fields ...
            scan_times: Arc::new(RwLock::new(AllocRingBuffer::new(1000))),
            block_execution_times: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(PlcError::Config("Engine is already running".into()));
        }

        self.running.store(true, Ordering::Relaxed);
        info!("Engine starting with {}ms scan time", self.config.scan_time_ms);

        let mut ticker = interval(Duration::from_millis(self.config.scan_time_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Periodic optimization task
        let bus_clone = self.bus.clone();
        tokio::spawn(async move {
            let mut optimization_interval = interval(Duration::from_secs(60));
            loop {
                optimization_interval.tick().await;
                if let Some(optimized_bus) = bus_clone.as_any().downcast_ref::<OptimizedSignalBus>() {
                    optimized_bus.optimize_cache();
                }
            }
        });

        while self.running.load(Ordering::Relaxed) {
            ticker.tick().await;

            let scan_start = Instant::now();
            let scan_num = self.scan_count.fetch_add(1, Ordering::Relaxed) + 1;

            // Take snapshot before processing
            let pre_scan_signals = self.bus.snapshot();
            
            // Execute all blocks with timing
            for block in &mut self.blocks {
                let block_start = Instant::now();
                
                if let Err(e) = block.execute(&self.bus) {
                    self.error_count.fetch_add(1, Ordering::Relaxed);
                    counter!("petra_block_errors_total", "block" => block.name().to_string()).increment(1);
                    warn!("Block '{}' error: {}", block.name(), e);
                }
                
                let block_duration = block_start.elapsed();
                self.block_execution_times.write()
                    .insert(block.name().to_string(), block_duration);
                
                histogram!("petra_block_execution_seconds", "block" => block.name().to_string())
                    .record(block_duration.as_secs_f64());
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
            self.scan_times.write().push(scan_duration);
            
            // Update metrics
            histogram!("petra_scan_duration_seconds").record(scan_duration.as_secs_f64());
            counter!("petra_scan_total").increment(1);
            gauge!("petra_scan_count").set(scan_num as f64);

            if scan_duration.as_millis() > self.config.scan_time_ms as u128 {
                counter!("petra_scan_overruns_total").increment(1);
                warn!(
                    "Scan {} overrun: {:?} > {}ms",
                    scan_num, scan_duration, self.config.scan_time_ms
                );
            }

            if scan_num % 1000 == 0 {
                let stats = self.detailed_stats();
                info!(
                    "Status: {} scans, {} errors, uptime: {}s, avg scan: {:?}",
                    stats.scan_count, stats.error_count, stats.uptime_secs, stats.avg_scan_time
                );
            }
        }

        info!("Engine stopped after {} scans", self.scan_count.load(Ordering::Relaxed));
        Ok(())
    }

    pub fn detailed_stats(&self) -> DetailedStats {
        let scan_times = self.scan_times.read();
        let avg_scan_time = if scan_times.is_empty() {
            Duration::from_millis(0)
        } else {
            let total: Duration = scan_times.iter().sum();
            total / scan_times.len() as u32
        };
        
        DetailedStats {
            running: self.running.load(Ordering::Relaxed),
            scan_count: self.scan_count.load(Ordering::Relaxed),
            error_count: self.error_count.load(Ordering::Relaxed),
            uptime_secs: self.start_time.elapsed().as_secs(),
            signal_count: self.bus.signal_count(),
            block_count: self.blocks.len(),
            avg_scan_time,
            recent_scan_times: scan_times.iter().cloned().collect(),
            block_execution_times: self.block_execution_times.read().clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DetailedStats {
    pub running: bool,
    pub scan_count: u64,
    pub error_count: u64,
    pub uptime_secs: u64,
    pub signal_count: usize,
    pub block_count: usize,
    pub avg_scan_time: Duration,
    pub recent_scan_times: Vec<Duration>,
    pub block_execution_times: HashMap<String, Duration>,
}

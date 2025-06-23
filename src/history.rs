use crate::{error::*, value::Value, signal::SignalBus};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};
use tracing::{info, warn, error, debug};
use chrono::Utc;

// Only import influxdb2 if the history feature is enabled
#[cfg(feature = "history")]
use influxdb2::models::{DataPoint, WriteDataPoint};
#[cfg(feature = "history")]
use influxdb2::Client;
#[cfg(feature = "history")]
use futures::stream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// InfluxDB connection string
    pub url: String,
    /// Database/bucket name
    pub database: String,
    /// API token for InfluxDB3
    pub token: Option<String>,
    /// Organization (for InfluxDB3)
    pub org: Option<String>,
    /// Batch size for writes
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Flush interval in milliseconds
    #[serde(default = "default_flush_interval")]
    pub flush_interval_ms: u64,
    /// Local buffer size per signal
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    /// Signals to track (empty = all)
    #[serde(default)]
    pub tracked_signals: Vec<String>,
    /// Downsample rules
    #[serde(default)]
    pub downsample_rules: Vec<DownsampleRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownsampleRule {
    pub signal_pattern: String,
    pub min_interval_ms: u64,
    pub aggregation: AggregationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationType {
    Last,
    Mean,
    Max,
    Min,
}

fn default_batch_size() -> usize { 1000 }
fn default_flush_interval() -> u64 { 1000 }
fn default_buffer_size() -> usize { 10000 }

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8086".to_string(),
            database: "petra".to_string(),
            token: None,
            org: None,
            batch_size: default_batch_size(),
            flush_interval_ms: default_flush_interval(),
            buffer_size: default_buffer_size(),
            tracked_signals: Vec::new(),
            downsample_rules: Vec::new(),
        }
    }
}

/// Thread-safe signal history buffer
#[derive(Clone)]
pub struct SignalHistory {
    buffer: Arc<RwLock<AllocRingBuffer<(Instant, Value)>>>,
    last_write: Arc<RwLock<Option<Instant>>>,
    downsample_rule: Option<DownsampleRule>,
}

impl SignalHistory {
    pub fn new(capacity: usize, rule: Option<DownsampleRule>) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(AllocRingBuffer::new(capacity))),
            last_write: Arc::new(RwLock::new(None)),
            downsample_rule: rule,
        }
    }

    pub fn should_record(&self, now: Instant) -> bool {
        if let Some(rule) = &self.downsample_rule {
            if let Some(last) = *self.last_write.read() {
                if now.duration_since(last).as_millis() < rule.min_interval_ms as u128 {
                    return false;
                }
            }
        }
        true
    }

    pub fn push(&self, value: Value, now: Instant) {
        if self.should_record(now) {
            self.buffer.write().push((now, value));
            *self.last_write.write() = Some(now);
        }
    }

    pub fn get_recent(&self, duration: Duration) -> Vec<(Instant, Value)> {
        let cutoff = Instant::now() - duration;
        self.buffer
            .read()
            .iter()
            .filter(|(t, _)| *t > cutoff)
            .cloned()
            .collect()
    }

    pub fn get_all(&self) -> Vec<(Instant, Value)> {
        self.buffer.read().iter().cloned().collect()
    }
}

/// Manages historical data collection and storage
pub struct HistoryManager {
    config: HistoryConfig,
    client: Client,
    bus: SignalBus,
    histories: Arc<RwLock<HashMap<String, SignalHistory>>>,
    write_buffer: Arc<RwLock<Vec<Point>>>,
    signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
    running: Arc<RwLock<bool>>,
}

impl HistoryManager {
    pub fn new(config: HistoryConfig, bus: SignalBus) -> Result<Self> {
        let client = Client::new(config.url.clone())
            .with_auth_token(config.token.clone().unwrap_or_default());
        
        Ok(Self {
            config,
            client,
            bus,
            histories: Arc::new(RwLock::new(HashMap::new())),
            write_buffer: Arc::new(RwLock::new(Vec::with_capacity(default_batch_size()))),
            signal_change_rx: None,
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub fn set_signal_change_channel(&mut self, rx: mpsc::Receiver<(String, Value)>) {
        self.signal_change_rx = Some(rx);
    }

    /// Initialize history tracking for signals
    pub fn initialize_histories(&self) {
        let signals = if self.config.tracked_signals.is_empty() {
            self.bus.snapshot().into_iter().map(|(name, _)| name).collect()
        } else {
            self.config.tracked_signals.clone()
        };

        let mut histories = self.histories.write();
        for signal in signals {
            let rule = self.config.downsample_rules
                .iter()
                .find(|r| signal.contains(&r.signal_pattern))
                .cloned();
            
            histories.insert(
                signal.clone(),
                SignalHistory::new(self.config.buffer_size, rule)
            );
        }
        
        info!("Initialized history tracking for {} signals", histories.len());
    }

    pub async fn run(&mut self) -> Result<()> {
        *self.running.write() = true;
        self.initialize_histories();
        
        let mut flush_interval = interval(Duration::from_millis(self.config.flush_interval_ms));
        flush_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        info!("History manager started, writing to {}", self.config.url);
        
        while *self.running.read() {
            tokio::select! {
                Some((name, value)) = async {
                    if let Some(rx) = &mut self.signal_change_rx {
                        rx.recv().await
                    } else {
                        None
                    }
                } => {
                    self.record_signal_change(&name, value).await;
                }
                
                _ = flush_interval.tick() => {
                    if let Err(e) = self.flush_to_influx().await {
                        error!("Failed to flush to InfluxDB: {}", e);
                    }
                }
            }
        }
        
        // Final flush
        let _ = self.flush_to_influx().await;
        Ok(())
    }

    async fn record_signal_change(&self, name: &str, value: Value) {
        let now = Instant::now();
        
        // Update local history
        if let Some(history) = self.histories.read().get(name) {
            history.push(value.clone(), now);
        }
        
        // Add to write buffer
        let point = Point::new("signal")
            .tag("name", name)
            .tag("type", value.type_name())
            .field("value", value_to_field(&value))
            .timestamp(chrono::Utc::now().timestamp_nanos());
        
        let mut buffer = self.write_buffer.write();
        buffer.push(point);
        
        // Flush if buffer is full
        if buffer.len() >= self.config.batch_size {
            drop(buffer);
            if let Err(e) = self.flush_to_influx().await {
                error!("Failed to flush on buffer full: {}", e);
            }
        }
    }

    async fn flush_to_influx(&self) -> Result<()> {
        let mut buffer = self.write_buffer.write();
        if buffer.is_empty() {
            return Ok(());
        }
        
        let points: Vec<Point> = buffer.drain(..).collect();
        drop(buffer);
        
        debug!("Flushing {} points to InfluxDB", points.len());
        
        let write_query = WriteQuery::new(&self.config.database, points)
            .precision(Precision::Nanoseconds);
        
        self.client
            .write(write_query)
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("InfluxDB write error: {}", e)
            )))?;
        
        Ok(())
    }

    pub fn get_signal_history(&self, signal: &str, duration: Duration) -> Option<Vec<(Instant, Value)>> {
        self.histories.read().get(signal).map(|h| h.get_recent(duration))
    }

    pub fn stop(&self) {
        *self.running.write() = false;
    }
}

fn value_to_field(value: &Value) -> influxdb3_client::Field {
    match value {
        Value::Bool(b) => (*b as i64).into(),
        Value::Int(i) => (*i as i64).into(),
        Value::Float(f) => (*f).into(),
    }
}

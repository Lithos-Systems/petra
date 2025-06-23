use crate::{error::*, value::Value, signal::SignalBus};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Instant};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use parquet::basic::Compression;
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Base directory for historical data
    pub data_dir: PathBuf,
    /// Maximum file size in MB before rotation
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
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
    /// Retention days (0 = unlimited)
    #[serde(default)]
    pub retention_days: u32,
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

fn default_max_file_size() -> u64 { 100 }
fn default_batch_size() -> usize { 1000 }
fn default_flush_interval() -> u64 { 1000 }
fn default_buffer_size() -> usize { 10000 }

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data/history"),
            max_file_size_mb: default_max_file_size(),
            batch_size: default_batch_size(),
            flush_interval_ms: default_flush_interval(),
            buffer_size: default_buffer_size(),
            tracked_signals: Vec::new(),
            downsample_rules: Vec::new(),
            retention_days: 0,
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

struct ParquetWriter {
    writer: ArrowWriter<File>,
    path: PathBuf,
    size: u64,
    row_count: usize,
}

/// Manages historical data collection and storage
pub struct HistoryManager {
    config: HistoryConfig,
    bus: SignalBus,
    histories: Arc<RwLock<HashMap<String, SignalHistory>>>,
    write_buffer: Arc<RwLock<Vec<(DateTime<Utc>, String, Value)>>>,
    signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
    running: Arc<RwLock<bool>>,
    current_writer: Option<ParquetWriter>,
    schema: Arc<Schema>,
    start_instant: Instant,
}

impl HistoryManager {
    pub fn new(config: HistoryConfig, bus: SignalBus) -> Result<Self> {
        // Create data directory
        fs::create_dir_all(&config.data_dir)?;
        
        // Define schema for time-series data
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Timestamp(TimeUnit::Nanosecond, None), false),
            Field::new("signal", DataType::Utf8, false),
            Field::new("value_type", DataType::Utf8, false),
            Field::new("value_bool", DataType::Boolean, true),
            Field::new("value_int", DataType::Int32, true),
            Field::new("value_float", DataType::Float64, true),
        ]));
        
        Ok(Self {
            config,
            bus,
            histories: Arc::new(RwLock::new(HashMap::new())),
            write_buffer: Arc::new(RwLock::new(Vec::with_capacity(default_batch_size()))),
            signal_change_rx: None,
            running: Arc::new(RwLock::new(false)),
            current_writer: None,
            schema,
            start_instant: Instant::now(),
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
        
        // Cleanup interval for old files
        let mut cleanup_interval = interval(Duration::from_secs(3600)); // 1 hour
        
        info!("History manager started, writing to {:?}", self.config.data_dir);
        
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
                    if let Err(e) = self.flush_to_parquet().await {
                        error!("Failed to flush to Parquet: {}", e);
                    }
                }
                
                _ = cleanup_interval.tick() => {
                    if self.config.retention_days > 0 {
                        if let Err(e) = self.cleanup_old_files().await {
                            error!("Failed to cleanup old files: {}", e);
                        }
                    }
                }
            }
        }
        
        // Final flush
        let _ = self.flush_to_parquet().await;
        
        // Close current writer
        if let Some(writer) = self.current_writer.take() {
            self.close_writer(writer)?;
        }
        
        Ok(())
    }

    async fn record_signal_change(&mut self, name: &str, value: Value) {
        let now = Instant::now();
        let timestamp = Utc::now();
        
        // Update local history
        if let Some(history) = self.histories.read().get(name) {
            history.push(value.clone(), now);
        }
        
        // Add to write buffer
        let mut buffer = self.write_buffer.write();
        buffer.push((timestamp, name.to_string(), value));
        
        // Flush if buffer is full
        if buffer.len() >= self.config.batch_size {
            drop(buffer);
            if let Err(e) = self.flush_to_parquet().await {
                error!("Failed to flush on buffer full: {}", e);
            }
        }
    }

    async fn flush_to_parquet(&mut self) -> Result<()> {
        let entries: Vec<_> = {
            let mut buffer = self.write_buffer.write();
            if buffer.is_empty() {
                return Ok(());
            }
            buffer.drain(..).collect()
        };
        
        debug!("Flushing {} entries to Parquet", entries.len());
        
        // Prepare arrays
        let mut timestamp_builder = TimestampNanosecondBuilder::with_capacity(entries.len());
        let mut signal_builder = StringBuilder::new();
        let mut value_type_builder = StringBuilder::new();
        let mut bool_builder = BooleanBuilder::new();
        let mut int_builder = Int32Builder::new();
        let mut float_builder = Float64Builder::new();
        
        for (ts, signal, value) in entries {
            timestamp_builder.append_value(ts.timestamp_nanos_opt().unwrap_or(0));
            signal_builder.append_value(&signal);
            
            match value {
                Value::Bool(b) => {
                    value_type_builder.append_value("bool");
                    bool_builder.append_value(b);
                    int_builder.append_null();
                    float_builder.append_null();
                }
                Value::Int(i) => {
                    value_type_builder.append_value("int");
                    bool_builder.append_null();
                    int_builder.append_value(i);
                    float_builder.append_null();
                }
                Value::Float(f) => {
                    value_type_builder.append_value("float");
                    bool_builder.append_null();
                    int_builder.append_null();
                    float_builder.append_value(f);
                }
            }
        }
        
        // Create record batch
        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(timestamp_builder.finish()),
                Arc::new(signal_builder.finish()),
                Arc::new(value_type_builder.finish()),
                Arc::new(bool_builder.finish()),
                Arc::new(int_builder.finish()),
                Arc::new(float_builder.finish()),
            ],
        ).map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create record batch: {}", e)
        )))?;
        
        // Get or create writer
        let writer = self.get_or_create_writer()?;
        
        // Write batch
        writer.writer.write(&batch)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write batch: {}", e)
            )))?;
        
        writer.row_count += batch.num_rows();
        
        // Check file size and rotate if needed
        let file_metadata = std::fs::metadata(&writer.path)?;
        writer.size = file_metadata.len();
        
        if writer.size > self.config.max_file_size_mb * 1024 * 1024 {
            self.rotate_file()?;
        }
        
        Ok(())
    }

    fn get_or_create_writer(&mut self) -> Result<&mut ParquetWriter> {
        if self.current_writer.is_none() {
            let filename = format!("petra_{}.parquet", Utc::now().timestamp_nanos_opt().unwrap_or(0));
            let path = self.config.data_dir.join(filename);
            
            let file = File::create(&path)?;
            
            let props = WriterProperties::builder()
                .set_compression(Compression::ZSTD(Default::default()))
                .build();
            
            let writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))
                .map_err(|e| PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create parquet writer: {}", e)
                )))?;
            
            self.current_writer = Some(ParquetWriter {
                writer,
                path: path.clone(),
                size: 0,
                row_count: 0,
            });
            
            info!("Created new parquet file: {:?}", path);
        }
        
        Ok(self.current_writer.as_mut().unwrap())
    }

    fn rotate_file(&mut self) -> Result<()> {
        if let Some(writer) = self.current_writer.take() {
            self.close_writer(writer)?;
        }
        Ok(())
    }

    fn close_writer(&self, writer: ParquetWriter) -> Result<()> {
        writer.writer.close()
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to close parquet file: {}", e)
            )))?;
        
        info!("Closed parquet file: {:?} ({} rows, {} MB)", 
            writer.path, 
            writer.row_count,
            writer.size / 1024 / 1024
        );
        
        Ok(())
    }

    async fn cleanup_old_files(&self) -> Result<()> {
        if self.config.retention_days == 0 {
            return Ok(());
        }
        
        let cutoff = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        
        let entries = std::fs::read_dir(&self.config.data_dir)?;
        let mut removed_count = 0;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                let metadata = entry.metadata()?;
                if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();
                    if modified_time < cutoff {
                        if let Err(e) = std::fs::remove_file(&path) {
                            warn!("Failed to remove old file {:?}: {}", path, e);
                        } else {
                            removed_count += 1;
                        }
                    }
                }
            }
        }
        
        if removed_count > 0 {
            info!("Cleaned up {} old parquet files", removed_count);
        }
        
        Ok(())
    }

    pub fn get_signal_history(&self, signal: &str, duration: Duration) -> Option<Vec<(Instant, Value)>> {
        self.histories.read().get(signal).map(|h| h.get_recent(duration))
    }

    pub fn stop(&self) {
        *self.running.write() = false;
    }
}

// src/history.rs
use crate::{error::*, signal::SignalBus, value::Value};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

#[cfg(feature = "parquet-history")]
use parquet::{
    file::properties::WriterProperties,
    arrow::ArrowWriter,
};

#[cfg(feature = "parquet-history")]
use arrow::array::{
    ArrayRef, BooleanArray, Float64Array, Int32Array, StringArray, TimestampMicrosecondArray,
};

#[cfg(feature = "parquet-history")]
use arrow::datatypes::{DataType, Field, Schema};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub storage_path: PathBuf,
    pub retention_days: u32,
    pub buffer_size: usize,
    pub flush_interval_secs: u64,
    
    #[cfg(feature = "history-compression")]
    pub compression: CompressionConfig,
    
    #[cfg(feature = "csv-history")]
    pub csv_enabled: bool,
    
    #[cfg(feature = "parquet-history")]
    pub parquet_enabled: bool,
    
    #[cfg(feature = "remote-history")]
    pub remote_storage: Option<RemoteStorageConfig>,
    
    #[cfg(feature = "history-filtering")]
    pub filters: Vec<HistoryFilter>,
}

#[cfg(feature = "history-compression")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: CompressionAlgorithm,
    pub level: u32,
}

#[cfg(feature = "history-compression")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Zstd,
    Lz4,
    Snappy,
    Gzip,
}

#[cfg(feature = "remote-history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStorageConfig {
    pub backend: RemoteBackend,
    pub upload_interval_secs: u64,
    pub batch_size: usize,
}

#[cfg(feature = "remote-history")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteBackend {
    S3 {
        bucket: String,
        prefix: String,
        region: String,
    },
    ClickHouse {
        url: String,
        database: String,
        table: String,
    },
    InfluxDb {
        url: String,
        org: String,
        bucket: String,
        token: String,
    },
}

#[cfg(feature = "history-filtering")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    pub signal_pattern: String,
    pub sample_rate_ms: Option<u64>,
    pub deadband: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub signal_name: String,
    pub value: Value,
    #[cfg(feature = "history-metadata")]
    pub metadata: Option<EntryMetadata>,
}

#[cfg(feature = "history-metadata")]
#[derive(Debug, Clone)]
pub struct EntryMetadata {
    pub quality: DataQuality,
    pub source: String,
    pub tags: Vec<String>,
}

#[cfg(feature = "history-metadata")]
#[derive(Debug, Clone, Copy)]
pub enum DataQuality {
    Good,
    Uncertain,
    Bad,
}

pub struct HistoryManager {
    config: HistoryConfig,
    bus: SignalBus,
    buffer: Arc<RwLock<Vec<HistoryEntry>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
    
    #[cfg(feature = "parquet-history")]
    parquet_writer: Option<Arc<RwLock<ParquetWriter>>>,
    
    #[cfg(feature = "csv-history")]
    csv_writer: Option<Arc<RwLock<CsvWriter>>>,
    
    #[cfg(feature = "remote-history")]
    remote_uploader: Option<RemoteUploader>,
    
    #[cfg(feature = "history-statistics")]
    statistics: Arc<RwLock<HistoryStatistics>>,
    
    #[cfg(feature = "history-indexing")]
    index: Arc<RwLock<HistoryIndex>>,
}

#[cfg(feature = "history-statistics")]
#[derive(Default, Clone)]
struct HistoryStatistics {
    total_entries: u64,
    entries_per_signal: std::collections::HashMap<String, u64>,
    bytes_written: u64,
    last_flush: Option<DateTime<Utc>>,
    flush_duration_ms: Option<u64>,
}

#[cfg(feature = "history-indexing")]
struct HistoryIndex {
    signal_files: std::collections::HashMap<String, Vec<PathBuf>>,
    time_index: std::collections::BTreeMap<DateTime<Utc>, PathBuf>,
}

impl HistoryManager {
    pub async fn new(config: HistoryConfig, bus: SignalBus) -> Result<Self> {
        // Create storage directory
        tokio::fs::create_dir_all(&config.storage_path).await?;
        
        let buffer = Arc::new(RwLock::new(Vec::with_capacity(config.buffer_size)));
        
        #[cfg(feature = "parquet-history")]
        let parquet_writer = if config.parquet_enabled {
            Some(Arc::new(RwLock::new(ParquetWriter::new(&config).await?)))
        } else {
            None
        };
        
        #[cfg(feature = "csv-history")]
        let csv_writer = if config.csv_enabled {
            Some(Arc::new(RwLock::new(CsvWriter::new(&config).await?)))
        } else {
            None
        };
        
        #[cfg(feature = "remote-history")]
        let remote_uploader = if let Some(remote_config) = &config.remote_storage {
            Some(RemoteUploader::new(remote_config.clone()).await?)
        } else {
            None
        };
        
        Ok(Self {
            config,
            bus,
            buffer,
            shutdown_tx: None,
            #[cfg(feature = "parquet-history")]
            parquet_writer,
            #[cfg(feature = "csv-history")]
            csv_writer,
            #[cfg(feature = "remote-history")]
            remote_uploader,
            #[cfg(feature = "history-statistics")]
            statistics: Arc::new(RwLock::new(HistoryStatistics::default())),
            #[cfg(feature = "history-indexing")]
            index: Arc::new(RwLock::new(HistoryIndex {
                signal_files: std::collections::HashMap::new(),
                time_index: std::collections::BTreeMap::new(),
            })),
        })
    }
    
    pub async fn run(mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Subscribe to all signals
        let (signal_tx, mut signal_rx) = mpsc::channel(1000);
        
        // Start flush timer
        let flush_interval = tokio::time::Duration::from_secs(self.config.flush_interval_secs);
        let mut flush_timer = tokio::time::interval(flush_interval);
        
        info!("History manager started with {} second flush interval", 
              self.config.flush_interval_secs);
        
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("History manager shutdown requested");
                    self.flush().await?;
                    break;
                }
                
                signal = signal_rx.recv() => {
                    if let Some((name, value)) = signal {
                        self.record(name, value).await?;
                    }
                }
                
                _ = flush_timer.tick() => {
                    if let Err(e) = self.flush().await {
                        error!("Failed to flush history: {}", e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn record(&self, signal_name: String, value: Value) -> Result<()> {
        // Check filters
        #[cfg(feature = "history-filtering")]
        if !self.should_record(&signal_name, &value).await {
            return Ok(());
        }
        
        let entry = HistoryEntry {
            timestamp: Utc::now(),
            signal_name,
            value,
            #[cfg(feature = "history-metadata")]
            metadata: None,
        };
        
        let mut buffer = self.buffer.write().await;
        buffer.push(entry);
        
        // Auto-flush if buffer is full
        if buffer.len() >= self.config.buffer_size {
            drop(buffer);
            self.flush().await?;
        }
        
        #[cfg(feature = "history-statistics")]
        {
            let mut stats = self.statistics.write().await;
            stats.total_entries += 1;
            *stats.entries_per_signal.entry(signal_name).or_insert(0) += 1;
        }
        
        Ok(())
    }
    
    #[cfg(feature = "history-filtering")]
    async fn should_record(&self, signal_name: &str, _value: &Value) -> bool {
        for filter in &self.config.filters {
            if signal_name.contains(&filter.signal_pattern) {
                // Apply filter logic
                return true;
            }
        }
        false
    }
    
    pub async fn flush(&self) -> Result<()> {
        let start = std::time::Instant::now();
        
        let entries = {
            let mut buffer = self.buffer.write().await;
            std::mem::take(&mut *buffer)
        };
        
        if entries.is_empty() {
            return Ok(());
        }
        
        info!("Flushing {} history entries", entries.len());
        
        // Write to enabled backends
        #[cfg(feature = "parquet-history")]
        if let Some(writer) = &self.parquet_writer {
            writer.write().await.write_batch(&entries).await?;
        }
        
        #[cfg(feature = "csv-history")]
        if let Some(writer) = &self.csv_writer {
            writer.write().await.write_batch(&entries).await?;
        }
        
        #[cfg(feature = "remote-history")]
        if let Some(uploader) = &self.remote_uploader {
            uploader.upload_batch(&entries).await?;
        }
        
        #[cfg(feature = "history-statistics")]
        {
            let mut stats = self.statistics.write().await;
            stats.last_flush = Some(Utc::now());
            stats.flush_duration_ms = Some(start.elapsed().as_millis() as u64);
        }
        
        Ok(())
    }
    
    #[cfg(feature = "history-replay")]
    pub async fn replay_timerange(
        &self, 
        start: DateTime<Utc>, 
        end: DateTime<Utc>,
        signal_filter: Option<Vec<String>>
    ) -> Result<ReplayIterator> {
        info!("Replaying history from {} to {}", start, end);
        
        #[cfg(feature = "history-indexing")]
        {
            let index = self.index.read().await;
            let files = index.time_index
                .range(start..=end)
                .map(|(_, path)| path.clone())
                .collect::<Vec<_>>();
            
            Ok(ReplayIterator::new(files, signal_filter))
        }
        
        #[cfg(not(feature = "history-indexing"))]
        {
            // Scan directory for matching files
            let files = self.find_files_in_range(start, end).await?;
            Ok(ReplayIterator::new(files, signal_filter))
        }
    }
    
    #[cfg(feature = "history-compaction")]
    pub async fn compact(&self, older_than_days: u32) -> Result<()> {
        info!("Compacting history older than {} days", older_than_days);

        let cutoff = Utc::now() - chrono::Duration::days(older_than_days as i64);

        // Find and compact old files
        let files = self.find_files_older_than(cutoff).await?;

        for file in files {
            self.compact(&file).await?;
        }

        Ok(())
    }

    #[cfg(all(feature = "history-replay", not(feature = "history-indexing")))]
    async fn find_files_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<PathBuf>> {
        async fn visit(dir: &Path, start: DateTime<Utc>, end: DateTime<Utc>, files: &mut Vec<PathBuf>) -> Result<()> {
            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;
                if metadata.is_dir() {
                    visit(&path, start, end, files).await?;
                } else if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();
                    if modified_time >= start && modified_time <= end {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        let mut files = Vec::new();
        visit(&self.config.storage_path, start, end, &mut files).await?;
        Ok(files)
    }

    #[cfg(feature = "history-compaction")]
    async fn find_files_older_than(&self, cutoff: DateTime<Utc>) -> Result<Vec<PathBuf>> {
        async fn visit(dir: &Path, cutoff: DateTime<Utc>, files: &mut Vec<PathBuf>) -> Result<()> {
            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;
                if metadata.is_dir() {
                    visit(&path, cutoff, files).await?;
                } else if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();
                    if modified_time < cutoff {
                        files.push(path);
                    }
                }
            }
            Ok(())
        }

        let mut files = Vec::new();
        visit(&self.config.storage_path, cutoff, &mut files).await?;
        Ok(files)
    }

    #[cfg(feature = "history-compaction")]
    async fn compact(&self, file: &Path) -> Result<()> {
        tokio::fs::remove_file(file).await?;
        Ok(())
    }
    
    #[cfg(feature = "history-statistics")]
    pub async fn get_statistics(&self) -> HistoryStatistics {
        self.statistics.read().await.clone()
    }
}

// Parquet writer implementation
#[cfg(feature = "parquet-history")]
struct ParquetWriter {
    path: PathBuf,
    schema: Arc<Schema>,
    current_file: Option<PathBuf>,
    row_group_size: usize,
}

#[cfg(feature = "parquet-history")]
impl ParquetWriter {
    async fn new(config: &HistoryConfig) -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Timestamp(arrow::datatypes::TimeUnit::Microsecond, None), false),
            Field::new("signal", DataType::Utf8, false),
            Field::new("value_type", DataType::Utf8, false),
            Field::new("value_bool", DataType::Boolean, true),
            Field::new("value_int", DataType::Int32, true),
            Field::new("value_float", DataType::Float64, true),
        ]));
        
        Ok(Self {
            path: config.storage_path.join("parquet"),
            schema,
            current_file: None,
            row_group_size: 10000,
        })
    }
    
    async fn write_batch(&mut self, entries: &[HistoryEntry]) -> Result<()> {
        // Implementation for writing to Parquet files
        Ok(())
    }
}

// CSV writer implementation
#[cfg(feature = "csv-history")]
struct CsvWriter {
    path: PathBuf,
    current_file: Option<tokio::fs::File>,
}

#[cfg(feature = "csv-history")]
impl CsvWriter {
    async fn new(config: &HistoryConfig) -> Result<Self> {
        let path = config.storage_path.join("csv");
        tokio::fs::create_dir_all(&path).await?;
        
        Ok(Self {
            path,
            current_file: None,
        })
    }
    
    async fn write_batch(&mut self, entries: &[HistoryEntry]) -> Result<()> {
        // Implementation for writing to CSV files
        Ok(())
    }
}

// Remote uploader
#[cfg(feature = "remote-history")]
struct RemoteUploader {
    config: RemoteStorageConfig,
    #[cfg(feature = "remote-retry")]
    retry_queue: Arc<RwLock<Vec<HistoryEntry>>>,
}

#[cfg(feature = "remote-history")]
impl RemoteUploader {
    async fn new(config: RemoteStorageConfig) -> Result<Self> {
        Ok(Self {
            config,
            #[cfg(feature = "remote-retry")]
            retry_queue: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    async fn upload_batch(&self, entries: &[HistoryEntry]) -> Result<()> {
        match &self.config.backend {
            #[cfg(feature = "s3-history")]
            RemoteBackend::S3 { bucket, prefix, region } => {
                // S3 upload implementation
                Ok(())
            }
            
            #[cfg(feature = "clickhouse-history")]
            RemoteBackend::ClickHouse { url, database, table } => {
                // ClickHouse upload implementation
                Ok(())
            }
            
            #[cfg(feature = "influxdb-history")]
            RemoteBackend::InfluxDb { url, org, bucket, token } => {
                // InfluxDB upload implementation
                Ok(())
            }
            
            _ => Ok(()),
        }
    }
}

// Replay iterator for historical data
#[cfg(feature = "history-replay")]
pub struct ReplayIterator {
    files: Vec<PathBuf>,
    current_file_index: usize,
    signal_filter: Option<Vec<String>>,
}

#[cfg(feature = "history-replay")]
impl ReplayIterator {
    fn new(files: Vec<PathBuf>, signal_filter: Option<Vec<String>>) -> Self {
        Self {
            files,
            current_file_index: 0,
            signal_filter,
        }
    }
    
    pub async fn next_batch(&mut self, batch_size: usize) -> Result<Vec<HistoryEntry>> {
        // Implementation for reading historical data
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_history_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = HistoryConfig {
            storage_path: temp_dir.path().to_path_buf(),
            retention_days: 30,
            buffer_size: 1000,
            flush_interval_secs: 60,
            #[cfg(feature = "history-compression")]
            compression: CompressionConfig {
                enabled: false,
                algorithm: CompressionAlgorithm::Zstd,
                level: 3,
            },
            #[cfg(feature = "csv-history")]
            csv_enabled: true,
            #[cfg(feature = "parquet-history")]
            parquet_enabled: true,
            #[cfg(feature = "remote-history")]
            remote_storage: None,
            #[cfg(feature = "history-filtering")]
            filters: vec![],
        };
        
        let bus = SignalBus::new();
        let manager = HistoryManager::new(config, bus).await.unwrap();
        
        assert!(temp_dir.path().exists());
    }
}

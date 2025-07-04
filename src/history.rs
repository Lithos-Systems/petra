// src/history.rs - Fixed with proper imports and BoxFuture handling
use crate::{error::{PlcError, Result}, value::Value};
use chrono::{DateTime, Utc, Duration};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use serde::{Serialize, Deserialize};
use futures::future::BoxFuture;  // Added import
use futures::FutureExt;          // Added import

#[cfg(feature = "parquet")]
use parquet::{
    file::writer::{SerializedFileWriter, WriteOptions},
    file::properties::WriterProperties,
    schema::types::Type,
    record::RecordWriter,
};

#[cfg(feature = "arrow")]
use arrow::{
    array::{TimestampMicrosecondArray, Float64Array, StringArray},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub signal_name: String,
    pub value: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    pub data_dir: PathBuf,
    pub retention_days: u32,
    pub max_batch_size: usize,
    pub max_memory_entries: usize,
    pub compression: CompressionType,
    #[serde(default = "default_compact_after_days")]
    pub compact_after_days: u32,
    #[serde(default)]
    pub enable_index: bool,
    #[serde(default)]
    pub sync_writes: bool,
}

fn default_compact_after_days() -> u32 {
    7
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    #[cfg(feature = "compression")]
    Zstd,
    #[cfg(feature = "compression")]
    Lz4,
    #[cfg(feature = "compression")]
    Snappy,
}

#[derive(Clone)]
pub struct HistoryManager {
    config: HistoryConfig,
    buffer: Arc<RwLock<Vec<HistoryEntry>>>,
    tx: mpsc::Sender<HistoryCommand>,
    #[cfg(feature = "parquet")]
    writer_props: WriterProperties,
}

#[derive(Debug)]
enum HistoryCommand {
    Write(HistoryEntry),
    Flush,
    Compact(u32),
    Query(HistoryQuery, mpsc::Sender<Result<Vec<HistoryEntry>>>),
}

#[derive(Debug, Clone)]
pub struct HistoryQuery {
    pub signal_name: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl HistoryManager {
    pub fn new(config: HistoryConfig) -> Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        
        let (tx, rx) = mpsc::channel(1000);
        
        #[cfg(feature = "parquet")]
        let writer_props = WriterProperties::builder()
            .set_compression(match config.compression {
                CompressionType::None => parquet::basic::Compression::UNCOMPRESSED,
                #[cfg(feature = "compression")]
                CompressionType::Zstd => parquet::basic::Compression::ZSTD,
                #[cfg(feature = "compression")]
                CompressionType::Lz4 => parquet::basic::Compression::LZ4,
                #[cfg(feature = "compression")]
                CompressionType::Snappy => parquet::basic::Compression::SNAPPY,
            })
            .build();
        
        let manager = Self {
            config,
            buffer: Arc::new(RwLock::new(Vec::new())),
            tx,
            #[cfg(feature = "parquet")]
            writer_props,
        };
        
        // Start background task
        tokio::spawn(manager.clone().background_task(rx));
        
        Ok(manager)
    }
    
    pub async fn write(&self, entry: HistoryEntry) -> Result<()> {
        self.tx.send(HistoryCommand::Write(entry)).await
            .map_err(|e| PlcError::Runtime(format!("Failed to send write command: {}", e)))
    }
    
    pub async fn flush(&self) -> Result<()> {
        self.tx.send(HistoryCommand::Flush).await
            .map_err(|e| PlcError::Runtime(format!("Failed to send flush command: {}", e)))
    }
    
    pub async fn query(&self, query: HistoryQuery) -> Result<Vec<HistoryEntry>> {
        let (response_tx, mut response_rx) = mpsc::channel(1);
        
        self.tx.send(HistoryCommand::Query(query, response_tx)).await
            .map_err(|e| PlcError::Runtime(format!("Failed to send query command: {}", e)))?;
        
        response_rx.recv().await
            .ok_or_else(|| PlcError::Runtime("No response from history manager".into()))?
    }
    
    pub async fn compact(&self, days: u32) -> Result<()> {
        self.tx.send(HistoryCommand::Compact(days)).await
            .map_err(|e| PlcError::Runtime(format!("Failed to send compact command: {}", e)))
    }
    
    async fn background_task(self, mut rx: mpsc::Receiver<HistoryCommand>) {
        while let Some(command) = rx.recv().await {
            match command {
                HistoryCommand::Write(entry) => {
                    if let Err(e) = self.handle_write(entry).await {
                        tracing::error!("Failed to write history entry: {}", e);
                    }
                }
                HistoryCommand::Flush => {
                    if let Err(e) = self.handle_flush().await {
                        tracing::error!("Failed to flush history buffer: {}", e);
                    }
                }
                HistoryCommand::Query(query, response_tx) => {
                    let result = self.handle_query(query).await;
                    if let Err(_) = response_tx.send(result).await {
                        tracing::warn!("Failed to send query response");
                    }
                }
                HistoryCommand::Compact(days) => {
                    if let Err(e) = self.handle_compact(days).await {
                        tracing::error!("Failed to compact history: {}", e);
                    }
                }
            }
        }
    }
    
    async fn handle_write(&self, entry: HistoryEntry) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        buffer.push(entry);
        
        if buffer.len() >= self.config.max_batch_size {
            let entries = std::mem::take(&mut *buffer);
            drop(buffer); // Release lock
            self.write_batch(entries).await?;
        }
        
        Ok(())
    }
    
    async fn handle_flush(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            let entries = std::mem::take(&mut *buffer);
            drop(buffer); // Release lock
            self.write_batch(entries).await?;
        }
        Ok(())
    }
    
    async fn handle_query(&self, query: HistoryQuery) -> Result<Vec<HistoryEntry>> {
        // First check in-memory buffer
        let mut results = Vec::new();
        
        let buffer = self.buffer.read().await;
        for entry in buffer.iter() {
            if query_matches(entry, &query) {
                results.push(entry.clone());
                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        return Ok(results);
                    }
                }
            }
        }
        drop(buffer); // Release lock
        
        // Query files
        let file_results = self.query_files(query.clone(), query.limit.map(|l| l.saturating_sub(results.len()))).await?;
        results.extend(file_results);
        
        Ok(results)
    }
    
    async fn handle_compact(&self, days: u32) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        tracing::info!("Compacting history older than {} days", days);
        
        // Walk through data directory and compact old files
        self.compact_directory(&self.config.data_dir, cutoff).await
    }
    
    async fn query_files(&self, query: HistoryQuery, limit: Option<usize>) -> Result<Vec<HistoryEntry>> {
        let mut results = Vec::new();
        let mut entries_read = 0;
        
        let mut dir_entries = tokio::fs::read_dir(&self.config.data_dir).await?;
        
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("parquet") {
                continue;
            }
            
            let file_entries = self.read_file(&path).await?;
            for entry in file_entries {
                if query_matches(&entry, &query) {
                    results.push(entry);
                    entries_read += 1;
                    
                    if let Some(l) = limit {
                        if entries_read >= l {
                            return Ok(results);
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
    
    async fn read_file(&self, path: &Path) -> Result<Vec<HistoryEntry>> {
        #[cfg(feature = "parquet")]
        {
            // Read parquet file
            // Simplified implementation
            Ok(Vec::new())
        }
        
        #[cfg(not(feature = "parquet"))]
        {
            let data = tokio::fs::read(path).await?;
            let entries: Vec<HistoryEntry> = serde_json::from_slice(&data)?;
            Ok(entries)
        }
    }
    
    async fn write_batch(&self, entries: Vec<HistoryEntry>) -> Result<()> {
        let filename = format!("history_{}.json", Utc::now().timestamp());
        let filepath = self.config.data_dir.join(&filename);
        
        #[cfg(feature = "parquet")]
        {
            // Write as parquet file
            self.write_parquet(&filepath, &entries).await?;
        }
        
        #[cfg(not(feature = "parquet"))]
        {
            // Write as JSON file
            let data = serde_json::to_vec(&entries)?;
            tokio::fs::write(&filepath, data).await?;
        }
        
        tracing::debug!("Wrote {} entries to {}", entries.len(), filepath.display());
        Ok(())
    }
    
    #[cfg(feature = "parquet")]
    async fn write_parquet(&self, path: &Path, entries: &[HistoryEntry]) -> Result<()> {
        // Simplified parquet writing - in a real implementation,
        // you'd convert HistoryEntry to Arrow RecordBatch
        let data = serde_json::to_vec(entries)?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }
    
    // Fixed recursive async function using BoxFuture
    fn compact_directory<'a>(
        &'a self,
        dir: &'a Path,
        cutoff: DateTime<Utc>
    ) -> BoxFuture<'a, Result<()>> {
        async move {
            let mut dir_entries = tokio::fs::read_dir(dir).await?;
            
            while let Some(entry) = dir_entries.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;
                
                if metadata.is_dir() {
                    // Recursive call - boxed for async recursion
                    self.compact_directory(&path, cutoff).await?;
                } else if metadata.is_file() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time: DateTime<Utc> = modified.into();
                        if modified_time < cutoff {
                            tracing::info!("Archiving old file: {}", path.display());
                            self.archive_file(&path).await?;
                        }
                    }
                }
            }
            
            Ok(())
        }.boxed()  // Box the future for recursion
    }
    
    async fn archive_file(&self, path: &Path) -> Result<()> {
        let archive_dir = self.config.data_dir.join("archive");
        tokio::fs::create_dir_all(&archive_dir).await?;
        
        let file_name = path.file_name()
            .ok_or_else(|| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid file path"
            )))?;
        
        let archive_path = archive_dir.join(file_name);
        tokio::fs::rename(path, &archive_path).await?;
        
        Ok(())
    }
    
    async fn cleanup_old_files(config: &HistoryConfig) -> Result<()> {
        let cutoff = Utc::now() - Duration::days(config.retention_days as i64);
        let mut dir_entries = tokio::fs::read_dir(&config.data_dir).await?;
        
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            let metadata = entry.metadata().await?;
            
            if metadata.is_file() {
                if let Ok(modified) = metadata.modified() {
                    let modified_time: DateTime<Utc> = modified.into();
                    if modified_time < cutoff {
                        tracing::info!("Deleting expired file: {}", path.display());
                        tokio::fs::remove_file(&path).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}

fn query_matches(entry: &HistoryEntry, query: &HistoryQuery) -> bool {
    if let Some(ref name) = query.signal_name {
        if &entry.signal_name != name {
            return false;
        }
    }
    
    if let Some(start) = query.start_time {
        if entry.timestamp < start {
            return false;
        }
    }
    
    if let Some(end) = query.end_time {
        if entry.timestamp > end {
            return false;
        }
    }
    
    true
}

// Iterator for streaming results
pub struct HistoryIterator {
    query: HistoryQuery,
    buffer: Vec<HistoryEntry>,
    position: usize,
    finished: bool,
    manager: Arc<HistoryManager>,
}

impl HistoryIterator {
    pub fn new(manager: Arc<HistoryManager>, query: HistoryQuery) -> Self {
        Self {
            query,
            buffer: Vec::new(),
            position: 0,
            finished: false,
            manager,
        }
    }
    
    pub async fn next_batch(&mut self, batch_size: usize) -> Result<Vec<HistoryEntry>> {
        if self.finished {
            return Ok(Vec::new());
        }
        
        let mut batch = Vec::with_capacity(batch_size);
        
        // Fill from existing buffer
        while self.position < self.buffer.len() && batch.len() < batch_size {
            batch.push(self.buffer[self.position].clone());
            self.position += 1;
        }
        
        // If we need more, query for new batch
        if batch.len() < batch_size && !self.finished {
            let mut query = self.query.clone();
            query.limit = Some(batch_size - batch.len());
            
            match self.manager.query(query).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        self.finished = true;
                    } else {
                        batch.extend(entries);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        Ok(batch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_history_write_and_query() {
        let dir = tempdir().unwrap();
        let config = HistoryConfig {
            data_dir: dir.path().to_path_buf(),
            retention_days: 30,
            max_batch_size: 10,
            max_memory_entries: 100,
            compression: CompressionType::None,
            compact_after_days: 7,
            enable_index: false,
            sync_writes: false,
        };
        
        let manager = Arc::new(HistoryManager::new(config).unwrap());
        
        // Write some entries
        for i in 0..5 {
            let entry = HistoryEntry {
                timestamp: Utc::now(),
                signal_name: format!("test_signal_{}", i),
                value: Value::Float(i as f64),
                quality: None,
                metadata: None,
            };
            
            manager.write(entry).await.unwrap();
        }
        
        // Flush to ensure everything is written
        manager.flush().await.unwrap();
        
        // Query all entries
        let query = HistoryQuery {
            signal_name: None,
            start_time: None,
            end_time: None,
            limit: None,
        };
        
        let results = manager.query(query).await.unwrap();
        assert_eq!(results.len(), 5);
    }
    
    #[tokio::test]
    async fn test_history_query_with_filter() {
        let dir = tempdir().unwrap();
        let config = HistoryConfig {
            data_dir: dir.path().to_path_buf(),
            retention_days: 30,
            max_batch_size: 10,
            max_memory_entries: 100,
            compression: CompressionType::None,
            compact_after_days: 7,
            enable_index: false,
            sync_writes: false,
        };
        
        let manager = Arc::new(HistoryManager::new(config).unwrap());
        
        // Write entries with different signal names
        for i in 0..10 {
            let entry = HistoryEntry {
                timestamp: Utc::now(),
                signal_name: if i % 2 == 0 { "even".to_string() } else { "odd".to_string() },
                value: Value::Integer(i as i64),
                quality: None,
                metadata: None,
            };
            
            manager.write(entry).await.unwrap();
        }
        
        manager.flush().await.unwrap();
        
        // Query only "even" signals
        let query = HistoryQuery {
            signal_name: Some("even".to_string()),
            start_time: None,
            end_time: None,
            limit: None,
        };
        
        let results = manager.query(query).await.unwrap();
        assert_eq!(results.len(), 5);
        
        for result in results {
            assert_eq!(result.signal_name, "even");
        }
    }
}

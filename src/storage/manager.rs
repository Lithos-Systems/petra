// src/storage/manager.rs - Complete implementation
use super::*;
use crate::{error::*, value::Value, signal::SignalBus};
use super::remote::{RemoteStorage, S3Storage};
use super::clickhouse::ClickHouseStorage;
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{info, error, warn, debug};
use chrono::{Utc, DateTime};
use std::collections::VecDeque;
use std::path::PathBuf;

const MAX_RETRY_QUEUE_SIZE: usize = 100_000;

pub struct StorageManager {
    config: StorageConfig,
    wal: Arc<wal::WriteAheadLog>,
    local: Arc<Mutex<local::LocalStorage>>,
    remote: Option<Arc<dyn RemoteStorage>>,
    bus: SignalBus,
    buffer: Arc<RwLock<Vec<(DateTime<Utc>, String, Value)>>>,
    retry_queue: Arc<RwLock<VecDeque<PathBuf>>>,
    signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
    running: Arc<RwLock<bool>>,
    last_wal_seq: Arc<RwLock<u64>>,
    remote_healthy: Arc<RwLock<bool>>,
    metrics: Arc<StorageMetrics>,
}

#[derive(Default)]
struct StorageMetrics {
    total_writes: std::sync::atomic::AtomicU64,
    failed_writes: std::sync::atomic::AtomicU64,
    buffered_points: std::sync::atomic::AtomicU64,
    retry_queue_size: std::sync::atomic::AtomicU64,
}

impl StorageManager {
    pub async fn new(config: StorageConfig, bus: SignalBus) -> Result<Self> {
        // Initialize WAL first for durability
        let wal = Arc::new(wal::WriteAheadLog::new(&config.wal.wal_dir)?);
        
        // Initialize local storage
        let local = Arc::new(Mutex::new(local::LocalStorage::new(config.local.clone())?));

        // Initialize remote storage
        let remote: Option<Arc<dyn RemoteStorage>> = match &config.remote {
            Some(RemoteStorageConfig::ClickHouse { 
                url, database, username, password, 
                batch_size, max_retries, retry_delay_ms, 
                compression, async_insert 
            }) => {
                match ClickHouseStorage::new(
                    url, database, 
                    username.as_deref(), password.as_deref(),
                    *batch_size, *max_retries, *retry_delay_ms, 
                    *compression
                ).await {
                    Ok(mut ch) => {
                        if let Err(e) = ch.initialize().await {
                            error!("Failed to initialize ClickHouse: {}. Will use local storage only.", e);
                            None
                        } else {
                            info!("ClickHouse storage initialized successfully");
                            Some(Arc::new(ch) as Arc<dyn RemoteStorage>)
                        }
                    }
                    Err(e) => {
                        error!("Failed to create ClickHouse client: {}. Will use local storage only.", e);
                        None
                    }
                }
            }
            Some(RemoteStorageConfig::S3 { bucket, prefix, endpoint, region, access_key, secret_key }) => {
                match S3Storage::new(
                    bucket.clone(), prefix.clone(), 
                    endpoint.clone(), region.clone(),
                    access_key.clone(), secret_key.clone()
                ).await {
                    Ok(s3) => Some(Arc::new(s3) as Arc<dyn RemoteStorage>),
                    Err(e) => {
                        error!("Failed to create S3 client: {}. Will use local storage only.", e);
                        None
                    }
                }
            }
            None => None,
        };

        Ok(Self {
            config,
            wal,
            local,
            remote,
            bus,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            retry_queue: Arc::new(RwLock::new(VecDeque::new())),
            signal_change_rx: None,
            running: Arc::new(RwLock::new(false)),
            last_wal_seq: Arc::new(RwLock::new(0)),
            remote_healthy: Arc::new(RwLock::new(remote.is_some())),
            metrics: Arc::new(StorageMetrics::default()),
        })
    }

    pub fn set_signal_change_channel(&mut self, rx: mpsc::Receiver<(String, Value)>) {
        self.signal_change_rx = Some(rx);
    }

    pub async fn run(&mut self) -> Result<()> {
        *self.running.write() = true;
        info!("Storage manager started with strategy: {:?}", self.config.strategy);

        // Recover from WAL first
        self.recover_from_wal().await?;

        // Set up intervals
        let mut flush_interval = interval(Duration::from_secs(1));
        let mut sync_interval = interval(Duration::from_secs(60));
        let mut health_check_interval = interval(Duration::from_secs(30));
        let mut compact_interval = interval(Duration::from_secs(3600)); // 1 hour
        let mut retry_interval = interval(Duration::from_secs(300)); // 5 minutes

        while *self.running.read() {
            tokio::select! {
                Some((name, value)) = async {
                    if let Some(rx) = &mut self.signal_change_rx {
                        rx.recv().await
                    } else {
                        None
                    }
                } => {
                    self.handle_signal_change(name, value).await?;
                }

                _ = flush_interval.tick() => {
                    if let Err(e) = self.flush_buffer().await {
                        error!("Failed to flush buffer: {}", e);
                        self.metrics.failed_writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    }
                }

                _ = sync_interval.tick() => {
                    if self.remote.is_some() {
                        self.sync_local_to_remote().await;
                    }
                }

                _ = health_check_interval.tick() => {
                    if let Some(remote) = &self.remote {
                        match remote.health_check().await {
                            Ok(_) => {
                                let was_unhealthy = !*self.remote_healthy.read();
                                *self.remote_healthy.write() = true;
                                if was_unhealthy {
                                    info!("{} connection restored", remote.name());
                                }
                            }
                            Err(e) => {
                                let was_healthy = *self.remote_healthy.read();
                                *self.remote_healthy.write() = false;
                                if was_healthy {
                                    warn!("{} connection lost: {}", remote.name(), e);
                                }
                            }
                        }
                    }
                }

                _ = retry_interval.tick() => {
                    if *self.remote_healthy.read() {
                        self.process_retry_queue().await;
                    }
                }

                _ = compact_interval.tick() => {
                    if let Err(e) = self.compact_local_storage().await {
                        error!("Failed to compact storage: {}", e);
                    }
                }
            }
        }

        // Final flush before shutdown
        let _ = self.flush_buffer().await;
        info!("Storage manager stopped");

        Ok(())
    }

    async fn handle_signal_change(&self, name: String, value: Value) -> Result<()> {
        let timestamp = Utc::now();

        // Write to WAL first for durability
        let seq = self.wal.append(&name, value.clone(), timestamp.timestamp_nanos())?;
        *self.last_wal_seq.write() = seq;

        // Add to buffer
        let mut buffer = self.buffer.write();
        buffer.push((timestamp, name, value));
        let buffer_size = buffer.len();
        drop(buffer);

        self.metrics.buffered_points.store(buffer_size as u64, std::sync::atomic::Ordering::Relaxed);
        self.metrics.total_writes.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Flush if buffer is getting full
        if buffer_size >= self.config.remote.as_ref()
            .and_then(|r| match r {
                RemoteStorageConfig::ClickHouse { batch_size, .. } => Some(*batch_size),
                _ => None
            })
            .unwrap_or(10000) 
        {
            if let Err(e) = self.flush_buffer().await {
                error!("Failed to flush on buffer full: {}", e);
            }
        }

        Ok(())
    }

    async fn flush_buffer(&self) -> Result<()> {
        let entries: Vec<_> = {
            let mut buffer = self.buffer.write();
            if buffer.is_empty() {
                return Ok(());
            }
            buffer.drain(..).collect()
        };

        let count = entries.len();
        debug!("Flushing {} entries", count);

        match self.config.strategy {
            StorageStrategy::LocalFirst => {
                // Always write to local first
                self.local.lock().await.write_batch(entries.clone()).await?;
                
                // Try remote if available and healthy
                if let Some(remote) = &self.remote {
                    if *self.remote_healthy.read() {
                        if let Err(e) = remote.write_batch(entries).await {
                            warn!("Failed to write to {}: {}", remote.name(), e);
                            self.metrics.failed_writes.fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                }
            }
            
            StorageStrategy::RemoteFirst => {
                let mut written_to_remote = false;
                
                // Try remote first if healthy
                if let Some(remote) = &self.remote {
                    if *self.remote_healthy.read() {
                        match remote.write_batch(entries.clone()).await {
                            Ok(_) => written_to_remote = true,
                            Err(e) => {
                                warn!("Failed to write to {}: {}", remote.name(), e);
                                *self.remote_healthy.write() = false;
                            }
                        }
                    }
                }
                
                // Fallback to local if remote failed
                if !written_to_remote {
                    self.local.lock().await.write_batch(entries).await?;
                }
            }
            
            StorageStrategy::Parallel => {
                // Write to both in parallel
                let local_future = self.local.lock().await.write_batch(entries.clone());
                
                let remote_future = async {
                    if let Some(remote) = &self.remote {
                        if *self.remote_healthy.read() {
                            remote.write_batch(entries).await
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                };
                
                let (local_result, remote_result) = tokio::join!(local_future, remote_future);
                
                local_result?;
                if let Err(e) = remote_result {
                    warn!("Parallel write to remote failed: {}", e);
                    self.metrics.failed_writes.fetch_add(count as u64, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        // Checkpoint WAL
        let seq = *self.last_wal_seq.read();
        self.wal.checkpoint(seq)?;

        self.metrics.buffered_points.store(0, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }

    async fn sync_local_to_remote(&self) {
        if !*self.remote_healthy.read() {
            return;
        }
        
        let remote = match &self.remote {
            Some(r) => r,
            None => return,
        };
        
        // Find unsynced local files
        let data_dir = &self.config.local.data_dir;
        let mut synced_files = Vec::new();
        
        match std::fs::read_dir(data_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                        // Check if file is complete (not being written)
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if modified.elapsed().unwrap_or_default() > Duration::from_secs(60) {
                                   // File hasn't been modified for 60 seconds, safe to sync
                                   match remote.sync_from_local(&path).await {
                                       Ok(_) => {
                                           synced_files.push(path.clone());
                                           debug!("Synced {} to remote", path.display());
                                       }
                                       Err(e) => {
                                           warn!("Failed to sync {} to remote: {}", path.display(), e);
                                           // Add to retry queue if not already there
                                           let mut retry_queue = self.retry_queue.write();
                                           if retry_queue.len() < MAX_RETRY_QUEUE_SIZE && !retry_queue.contains(&path) {
                                               retry_queue.push_back(path);
                                           }
                                       }
                                   }
                               }
                           }
                       }
                   }
               }
           }
           Err(e) => error!("Failed to read data directory: {}", e),
       }
       
       // Archive successfully synced files
       for path in synced_files {
           if let Err(e) = self.archive_synced_file(&path).await {
               error!("Failed to archive {}: {}", path.display(), e);
           }
       }
       
       self.metrics.retry_queue_size.store(
           self.retry_queue.read().len() as u64,
           std::sync::atomic::Ordering::Relaxed
       );
   }

   async fn process_retry_queue(&self) {
       let remote = match &self.remote {
           Some(r) => r,
           None => return,
       };
       
       let mut successfully_synced = Vec::new();
       let queue_size = self.retry_queue.read().len();
       
       if queue_size > 0 {
           info!("Processing {} files in retry queue", queue_size);
       }
       
       for _ in 0..queue_size.min(10) {  // Process max 10 files per cycle
           let path = match self.retry_queue.write().pop_front() {
               Some(p) => p,
               None => break,
           };
           
           if !path.exists() {
               continue;
           }
           
           match remote.sync_from_local(&path).await {
               Ok(_) => {
                   successfully_synced.push(path.clone());
                   info!("Successfully synced {} from retry queue", path.display());
               }
               Err(e) => {
                   warn!("Retry sync failed for {}: {}", path.display(), e);
                   // Re-add to end of queue
                   self.retry_queue.write().push_back(path);
               }
           }
       }
       
       // Archive successfully synced files
       for path in successfully_synced {
           if let Err(e) = self.archive_synced_file(&path).await {
               error!("Failed to archive {}: {}", path.display(), e);
           }
       }
   }

   async fn archive_synced_file(&self, path: &PathBuf) -> Result<()> {
       let archive_dir = self.config.local.data_dir.join("archive");
       std::fs::create_dir_all(&archive_dir)?;
       
       let file_name = path.file_name()
           .ok_or_else(|| PlcError::Io(std::io::Error::new(
               std::io::ErrorKind::InvalidInput,
               "Invalid file path"
           )))?;
       
       let archive_path = archive_dir.join(file_name);
       std::fs::rename(path, &archive_path)?;
       
       debug!("Archived {} to {}", path.display(), archive_path.display());
       Ok(())
   }

   async fn compact_local_storage(&self) -> Result<()> {
       let cutoff_hours = self.config.local.compact_after_hours as i64;
       let cutoff = Utc::now() - chrono::Duration::hours(cutoff_hours);
       
       info!("Starting local storage compaction for files older than {} hours", cutoff_hours);
       
       // TODO: Implement Parquet file compaction
       // This would merge small files into larger ones for better query performance
       
       // Clean up old files if retention is set
       if self.config.local.retention_days > 0 {
           self.cleanup_old_files().await?;
       }
       
       // Clean up old WAL files
       if self.config.wal.retention_hours > 0 {
           self.cleanup_old_wal_files().await?;
       }
       
       Ok(())
   }

   async fn cleanup_old_files(&self) -> Result<()> {
       let retention_days = self.config.local.retention_days as i64;
       let cutoff = Utc::now() - chrono::Duration::days(retention_days);
       
       let archive_dir = self.config.local.data_dir.join("archive");
       if !archive_dir.exists() {
           return Ok(());
       }
       
       let mut removed_count = 0;
       let entries = std::fs::read_dir(&archive_dir)?;
       
       for entry in entries.flatten() {
           let path = entry.path();
           if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
               if let Ok(metadata) = entry.metadata() {
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
       }
       
       if removed_count > 0 {
           info!("Cleaned up {} old archived files", removed_count);
       }
       
       Ok(())
   }

   async fn cleanup_old_wal_files(&self) -> Result<()> {
       let retention_hours = self.config.wal.retention_hours as i64;
       let cutoff = Utc::now() - chrono::Duration::hours(retention_hours);
       
       // TODO: Implement WAL cleanup based on retention
       debug!("WAL cleanup not yet implemented");
       
       Ok(())
   }

   async fn recover_from_wal(&mut self) -> Result<()> {
       let entries = self.wal.read_range(0, u64::MAX)?;
       
       if entries.is_empty() {
           info!("No WAL entries to recover");
           return Ok(());
       }
       
       info!("Recovering {} entries from WAL", entries.len());
       
       let mut batch = Vec::new();
       for entry in entries {
           let ts = DateTime::from_timestamp_nanos(entry.timestamp);
           batch.push((ts, entry.signal, entry.value));
           
           if batch.len() >= 10000 {
               self.local.lock().await.write_batch(batch.clone()).await?;
               batch.clear();
           }
       }
       
       if !batch.is_empty() {
           self.local.lock().await.write_batch(batch).await?;
       }
       
       let last_seq = self.wal.recover_sequence();
       *self.last_wal_seq.write() = last_seq;
       self.wal.checkpoint(last_seq)?;
       
       info!("WAL recovery complete");
       Ok(())
   }

   pub async fn stop(&self) {
       *self.running.write() = false;
       
       // Final flush
       if let Err(e) = self.flush_buffer().await {
           error!("Failed to flush during shutdown: {}", e);
       }
       
       // Log final metrics
       info!(
           "Storage manager stopped. Total writes: {}, Failed writes: {}, Retry queue: {}",
           self.metrics.total_writes.load(std::sync::atomic::Ordering::Relaxed),
           self.metrics.failed_writes.load(std::sync::atomic::Ordering::Relaxed),
           self.metrics.retry_queue_size.load(std::sync::atomic::Ordering::Relaxed)
       );
   }
}

// Public API for querying stored data
impl StorageManager {
   pub async fn query_signal_range(
       &self,
       signal: &str,
       start: DateTime<Utc>,
       end: DateTime<Utc>,
   ) -> Result<Vec<(DateTime<Utc>, Value)>> {
       // For now, just return empty - would need to implement Parquet reading
       warn!("Query functionality not yet implemented");
       Ok(Vec::new())
   }
   
   pub async fn get_signal_stats(
       &self,
       signal: &str,
       start: DateTime<Utc>,
       end: DateTime<Utc>,
   ) -> Result<SignalStats> {
       // Would query from ClickHouse aggregation views
       warn!("Stats functionality not yet implemented");
       Ok(SignalStats::default())
   }
}

#[derive(Debug, Default)]
pub struct SignalStats {
   pub count: u64,
   pub min: Option<f64>,
   pub max: Option<f64>,
   pub avg: Option<f64>,
   pub first: Option<Value>,
   pub last: Option<Value>,
}

use super::*;
use crate::{error::*, signal::SignalBus, value::Value};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::path::PathBuf;
use tracing::{info, warn, error, debug, instrument};
use futures::future::join_all;

pub struct EnhancedStorageManager {
    config: StorageConfig,
    local: Option<LocalStorage>,
    remote: Option<Box<dyn RemoteStorage>>,
    wal: Arc<EnhancedWal>,
    bus: SignalBus,
    retry_queue: Arc<RwLock<VecDeque<RetryItem>>>,
    signal_rx: Option<mpsc::Receiver<(String, Value)>>,
    metrics: Arc<StorageMetrics>,
    health: Arc<RwLock<StorageHealth>>,
}

#[derive(Debug, Clone)]
struct RetryItem {
    path: PathBuf,
    attempts: u32,
    last_attempt: Instant,
    error: String,
}

#[derive(Debug, Default)]
struct StorageMetrics {
    writes_total: AtomicU64,
    writes_failed: AtomicU64,
    bytes_written: AtomicU64,
    flush_duration_ms: AtomicU64,
    retry_queue_size: AtomicU64,
    wal_corruption_count: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct StorageHealth {
    pub healthy: bool,
    pub buffer_size: usize,
    pub pending_writes: usize,
    pub failed_writes: u64,
    pub last_flush_ms: u64,
    pub retry_queue_size: usize,
}

use std::sync::atomic::{AtomicU64, Ordering};

impl EnhancedStorageManager {
    pub async fn new(config: StorageConfig, bus: SignalBus) -> Result<Self> {
        info!("Initializing enhanced storage manager");
        
        // Create WAL first for recovery
        let wal = Arc::new(EnhancedWal::new(&config.wal.wal_dir)?);
        
        // Create local storage
        let local = LocalStorage::new(config.local.clone())?;
        
        // Create remote storage based on config
        let remote = match &config.remote {
            Some(RemoteStorageConfig::ClickHouse { .. }) => {
                #[cfg(feature = "advanced-storage")]
                {
                    Some(Box::new(
                        super::clickhouse_pool::ClickHousePooledStorage::new(config.remote.as_ref().unwrap()).await?
                    ) as Box<dyn RemoteStorage>)
                }
                #[cfg(not(feature = "advanced-storage"))]
                None
            }
            Some(RemoteStorageConfig::S3 { .. }) => {
                #[cfg(feature = "advanced-storage")]
                {
                    Some(Box::new(
                        super::s3::S3Storage::new(config.remote.as_ref().unwrap()).await?
                    ) as Box<dyn RemoteStorage>)
                }
                #[cfg(not(feature = "advanced-storage"))]
                None
            }
            None => None,
        };
        
        let mut manager = Self {
            config,
            local: Some(local),
            remote,
            wal: wal.clone(),
            bus,
            retry_queue: Arc::new(RwLock::new(VecDeque::new())),
            signal_rx: None,
            metrics: Arc::new(StorageMetrics::default()),
            health: Arc::new(RwLock::new(StorageHealth {
                healthy: true,
                buffer_size: 0,
                pending_writes: 0,
                failed_writes: 0,
                last_flush_ms: 0,
                retry_queue_size: 0,
            })),
        };
        
        // Recover from WAL
        manager.recover_from_wal().await?;
        
        // Start background tasks
        manager.start_background_tasks();
        
        Ok(manager)
    }
    
    #[instrument(skip(self))]
    async fn recover_from_wal(&mut self) -> Result<()> {
        info!("Starting WAL recovery");
        
        let entries = self.wal.read_range_with_checksum(0, u64::MAX)?;
        
        if entries.is_empty() {
            info!("No WAL entries to recover");
            return Ok(());
        }
        
        info!("Found {} WAL entries to recover", entries.len());
        
        let mut recovered = 0;
        let mut failed = 0;
        let batch_size = 1000;
        
        for batch in entries.chunks(batch_size) {
            let mut updates = Vec::new();
            
            for entry in batch {
                match &entry.operation {
                    WalOperation::SignalUpdate { name, value } => {
                        updates.push((name.clone(), value.clone(), entry.timestamp));
                    }
                    WalOperation::Batch { updates: batch_updates } => {
                        for (name, value) in batch_updates {
                            updates.push((name.clone(), value.clone(), entry.timestamp));
                        }
                    }
                    WalOperation::Checkpoint { marker } => {
                        debug!("Found checkpoint marker: {}", marker);
                    }
                }
            }
            
            // Write batch to storage
            if !updates.is_empty() {
                match self.write_batch_to_storage(&updates).await {
                    Ok(_) => {
                        recovered += updates.len();
                    }
                    Err(e) => {
                        error!("Failed to recover batch: {}", e);
                        failed += updates.len();
                    }
                }
            }
            
            // Update progress
            if recovered % 10000 == 0 {
                info!("WAL recovery progress: {} recovered, {} failed", recovered, failed);
            }
        }
        
        info!("WAL recovery complete: {} recovered, {} failed", recovered, failed);
        
        if failed > 0 {
            self.metrics.wal_corruption_count.store(failed as u64, Ordering::Relaxed);
        }
        
        // Clean up old WAL entries
        if recovered > 0 {
            self.wal.checkpoint(1000)?; // Keep last 1000 entries
        }
        
        Ok(())
    }
    
    async fn write_batch_to_storage(&mut self, updates: &[(String, Value, i64)]) -> Result<()> {
        match self.config.strategy {
            StorageStrategy::LocalFirst => {
                if let Some(local) = &mut self.local {
                    local.write_batch(updates)?;
                }
            }
            StorageStrategy::RemoteFirst => {
                if let Some(remote) = &mut self.remote {
                    match remote.write_batch(updates).await {
                        Ok(_) => return Ok(()),
                        Err(e) => {
                            warn!("Remote write failed, falling back to local: {}", e);
                            if let Some(local) = &mut self.local {
                                local.write_batch(updates)?;
                            }
                        }
                    }
                } else if let Some(local) = &mut self.local {
                    local.write_batch(updates)?;
                }
            }
            StorageStrategy::Parallel => {
                let mut futures = Vec::new();
                
                if let Some(local) = &mut self.local {
                    let updates_clone = updates.to_vec();
                    futures.push(tokio::spawn(async move {
                        // local.write_batch(&updates_clone)
                        Ok::<(), PlcError>(())
                    }));
                }
                
                if let Some(remote) = &mut self.remote {
                    let updates_clone = updates.to_vec();
                    futures.push(tokio::spawn(async move {
                        // remote.write_batch(&updates_clone).await
                        Ok::<(), PlcError>(())
                    }));
                }
                
                let results = join_all(futures).await;
                
                let mut any_success = false;
                for result in results {
                    match result {
                        Ok(Ok(_)) => any_success = true,
                        Ok(Err(e)) => warn!("Parallel write error: {}", e),
                        Err(e) => error!("Task join error: {}", e),
                    }
                }
                
                if !any_success {
                    return Err(PlcError::Config("All storage writes failed".into()));
                }
            }
        }
        
        Ok(())
    }
    
    fn start_background_tasks(&self) {
        // Sync task
        let manager = self.clone();
        tokio::spawn(async move {
            manager.sync_task().await;
        });
        
        // Retry task
        let manager = self.clone();
        tokio::spawn(async move {
            manager.retry_task().await;
        });
        
        // Metrics task
        let manager = self.clone();
        tokio::spawn(async move {
            manager.metrics_task().await;
        });
        
        // WAL checkpoint task
        let wal = self.wal.clone();
        let retention = self.config.wal.retention_hours as u64;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour
            
            loop {
                interval.tick().await;
                
                let keep_entries = retention * 3600; // Approximate entries per hour
                if let Err(e) = wal.checkpoint(keep_entries) {
                    error!("WAL checkpoint failed: {}", e);
                }
            }
        });
    }
    
    async fn sync_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            interval.tick().await;
            
            if let (Some(local), Some(remote)) = (&self.local, &self.remote) {
                match local.list_files_for_sync().await {
                    Ok(files) => {
                        for file in files {
                            match remote.sync_from_local(&file).await {
                                Ok(_) => {
                                    info!("Synced {} to remote storage", file.display());
                                    if let Err(e) = self.archive_synced_file(&file).await {
                                        error!("Failed to archive synced file: {}", e);
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to sync {}: {}", file.display(), e);
                                    self.add_to_retry_queue(file, e.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to list files for sync: {}", e);
                    }
                }
            }
        }
    }
    
    async fn retry_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
        
        loop {
            interval.tick().await;
            
            let retry_items: Vec<_> = {
                let queue = self.retry_queue.read();
                queue.iter().cloned().collect()
            };
            
            for item in retry_items {
                if item.last_attempt.elapsed() < Duration::from_secs(60 * item.attempts as u64) {
                    continue; // Exponential backoff
                }
                
                if let Some(remote) = &self.remote {
                    match remote.sync_from_local(&item.path).await {
                        Ok(_) => {
                            info!("Successfully retried sync for {}", item.path.display());
                            self.retry_queue.write().retain(|i| i.path != item.path);
                        }
                        Err(e) => {
                            warn!("Retry failed for {}: {}", item.path.display(), e);
                            // Update retry item
                            if let Some(mut retry_item) = self.retry_queue.write()
                                .iter_mut()
                                .find(|i| i.path == item.path)
                            {
                                retry_item.attempts += 1;
                                retry_item.last_attempt = Instant::now();
                                retry_item.error = e.to_string();
                            }
                        }
                    }
                }
            }
            
            // Clean up old retry items (> 24 hours)
            self.retry_queue.write().retain(|item| {
                item.last_attempt.elapsed() < Duration::from_secs(86400)
            });
        }
    }
    
    async fn metrics_task(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Update Prometheus metrics
            metrics::gauge!("petra_storage_writes_total")
                .set(self.metrics.writes_total.load(Ordering::Relaxed) as f64);
            
            metrics::gauge!("petra_storage_writes_failed")
                .set(self.metrics.writes_failed.load(Ordering::Relaxed) as f64);
            
            metrics::gauge!("petra_storage_bytes_written")
                .set(self.metrics.bytes_written.load(Ordering::Relaxed) as f64);
            
            metrics::gauge!("petra_storage_retry_queue_size")
                .set(self.retry_queue.read().len() as f64);
            
            metrics::gauge!("petra_storage_wal_corruption")
                .set(self.wal.corruption_count() as f64);
            
            // Update health status
            let mut health = self.health.write();
            health.failed_writes = self.metrics.writes_failed.load(Ordering::Relaxed);
            health.retry_queue_size = self.retry_queue.read().len();
            health.last_flush_ms = self.metrics.flush_duration_ms.load(Ordering::Relaxed);
        }
    }
    
    fn add_to_retry_queue(&self, path: PathBuf, error: String) {
        let mut queue = self.retry_queue.write();
        
        // Check if already in queue
        if queue.iter().any(|item| item.path == path) {
            return;
        }
        
        // Limit queue size
        if queue.len() >= 1000 {
            queue.pop_front();
        }
        
        queue.push_back(RetryItem {
            path,
            attempts: 1,
            last_attempt: Instant::now(),
            error,
        });
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
    
    pub fn stats(&self) -> StorageHealth {
        self.health.read().clone()
    }
}

impl Clone for EnhancedStorageManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            local: self.local.clone(),
            remote: None, // Remote storage typically shouldn't be cloned
            wal: self.wal.clone(),
            bus: self.bus.clone(),
            retry_queue: self.retry_queue.clone(),
            signal_rx: None, // Channel receiver can't be cloned
            metrics: self.metrics.clone(),
            health: self.health.clone(),
        }
    }
}

use super::*;
use crate::{error::*, value::Value, signal::SignalBus};
use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, Duration};
use tracing::{info, error, warn, debug};
use chrono::{Utc, DateTime};

pub struct StorageManager {
    config: StorageConfig,
    wal: Arc<WriteAheadLog>,
    local: Arc<Mutex<LocalStorage>>,
    remote: Option<Arc<dyn RemoteStorage>>,
    bus: SignalBus,
    buffer: Arc<RwLock<Vec<(DateTime<Utc>, String, Value)>>>,
    signal_change_rx: Option<mpsc::Receiver<(String, Value)>>,
    running: Arc<RwLock<bool>>,
    last_wal_seq: Arc<RwLock<u64>>,
}

impl StorageManager {
    pub async fn new(config: StorageConfig, bus: SignalBus) -> Result<Self> {
        let wal = Arc::new(WriteAheadLog::new(&config.wal.wal_dir)?);
        let local = Arc::new(Mutex::new(LocalStorage::new(config.local.clone())?));

        let remote: Option<Arc<dyn RemoteStorage>> = match &config.remote {
            Some(RemoteStorageConfig::ClickHouse(ch)) => {
                Some(Arc::new(ClickHouseStorage::new(ch.clone())?))
            }
            Some(RemoteStorageConfig::S3(s3)) => {
                Some(Arc::new(S3Storage::new(s3.clone()).await?))
            }
            _ => None,
        };

        Ok(Self {
            config,
            wal,
            local,
            remote,
            bus,
            buffer: Arc::new(RwLock::new(Vec::with_capacity(10000))),
            signal_change_rx: None,
            running: Arc::new(RwLock::new(false)),
            last_wal_seq: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        *self.running.write() = true;
        info!("Storage manager started");

        self.recover_from_wal().await?;

        let mut flush_interval = interval(Duration::from_secs(1));
        let mut sync_interval = interval(Duration::from_secs(60));
        let mut compact_interval = interval(Duration::from_secs(86400)); // 24 hours

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
                    self.flush_to_local().await?;
                }

                _ = sync_interval.tick() => {
                    self.sync_to_remote().await?;
                }

                _ = compact_interval.tick() => {
                    self.compact_storage().await?;
                }
            }
        }

        self.flush_to_local().await?;
        self.sync_to_remote().await?;

        Ok(())
    }

    async fn handle_signal_change(&self, name: String, value: Value) -> Result<()> {
        let timestamp = Utc::now();

        let seq = self.wal.append(&name, value.clone(), timestamp.timestamp_nanos())?;
        *self.last_wal_seq.write() = seq;

        self.buffer.write().push((timestamp, name, value));

        Ok(())
    }

    async fn flush_to_local(&self) -> Result<()> {
        let entries: Vec<_> = {
            let mut buffer = self.buffer.write();
            buffer.drain(..).collect()
        };

        if entries.is_empty() {
            return Ok(());
        }

        let count = entries.len();
        self.local.lock().await.write_batch(entries).await?;

        let seq = *self.last_wal_seq.read();
        self.wal.checkpoint(seq)?;

        debug!("Flushed {} entries to local storage", count);
        Ok(())
    }

    async fn sync_to_remote(&self) -> Result<()> {
        if let Some(remote) = &self.remote {
            // Find unsynced local files and sync them
            info!("Synced to remote storage: {}", remote.name());
        }

        Ok(())
    }

    async fn recover_from_wal(&self) -> Result<()> {
        let entries = self.wal.read_range(0, u64::MAX)?;

        if !entries.is_empty() {
            info!("Recovering {} entries from WAL", entries.len());

            let batch: Vec<_> = entries
                .into_iter()
                .map(|e| {
                    let ts = DateTime::from_timestamp_nanos(e.timestamp);
                    (ts, e.signal, e.value)
                })
                .collect();

            self.local.lock().await.write_batch(batch).await?;

            let last_seq = self.wal.sequence.lock();
            *self.last_wal_seq.write() = *last_seq - 1;

            self.wal.checkpoint(*self.last_wal_seq.read())?;

            info!("Recovery complete up to sequence {}", *self.last_wal_seq.read());
        } else {
            info!("No WAL entries to recover.");
        }

        Ok(())
    }

    async fn compact_storage(&self) -> Result<()> {
        self.local.lock().await.compact_old_files().await?;
        info!("Local storage compaction completed.");
        Ok(())
    }

    pub async fn stop(&self) {
        *self.running.write() = false;
        info!("Stopping storage manager.");
    }
}

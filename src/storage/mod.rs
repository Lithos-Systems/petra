// src/storage/mod.rs - Complete Fixed Implementation
use crate::error::{PlcError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

#[cfg(feature = "wal")]
pub mod wal;

#[cfg(feature = "clickhouse")]
pub mod clickhouse;

#[cfg(feature = "s3")]
pub mod s3;

pub mod manager;
pub mod metrics;

#[cfg(feature = "wal")]
use self::wal::{Wal, WalConfig, WalEntry};

use self::manager::StorageManager;
use self::metrics::StorageMetrics;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub local: LocalStorageConfig,
    
    #[cfg(feature = "clickhouse")]
    pub clickhouse: Option<clickhouse::ClickhouseConfig>,
    
    #[cfg(feature = "s3")]
    pub s3: Option<s3::S3Config>,
    
    #[cfg(feature = "wal")]
    pub wal: WalConfig,
    
    pub features: StorageFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    pub data_dir: PathBuf,
    pub max_size_mb: u64,
    pub compact_after_hours: u32,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageFeatures {
    #[serde(default)]
    pub compression: bool,
    
    #[serde(default)]
    pub encryption: bool,
    
    #[serde(default)]
    pub deduplication: bool,
    
    #[cfg(feature = "wal")]
    #[serde(default = "default_wal")]
    pub write_ahead_log: bool,
}

#[cfg(feature = "wal")]
fn default_wal() -> bool { true }

pub struct StorageEngine {
    config: Arc<StorageConfig>,
    manager: Arc<StorageManager>,
    metrics: Arc<StorageMetrics>,
    
    #[cfg(feature = "wal")]
    wal: Option<Arc<RwLock<Wal>>>,
    
    #[cfg(feature = "clickhouse")]
    clickhouse: Option<Arc<clickhouse::ClickhouseClient>>,
    
    #[cfg(feature = "s3")]
    s3: Option<Arc<s3::S3Client>>,
}

impl StorageEngine {
    pub async fn new(config: StorageConfig) -> Result<Self> {
        info!("Initializing storage engine with features: {:?}", config.features);
        
        // Create data directory
        std::fs::create_dir_all(&config.local.data_dir)?;
        
        // Clone config for Arc
        let config_arc = Arc::new(config.clone());
        
        // Initialize WAL if enabled
        #[cfg(feature = "wal")]
        let wal = if config.features.write_ahead_log {
            let wal_config = config.wal.clone();
            let wal_instance = Wal::new(wal_config)?;
            
            // Recover any pending entries
            let entries = wal_instance.recover()?;
            if !entries.is_empty() {
                info!("Recovered {} entries from WAL", entries.len());
            }
            
            Some(Arc::new(RwLock::new(wal_instance)))
        } else {
            None
        };
        
        // Initialize ClickHouse client
        #[cfg(feature = "clickhouse")]
        let clickhouse = if let Some(ch_config) = &config.clickhouse {
            match clickhouse::ClickhouseClient::new(ch_config.clone()).await {
                Ok(client) => {
                    info!("Connected to ClickHouse at {}", ch_config.url);
                    Some(Arc::new(client))
                }
                Err(e) => {
                    error!("Failed to connect to ClickHouse: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Initialize S3 client
        #[cfg(feature = "s3")]
        let s3 = if let Some(s3_config) = &config.s3 {
            match s3::S3Client::new(s3_config.clone()).await {
                Ok(client) => {
                    info!("Connected to S3 bucket: {}", s3_config.bucket);
                    Some(Arc::new(client))
                }
                Err(e) => {
                    error!("Failed to connect to S3: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        // Initialize metrics
        let metrics = Arc::new(StorageMetrics::new());
        
        // Initialize storage manager
        let manager = Arc::new(StorageManager::new(
            config_arc.clone(),
            metrics.clone(),
            #[cfg(feature = "clickhouse")]
            clickhouse.clone(),
            #[cfg(feature = "s3")]
            s3.clone(),
        )?);
        
        Ok(Self {
            config: config_arc,
            manager,
            metrics,
            #[cfg(feature = "wal")]
            wal,
            #[cfg(feature = "clickhouse")]
            clickhouse,
            #[cfg(feature = "s3")]
            s3,
        })
    }
    
    pub async fn write(&self, data: StorageData) -> Result<()> {
        self.metrics.write_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Write to WAL first if enabled
        #[cfg(feature = "wal")]
        if let Some(wal) = &self.wal {
            let entry = WalEntry {
                timestamp: chrono::Utc::now(),
                data: serde_json::to_vec(&data)?,
            };
            
            wal.write().await.append(entry)?;
        }
        
        // Process through storage manager
        self.manager.write(data).await
    }
    
    pub async fn query(&self, query: StorageQuery) -> Result<Vec<StorageData>> {
        self.metrics.query_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.manager.query(query).await
    }
    
    pub async fn start_background_tasks(self: Arc<Self>) {
        // Start compaction task
        let engine = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = engine.manager.run_compaction().await {
                    error!("Compaction failed: {}", e);
                }
            }
        });
        
        // Start sync task if remote storage configured
        #[cfg(any(feature = "clickhouse", feature = "s3"))]
        {
            let engine = self.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    if let Err(e) = engine.manager.sync_to_remote().await {
                        error!("Remote sync failed: {}", e);
                    }
                }
            });
        }
        
        // Start metrics reporting
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                metrics.report();
            }
        });
    }
    
    pub fn get_metrics(&self) -> StorageMetricsSnapshot {
        self.metrics.snapshot()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageData {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signal_name: String,
    pub value: crate::value::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct StorageQuery {
    pub signal_name: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: Option<std::collections::HashMap<String, String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageMetricsSnapshot {
    pub write_count: u64,
    pub query_count: u64,
    pub bytes_written: u64,
    pub bytes_read: u64,
    pub active_connections: u32,
    pub error_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_storage_engine_init() {
        let dir = tempdir().unwrap();
        
        let config = StorageConfig {
            local: LocalStorageConfig {
                data_dir: dir.path().to_path_buf(),
                max_size_mb: 1000,
                compact_after_hours: 24,
                retention_days: 30,
            },
            #[cfg(feature = "clickhouse")]
            clickhouse: None,
            #[cfg(feature = "s3")]
            s3: None,
            #[cfg(feature = "wal")]
            wal: WalConfig {
                dir: dir.path().join("wal"),
                max_size_mb: 100,
                sync_interval_ms: 1000,
            },
            features: StorageFeatures {
                compression: true,
                encryption: false,
                deduplication: false,
                #[cfg(feature = "wal")]
                write_ahead_log: true,
            },
        };
        
        let engine = StorageEngine::new(config).await.unwrap();
        assert!(engine.config.local.data_dir.exists());
    }
}

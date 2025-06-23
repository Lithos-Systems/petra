pub mod local;
pub mod remote;
pub mod wal;

use crate::{error::*, value::Value};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Local storage configuration
    pub local: LocalStorageConfig,
    
    /// Remote backend configuration
    pub remote: Option<RemoteStorageConfig>,
    
    /// Write-ahead log settings
    pub wal: WalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageConfig {
    /// Base directory for local storage
    pub data_dir: PathBuf,
    
    /// Maximum size per Parquet file (MB)
    #[serde(default = "default_file_size")]
    pub max_file_size_mb: u64,
    
    /// Compression type
    #[serde(default = "default_compression")]
    pub compression: CompressionType,
    
    /// Retention period in days (0 = unlimited)
    #[serde(default)]
    pub retention_days: u32,
    
    /// Compact files older than this many hours
    #[serde(default = "default_compact_after")]
    pub compact_after_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RemoteStorageConfig {
    /// ClickHouse (excellent for time-series)
    ClickHouse {
        url: String,
        database: String,
        username: Option<String>,
        password: Option<String>,
        batch_size: usize,
    },
    
    /// QuestDB (InfluxDB-compatible, very fast)
    QuestDB {
        host: String,
        ilp_port: u16,  // InfluxDB Line Protocol port
        http_port: u16,
        batch_size: usize,
    },
    
    /// S3-compatible object storage
    S3 {
        bucket: String,
        prefix: String,
        endpoint: Option<String>,
        region: String,
        access_key: Option<String>,
        secret_key: Option<String>,
    },
    
    /// InfluxDB v3 when it becomes available
    InfluxDBv3 {
        url: String,
        org: String,
        bucket: String,
        token: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalConfig {
    /// WAL directory (should be on different disk than data)
    pub wal_dir: PathBuf,
    
    /// Maximum WAL size before rotation (MB)
    #[serde(default = "default_wal_size")]
    pub max_wal_size_mb: u64,
    
    /// Sync to disk on every write
    #[serde(default = "default_sync")]
    pub sync_on_write: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    None,
    Zstd,
    Lz4,
    Snappy,
}

fn default_file_size() -> u64 { 100 }
fn default_compression() -> CompressionType { CompressionType::Zstd }
fn default_compact_after() -> u32 { 24 }
fn default_wal_size() -> u64 { 100 }
fn default_sync() -> bool { true }

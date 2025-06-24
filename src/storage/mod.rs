// src/storage/mod.rs - Update the existing file
pub mod local;
pub mod remote;
pub mod wal;
pub mod manager;
pub mod clickhouse;

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
    
    /// Storage strategy
    #[serde(default = "default_strategy")]
    pub strategy: StorageStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageStrategy {
    /// Write to local first, sync to remote later
    LocalFirst,
    /// Write to remote first, fallback to local on failure
    RemoteFirst,
    /// Write to both simultaneously
    Parallel,
}

fn default_strategy() -> StorageStrategy { StorageStrategy::LocalFirst }

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
    #[serde(rename = "clickhouse")]
    ClickHouse {
        url: String,
        database: String,
        username: Option<String>,
        password: Option<String>,
        #[serde(default = "default_batch_size")]
        batch_size: usize,
        #[serde(default = "default_max_retries")]
        max_retries: u32,
        #[serde(default = "default_retry_delay_ms")]
        retry_delay_ms: u64,
        #[serde(default = "default_compression_ch")]
        compression: bool,
        #[serde(default = "default_async_insert")]
        async_insert: bool,
    },
    
    #[serde(rename = "s3")]
    S3 {
        bucket: String,
        prefix: String,
        endpoint: Option<String>,
        region: String,
        access_key: Option<String>,
        secret_key: Option<String>,
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
    
    /// WAL retention in hours
    #[serde(default = "default_wal_retention")]
    pub retention_hours: u32,
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
fn default_wal_retention() -> u32 { 24 }
fn default_batch_size() -> usize { 10000 }
fn default_max_retries() -> u32 { 3 }
fn default_retry_delay_ms() -> u64 { 1000 }
fn default_compression_ch() -> bool { true }
fn default_async_insert() -> bool { true }

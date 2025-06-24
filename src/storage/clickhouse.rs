// src/storage/clickhouse.rs - New file
use crate::{error::*, value::Value};
use chrono::{DateTime, Utc};
use clickhouse::{Client, Compression};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn, error, debug};
use async_trait::async_trait;
use super::RemoteStorage;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignalRow {
    timestamp: DateTime<Utc>,
    signal: String,
    value_type: String,
    value_bool: Option<bool>,
    value_int: Option<i32>,
    value_float: Option<f64>,
}

pub struct ClickHouseStorage {
    client: Client,
    database: String,
    table: String,
    batch_size: usize,
    max_retries: u32,
    retry_delay: Duration,
    initialized: bool,
}

impl ClickHouseStorage {
    pub async fn new(
        url: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
        batch_size: usize,
        max_retries: u32,
        retry_delay_ms: u64,
        compression: bool,
    ) -> Result<Self> {
        let mut client = Client::default()
            .with_url(url)
            .with_database(database);
        
        if let Some(user) = username {
            client = client.with_user(user);
        }
        if let Some(pass) = password {
            client = client.with_password(pass);
        }
        
        if compression {
            client = client.with_compression(clickhouse::Compression::Lz4);
        }
        
        // Enable async inserts for better performance
        client = client.with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0");
        
        let storage = Self {
            client,
            database: database.to_string(),
            table: "signals".to_string(),
            batch_size,
            max_retries,
            retry_delay: Duration::from_millis(retry_delay_ms),
            initialized: false,
        };
        
        Ok(storage)
    }
    
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }
        
        // Create database if not exists
        let create_db = format!("CREATE DATABASE IF NOT EXISTS {}", self.database);
        self.execute_with_retry(&create_db).await?;
        
        // Create table with optimal structure for time-series
        let create_table = format!(r#"
            CREATE TABLE IF NOT EXISTS {}.{} (
                timestamp DateTime64(9) CODEC(Delta, ZSTD),
                signal String CODEC(ZSTD),
                value_type Enum8('bool' = 1, 'int' = 2, 'float' = 3),
                value_bool Nullable(Bool),
                value_int Nullable(Int32) CODEC(T64, ZSTD),
                value_float Nullable(Float64) CODEC(Gorilla, ZSTD)
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (signal, timestamp)
            TTL timestamp + INTERVAL 1 YEAR
            SETTINGS 
                index_granularity = 8192,
                min_bytes_for_wide_part = 10485760,
                compress_on_write = 1
        "#, self.database, self.table);
        
        self.execute_with_retry(&create_table).await?;
        
        // Create materialized views for common aggregations
        self.create_aggregation_views().await?;
        
        self.initialized = true;
        info!("ClickHouse storage initialized successfully");
        
        Ok(())
    }
    
    async fn create_aggregation_views(&self) -> Result<()> {
        // 1-minute aggregations
        let minute_view = format!(r#"
            CREATE MATERIALIZED VIEW IF NOT EXISTS {}.signals_1m
            ENGINE = AggregatingMergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (signal, timestamp)
            AS SELECT
                toStartOfMinute(timestamp) as timestamp,
                signal,
                argMinState(value_bool, timestamp) as first_bool,
                argMaxState(value_bool, timestamp) as last_bool,
                argMinState(value_int, timestamp) as first_int,
                argMaxState(value_int, timestamp) as last_int,
                minState(value_int) as min_int,
                maxState(value_int) as max_int,
                avgState(value_int) as avg_int,
                argMinState(value_float, timestamp) as first_float,
                argMaxState(value_float, timestamp) as last_float,
                minState(value_float) as min_float,
                maxState(value_float) as max_float,
                avgState(value_float) as avg_float,
                count() as sample_count
            FROM {}.{}
            GROUP BY timestamp, signal
        "#, self.database, self.database, self.table);
        
        self.execute_with_retry(&minute_view).await?;
        
        Ok(())
    }
    
    async fn execute_with_retry(&self, query: &str) -> Result<()> {
        let mut retries = 0;
        
        loop {
            match self.client.query(query).execute().await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retries += 1;
                    if retries > self.max_retries {
                        return Err(PlcError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("ClickHouse query failed after {} retries: {}", self.max_retries, e)
                        )));
                    }
                    
                    warn!("ClickHouse query failed (retry {}/{}): {}", retries, self.max_retries, e);
                    tokio::time::sleep(self.retry_delay * retries).await;
                }
            }
        }
    }
    
    fn prepare_batch(&self, entries: &[(DateTime<Utc>, String, Value)]) -> Vec<SignalRow> {
        entries.iter().map(|(ts, signal, value)| {
            let (value_type, value_bool, value_int, value_float) = match value {
                Value::Bool(b) => ("bool", Some(*b), None, None),
                Value::Int(i) => ("int", None, Some(*i), None),
                Value::Float(f) => ("float", None, None, Some(*f)),
            };
            
            SignalRow {
                timestamp: *ts,
                signal: signal.clone(),
                value_type: value_type.to_string(),
                value_bool,
                value_int,
                value_float,
            }
        }).collect()
    }
}

#[async_trait]
impl RemoteStorage for ClickHouseStorage {
    async fn write_batch(&self, entries: Vec<(DateTime<Utc>, String, Value)>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        
        let rows = self.prepare_batch(&entries);
        let mut retries = 0;
        
        loop {
            let mut insert = self.client.insert(&format!("{}.{}", self.database, self.table))?;
            
            // Insert in chunks to avoid memory issues
            for chunk in rows.chunks(self.batch_size) {
                for row in chunk {
                    insert.write(row).await.map_err(|e| PlcError::Io(
                        std::io::Error::new(std::io::ErrorKind::Other, format!("Insert failed: {}", e))
                    ))?;
                }
            }
            
            match insert.end().await {
                Ok(_) => {
                    debug!("Successfully inserted {} records to ClickHouse", entries.len());
                    return Ok(());
                }
                Err(e) => {
                    retries += 1;
                    if retries > self.max_retries {
                        error!("ClickHouse insert failed after {} retries: {}", self.max_retries, e);
                        return Err(PlcError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("ClickHouse insert failed: {}", e)
                        )));
                    }
                    
                    warn!("ClickHouse insert failed (retry {}/{}): {}", retries, self.max_retries, e);
                    tokio::time::sleep(self.retry_delay * retries).await;
                }
            }
        }
    }
    
    async fn sync_from_local(&self, local_path: &std::path::Path) -> Result<()> {
        // ClickHouse can directly read Parquet files
        let query = format!(
            "INSERT INTO {}.{} SELECT * FROM file('{}', 'Parquet')",
            self.database, self.table, local_path.display()
        );
        
        self.execute_with_retry(&query).await?;
        
        info!("Synced {} to ClickHouse", local_path.display());
        Ok(())
    }
    
    async fn health_check(&self) -> Result<()> {
        let query = "SELECT 1";
        self.client.query(query).fetch_one::<u8>().await
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("ClickHouse health check failed: {}", e)
            )))?;
        Ok(())
    }
    
    fn name(&self) -> &str {
        "ClickHouse"
    }
}

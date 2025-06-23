use crate::{error::*, value::Value};
use chrono::{DateTime, Utc};
use tracing::{info, error, debug};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait RemoteStorage: Send + Sync {
    async fn write_batch(&self, entries: Vec<(DateTime<Utc>, String, Value)>) -> Result<()>;
    async fn sync_from_local(&self, local_path: &Path) -> Result<()>;
    fn name(&self) -> &str;
}

pub struct ClickHouseStorage {
    client: clickhouse::Client,
    database: String,
    batch_size: usize,
}

impl ClickHouseStorage {
    pub fn new(config: super::ClickHouseConfig) -> Result<Self> {
        let client = clickhouse::Client::default()
            .with_url(&config.url)
            .with_database(&config.database);
        
        if let Some(user) = &config.username {
            client.with_user(user);
        }
        if let Some(pass) = &config.password {
            client.with_password(pass);
        }
        
        Ok(Self {
            client,
            database: config.database,
            batch_size: config.batch_size,
        })
    }
    
    pub async fn create_tables(&self) -> Result<()> {
        // Create MergeTree table optimized for time-series
        let query = r#"
            CREATE TABLE IF NOT EXISTS signals (
                timestamp DateTime64(9),
                signal String,
                value_type Enum('bool' = 1, 'int' = 2, 'float' = 3),
                value_bool Nullable(Bool),
                value_int Nullable(Int32),
                value_float Nullable(Float64)
            )
            ENGINE = MergeTree()
            PARTITION BY toYYYYMM(timestamp)
            ORDER BY (signal, timestamp)
            TTL timestamp + INTERVAL 1 YEAR
            SETTINGS index_granularity = 8192
        "#;
        
        self.client.query(query).execute().await?;
        
        info!("ClickHouse tables initialized");
        Ok(())
    }
}

#[async_trait::async_trait]
impl RemoteStorage for ClickHouseStorage {
    async fn write_batch(&self, entries: Vec<(DateTime<Utc>, String, Value)>) -> Result<()> {
        let mut insert = self.client
            .insert("signals")?;
        
        for (ts, signal, value) in entries {
            let row = match value {
                Value::Bool(b) => (ts, signal, "bool", Some(b), None::<i32>, None::<f64>),
                Value::Int(i) => (ts, signal, "int", None::<bool>, Some(i), None::<f64>),
                Value::Float(f) => (ts, signal, "float", None::<bool>, None::<i32>, Some(f)),
            };
            insert.write(&row).await?;
        }
        
        insert.end().await?;
        Ok(())
    }
    
    async fn sync_from_local(&self, local_path: &Path) -> Result<()> {
        // ClickHouse can directly read Parquet files!
        let query = format!(
            "INSERT INTO signals SELECT * FROM file('{}', 'Parquet')",
            local_path.display()
        );
        
        self.client.query(&query).execute().await?;
        
        debug!("Synced {} to ClickHouse", local_path.display());
        Ok(())
    }
    
    fn name(&self) -> &str {
        "ClickHouse"
    }
}

pub struct S3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

impl S3Storage {
    pub async fn new(config: super::S3Config) -> Result<Self> {
        let aws_config = aws_config::load_defaults().await;
        let client = aws_sdk_s3::Client::new(&aws_config);
        
        Ok(Self {
            client,
            bucket: config.bucket,
            prefix: config.prefix,
        })
    }
}

#[async_trait::async_trait]
impl RemoteStorage for S3Storage {
    async fn write_batch(&self, entries: Vec<(DateTime<Utc>, String, Value)>) -> Result<()> {
        // For S3, we sync entire Parquet files
        Ok(())
    }
    
    async fn sync_from_local(&self, local_path: &Path) -> Result<()> {
        let key = format!(
            "{}/{}",
            self.prefix,
            local_path.file_name().unwrap().to_str().unwrap()
        );
        
        let body = ByteStream::from_path(local_path).await?;
        
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .send()
            .await?;
        
        debug!("Uploaded {} to S3", key);
        Ok(())
    }
    
    fn name(&self) -> &str {
        "S3"
    }
}

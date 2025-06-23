use crate::{error::*, value::Value};
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::{info, debug, error};

pub struct LocalStorage {
    config: super::LocalStorageConfig,
    current_writer: Option<ParquetWriter>,
    schema: Arc<Schema>,
}

struct ParquetWriter {
    writer: ArrowWriter<File>,
    path: PathBuf,
    size: u64,
    row_count: usize,
}

impl LocalStorage {
    pub fn new(config: super::LocalStorageConfig) -> Result<Self> {
        // Create data directory
        fs::create_dir_all(&config.data_dir)?;
        
        // Define schema for time-series data
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp", DataType::Timestamp(TimeUnit::Nanosecond, None), false),
            Field::new("signal", DataType::Utf8, false),
            Field::new("value_type", DataType::Utf8, false),
            Field::new("value_bool", DataType::Boolean, true),
            Field::new("value_int", DataType::Int32, true),
            Field::new("value_float", DataType::Float64, true),
        ]));
        
        Ok(Self {
            config,
            current_writer: None,
            schema,
        })
    }
    
    pub async fn write_batch(&mut self, entries: Vec<(DateTime<Utc>, String, Value)>) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }
        
        // Prepare arrays
        let mut timestamp_builder = TimestampNanosecondBuilder::new();
        let mut signal_builder = StringBuilder::new();
        let mut value_type_builder = StringBuilder::new();
        let mut bool_builder = BooleanBuilder::new();
        let mut int_builder = Int32Builder::new();
        let mut float_builder = Float64Builder::new();
        
        for (ts, signal, value) in entries {
            timestamp_builder.append_value(ts.timestamp_nanos_opt().unwrap_or(0));
            signal_builder.append_value(&signal);
            
            match value {
                Value::Bool(b) => {
                    value_type_builder.append_value("bool");
                    bool_builder.append_value(b);
                    int_builder.append_null();
                    float_builder.append_null();
                }
                Value::Int(i) => {
                    value_type_builder.append_value("int");
                    bool_builder.append_null();
                    int_builder.append_value(i);
                    float_builder.append_null();
                }
                Value::Float(f) => {
                    value_type_builder.append_value("float");
                    bool_builder.append_null();
                    int_builder.append_null();
                    float_builder.append_value(f);
                }
            }
        }
        
        // Create record batch
        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(timestamp_builder.finish()),
                Arc::new(signal_builder.finish()),
                Arc::new(value_type_builder.finish()),
                Arc::new(bool_builder.finish()),
                Arc::new(int_builder.finish()),
                Arc::new(float_builder.finish()),
            ],
        ).map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create record batch: {}", e)
        )))?;
        
        // Get or create writer
        let writer = self.get_or_create_writer()?;
        
        // Write batch
        writer.writer.write(&batch)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write batch: {}", e)
            )))?;
        
        writer.row_count += batch.num_rows();
        writer.size = writer.writer.flushed_bytes() as u64;
        
        // Rotate if needed
        if writer.size > self.config.max_file_size_mb * 1024 * 1024 {
            self.rotate_file()?;
        }
        
        Ok(())
    }
    
    fn get_or_create_writer(&mut self) -> Result<&mut ParquetWriter> {
        if self.current_writer.is_none() {
            let filename = format!("petra_{}.parquet", Utc::now().timestamp_nanos_opt().unwrap_or(0));
            let path = self.config.data_dir.join(filename);
            
            let file = File::create(&path)?;
            
            let props = WriterProperties::builder()
                .build();
            
            let writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))
                .map_err(|e| PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create parquet writer: {}", e)
                )))?;
            
            self.current_writer = Some(ParquetWriter {
                writer,
                path: path.clone(),
                size: 0,
                row_count: 0,
            });
            
            info!("Created new parquet file: {:?}", path);
        }
        
        Ok(self.current_writer.as_mut().unwrap())
    }
    
    fn rotate_file(&mut self) -> Result<()> {
        if let Some(mut writer) = self.current_writer.take() {
            writer.writer.close()
                .map_err(|e| PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to close parquet file: {}", e)
                )))?;
            
            info!("Rotated parquet file: {:?} ({} rows, {} MB)", 
                writer.path, 
                writer.row_count,
                writer.size / 1024 / 1024
            );
        }
        
        Ok(())
    }
    
    pub async fn query_range(
        &self,
        signal: Option<&str>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<(DateTime<Utc>, String, Value)>> {
        // Use DataFusion for efficient querying
        use datafusion::prelude::*;
        
        let ctx = SessionContext::new();
        
        // Register all parquet files in range
        let files: Vec<_> = fs::read_dir(&self.config.data_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .map(|ext| ext == "parquet")
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect();
        
        for (i, file) in files.iter().enumerate() {
            ctx.register_parquet(
                &format!("t{}", i),
                file.to_str().unwrap(),
                ParquetReadOptions::default(),
            ).await.map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to register parquet file: {}", e)
            )))?;
        }
        
        // Build query
        let mut query = String::from("SELECT timestamp, signal, value_type, value_bool, value_int, value_float FROM (");
        for i in 0..files.len() {
            if i > 0 {
                query.push_str(" UNION ALL ");
            }
            query.push_str(&format!("SELECT * FROM t{}", i));
        }
        query.push_str(&format!(
            ") WHERE timestamp >= {} AND timestamp <= {}",
            start.timestamp_nanos_opt().unwrap_or(0),
            end.timestamp_nanos_opt().unwrap_or(0)
        ));
        
        if let Some(sig) = signal {
            query.push_str(&format!(" AND signal = '{}'", sig));
        }
        
        query.push_str(" ORDER BY timestamp");
        
        // Execute query
        let df = ctx.sql(&query).await.map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Query failed: {}", e)
        )))?;
        
        let results = df.collect().await.map_err(|e| PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to collect results: {}", e)
        )))?;
        
        // Convert results back to our format
        let mut output = Vec::new();
        for batch in results {
            let timestamp_array = batch.column(0)
                .as_any()
                .downcast_ref::<TimestampNanosecondArray>()
                .unwrap();
            let signal_array = batch.column(1)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let value_type_array = batch.column(2)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            let bool_array = batch.column(3)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap();
            let int_array = batch.column(4)
                .as_any()
                .downcast_ref::<Int32Array>()
                .unwrap();
            let float_array = batch.column(5)
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap();
            
            for i in 0..batch.num_rows() {
                let ts = DateTime::from_timestamp_nanos(timestamp_array.value(i));
                let signal = signal_array.value(i).to_string();
                let value_type = value_type_array.value(i);
                
                let value = match value_type {
                    "bool" => Value::Bool(bool_array.value(i)),
                    "int" => Value::Int(int_array.value(i)),
                    "float" => Value::Float(float_array.value(i)),
                    _ => continue,
                };
                
                output.push((ts, signal, value));
            }
        }
        
        Ok(output)
    }
    
    pub async fn compact_old_files(&self) -> Result<()> {
        // Implement file compaction logic
        // For now, just log that compaction was called
        info!("File compaction completed (placeholder implementation)");
        Ok(())
    }
}

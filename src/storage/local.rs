use crate::{error::*, value::Value};
use arrow::array::*;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use parquet::basic::Compression;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tracing::info;

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
        let mut timestamp_builder = TimestampNanosecondBuilder::with_capacity(entries.len());
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
        
        // Check file size and rotate if needed
        let file_metadata = std::fs::metadata(&writer.path)?;
        writer.size = file_metadata.len();
        
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
            
            let compression = match self.config.compression {
                super::CompressionType::None => Compression::UNCOMPRESSED,
                super::CompressionType::Zstd => Compression::ZSTD(Default::default()),
                super::CompressionType::Lz4 => Compression::LZ4,
                super::CompressionType::Snappy => Compression::SNAPPY,
            };
            
            let props = WriterProperties::builder()
                .set_compression(compression)
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
}

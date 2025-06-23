// src/bin/storage_test.rs
use petra::{Config, Engine, Value, Result};
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tokio::sync::mpsc;
use tracing::{info, error, debug};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("storage_test=info,petra=debug")
        .init();

    info!("Petra Storage Test v{}", petra::VERSION);

    // Create test data directory
    std::fs::create_dir_all("./data/storage_test")?;

    let config = Config::from_file("configs/storage-test.yaml")?;
    let mut engine = Engine::new(config.clone())?;
    
    // Set up our own simple storage for testing
    let (storage_tx, mut storage_rx) = mpsc::channel::<(String, Value)>(1000);
    engine.set_signal_change_channel(storage_tx);
    
    // Simple local storage writer
    let storage_handle = tokio::spawn(async move {
        let mut data_points = Vec::new();
        let mut flush_interval = tokio::time::interval(Duration::from_secs(2));
        let start_time = std::time::Instant::now();
        
        loop {
            tokio::select! {
                Some((name, value)) = storage_rx.recv() => {
                    let timestamp = start_time.elapsed().as_secs_f64();
                    data_points.push((timestamp, name, value));
                    debug!("Stored data point: {} points total", data_points.len());
                }
                _ = flush_interval.tick() => {
                    if !data_points.is_empty() {
                        if let Err(e) = write_to_parquet(&data_points).await {
                            error!("Failed to write parquet: {}", e);
                        } else {
                            info!("Wrote {} data points to parquet", data_points.len());
                            data_points.clear();
                        }
                    }
                }
            }
        }
    });

    // Run test for a limited time
    let test_duration = Duration::from_secs(30);
    info!("Running storage test for {:?}", test_duration);

    let engine_handle = tokio::spawn(async move {
        if let Err(e) = engine.run().await {
            error!("Engine error: {}", e);
        }
    });

    // Let it run for the test duration
    sleep(test_duration).await;
    
    // Stop everything
    engine_handle.abort();
    storage_handle.abort();
    
    info!("Test completed, checking results...");
    
    // Give a moment for final writes
    sleep(Duration::from_secs(2)).await;
    
    // Check if parquet files were created
    check_storage_results().await?;
    
    Ok(())
}

async fn write_to_parquet(data_points: &[(f64, String, Value)]) -> Result<()> {
    use arrow::array::*;
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;
    use parquet::file::properties::WriterProperties;
    use std::fs::File;
    use std::sync::Arc;
    
    if data_points.is_empty() {
        return Ok(());
    }
    
    // Create schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("timestamp", DataType::Float64, false),
        Field::new("signal", DataType::Utf8, false),
        Field::new("value_type", DataType::Utf8, false),
        Field::new("value_bool", DataType::Boolean, true),
        Field::new("value_int", DataType::Int32, true),
        Field::new("value_float", DataType::Float64, true),
    ]));
    
    // Build arrays
    let mut timestamp_builder = Float64Builder::new();
    let mut signal_builder = StringBuilder::new();
    let mut value_type_builder = StringBuilder::new();
    let mut bool_builder = BooleanBuilder::new();
    let mut int_builder = Int32Builder::new();
    let mut float_builder = Float64Builder::new();
    
    for (timestamp, signal, value) in data_points {
        timestamp_builder.append_value(*timestamp);
        signal_builder.append_value(signal);
        
        match value {
            Value::Bool(b) => {
                value_type_builder.append_value("bool");
                bool_builder.append_value(*b);
                int_builder.append_null();
                float_builder.append_null();
            }
            Value::Int(i) => {
                value_type_builder.append_value("int");
                bool_builder.append_null();
                int_builder.append_value(*i);
                float_builder.append_null();
            }
            Value::Float(f) => {
                value_type_builder.append_value("float");
                bool_builder.append_null();
                int_builder.append_null();
                float_builder.append_value(*f);
            }
        }
    }
    
    // Create record batch
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(timestamp_builder.finish()),
            Arc::new(signal_builder.finish()),
            Arc::new(value_type_builder.finish()),
            Arc::new(bool_builder.finish()),
            Arc::new(int_builder.finish()),
            Arc::new(float_builder.finish()),
        ],
    ).map_err(|e| petra::PlcError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("Failed to create record batch: {}", e)
    )))?;
    
    // Write to file
    let filename = format!("./data/storage_test/petra_test_{}.parquet", 
                          std::time::SystemTime::now()
                              .duration_since(std::time::UNIX_EPOCH)
                              .unwrap()
                              .as_secs());
    
    let file = File::create(&filename)
        .map_err(|e| petra::PlcError::Io(e))?;
    
    let props = WriterProperties::builder()
        .set_compression(parquet::basic::Compression::SNAPPY)
        .build();
    
    let mut writer = ArrowWriter::try_new(file, schema, Some(props))
        .map_err(|e| petra::PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create parquet writer: {}", e)
        )))?;
    
    writer.write(&batch)
        .map_err(|e| petra::PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to write batch: {}", e)
        )))?;
    
    writer.close()
        .map_err(|e| petra::PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to close parquet file: {}", e)
        )))?;
    
    info!("Successfully wrote parquet file: {}", filename);
    Ok(())
}

async fn check_storage_results() -> Result<()> {
    let data_dir = Path::new("./data/storage_test");
    
    if !data_dir.exists() {
        error!("Data directory does not exist!");
        return Ok(());
    }
    
    let mut parquet_files = 0;
    let mut total_size = 0u64;
    let mut total_rows = 0usize;
    
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "parquet" {
                    parquet_files += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                    
                    // Try to read and count rows
                    if let Ok(row_count) = count_parquet_rows(&entry.path()) {
                        total_rows += row_count;
                        info!("Found parquet file: {:?} ({} rows)", entry.path(), row_count);
                    } else {
                        info!("Found parquet file: {:?} (could not read)", entry.path());
                    }
                }
            }
        }
    }
    
    if parquet_files > 0 {
        info!("✅ Storage test PASSED:");
        info!("   - {} parquet files created", parquet_files);
        info!("   - Total rows: {}", total_rows);
        info!("   - Total size: {} bytes", total_size);
        info!("   - Average file size: {} bytes", total_size / parquet_files);
        
        if total_rows > 0 {
            info!("   - Data collection rate: {:.1} rows/second", total_rows as f64 / 30.0);
        }
    } else {
        error!("❌ Storage test FAILED: No parquet files found");
    }
    
    Ok(())
}

fn count_parquet_rows(path: &Path) -> std::result::Result<usize, Box<dyn std::error::Error>> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    use std::fs::File;
    
    let file = File::open(path)?;
    let reader = SerializedFileReader::new(file)?;
    let parquet_metadata = reader.metadata();
    
    let mut total_rows = 0;
    for row_group in parquet_metadata.row_groups() {
        total_rows += row_group.num_rows() as usize;
    }
    
    Ok(total_rows)
}

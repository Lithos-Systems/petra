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
    
    // Simple local storage writer
    let storage_handle = tokio::spawn(async move {
        let mut data_points = Vec::new();
        let mut flush_interval = tokio::time::interval(Duration::from_secs(5));
        let start_time = std::time::Instant::now();
        let mut file_counter = 0;
        
        loop {
            tokio::select! {
                Some((name, value)) = storage_rx.recv() => {
                    let timestamp = start_time.elapsed().as_secs_f64();
                    data_points.push((timestamp, name, value));
                    debug!("Stored data point: {} points total", data_points.len());
                }
                _ = flush_interval.tick() => {
                    if !data_points.is_empty() {
                        file_counter += 1;
                        if let Err(e) = write_to_parquet(&data_points, file_counter).await {
                            error!("Failed to write parquet: {}", e);
                        } else {
                            info!("Wrote {} data points to parquet file {}", data_points.len(), file_counter);
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
    
    // Give storage a moment to finish current write
    sleep(Duration::from_secs(2)).await;
    storage_handle.abort();
    
    info!("Test completed, checking results...");
    
    // Check if parquet files were created
    check_storage_results().await?;
    
    Ok(())
}

async fn write_to_parquet(data_points: &[(f64, String, Value)], file_number: i32) -> Result<()> {
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
    
    info!("Writing {} data points to parquet file", data_points.len());
    
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
            Value::Integer(i) => {
                value_type_builder.append_value("int");
                bool_builder.append_null();
                int_builder.append_value(*i as i32);
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
    let filename = format!("./data/storage_test/petra_test_{:03}.parquet", file_number);
    
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
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                parquet_files += 1;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
                
                // Try to read and count rows
                if let Ok(row_count) = count_parquet_rows(&path) {
                    total_rows += row_count;
                    info!("Found parquet file: {:?} ({} rows, {} bytes)", 
                         path, row_count, entry.metadata().unwrap().len());
                } else {
                    info!("Found parquet file: {:?} (could not read)", path);
                }
            }
        }
    }
    
    if parquet_files > 0 {
        info!("✅ Storage test PASSED:");
        info!("   📁 {} parquet files created", parquet_files);
        info!("   📊 {} total data rows", total_rows);
        info!("   💾 {} total bytes", total_size);
        info!("   📈 {:.1} rows/second average", total_rows as f64 / 30.0);
        info!("   📏 {} bytes/file average", total_size / parquet_files.max(1));
        
        // Sample some data
        if let Some(entry) = std::fs::read_dir(data_dir).ok()
            .and_then(|mut entries| entries.find(|e| {
                match e {
                    Ok(dir_entry) => {
                        dir_entry
                            .path()
                            .extension()
                            .map(|ext| ext == "parquet")
                            .unwrap_or(false)
                    }
                    Err(_) => false,
                }
            })) {
            if let Ok(entry) = entry {
                info!("📋 Sample data from {}:", entry.path().display());
                if let Err(e) = sample_parquet_data(&entry.path()) {
                    error!("   Could not read sample data: {}", e);
                }
            }
        }
    } else {
        error!("❌ Storage test FAILED: No parquet files found");
        error!("   Check that the DATA_GENERATOR block is creating signals");
        error!("   Make sure the engine is running and processing blocks");
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

fn sample_parquet_data(path: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::fs::File;
    // Import the array types from arrow
    use arrow::array::{Array, Float64Array, StringArray, BooleanArray, Int32Array};
    
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let mut reader = builder.build()?;
    
    if let Some(batch) = reader.next() {
        let batch = batch?;
        let num_rows = batch.num_rows().min(5); // Show first 5 rows
        
        for row in 0..num_rows {
            let timestamp = batch.column(0).as_any().downcast_ref::<Float64Array>()
                .map(|arr| arr.value(row)).unwrap_or(0.0);
            let signal = batch.column(1).as_any().downcast_ref::<StringArray>()
                .map(|arr| arr.value(row)).unwrap_or("unknown");
            let value_type = batch.column(2).as_any().downcast_ref::<StringArray>()
                .map(|arr| arr.value(row)).unwrap_or("unknown");
            
            let value_str = match value_type {
                "bool" => {
                    batch.column(3).as_any().downcast_ref::<BooleanArray>()
                        .and_then(|arr| if arr.is_null(row) { None } else { Some(arr.value(row).to_string()) })
                        .unwrap_or("null".to_string())
                }
                "int" => {
                    batch.column(4).as_any().downcast_ref::<Int32Array>()
                        .and_then(|arr| if arr.is_null(row) { None } else { Some(arr.value(row).to_string()) })
                        .unwrap_or("null".to_string())
                }
                "float" => {
                    batch.column(5).as_any().downcast_ref::<Float64Array>()
                        .and_then(|arr| if arr.is_null(row) { None } else { Some(format!("{:.3}", arr.value(row))) })
                        .unwrap_or("null".to_string())
                }
                _ => "unknown".to_string()
            };
            
            info!("   [{:.1}s] {} = {}", timestamp, signal, value_str);
        }
    }
    
    Ok(())
}

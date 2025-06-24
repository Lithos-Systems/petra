// src/bin/parquet_viewer.rs
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::file::reader::{FileReader, SerializedFileReader};
use arrow::array::{Array, BooleanArray, Float64Array, Int32Array, StringArray};
use std::fs::File;
use std::path::Path;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "parquet_viewer")]
#[command(about = "View Petra parquet files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all parquet files in directory
    List {
        /// Directory to search
        #[arg(short, long, default_value = "./data/storage_test")]
        dir: String,
    },
    /// Show file info and schema
    Info {
        /// Parquet file path
        file: String,
    },
    /// Show data from file
    Show {
        /// Parquet file path
        file: String,
        /// Number of rows to show
        #[arg(short, long, default_value = "10")]
        rows: usize,
        /// Filter by signal name
        #[arg(short, long)]
        signal: Option<String>,
    },
    /// Export to CSV
    Export {
        /// Parquet file path
        file: String,
        /// Output CSV file
        #[arg(short, long)]
        output: String,
    },
    /// Show statistics
    Stats {
        /// Parquet file or directory
        path: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::List { dir } => list_files(dir)?,
        Commands::Info { file } => show_info(file)?,
        Commands::Show { file, rows, signal } => show_data(file, *rows, signal.as_deref())?,
        Commands::Export { file, output } => export_csv(file, output)?,
        Commands::Stats { path } => show_stats(path)?,
    }

    Ok(())
}

fn list_files(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Parquet files in {}:", dir);
    
    let entries = std::fs::read_dir(dir)?;
    let mut files = Vec::new();
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
            let metadata = entry.metadata()?;
            let size = metadata.len();
            
            // Get row count
            let row_count = count_rows(&path).unwrap_or(0);
            
            files.push((path, size, row_count));
        }
    }
    
    files.sort_by(|a, b| a.0.cmp(&b.0));
    
    println!("{:<30} {:>10} {:>10}", "File", "Size", "Rows");
    println!("{:-<50}", "");
    
    for (path, size, rows) in files {
        println!("{:<30} {:>7} KB {:>10}", 
                path.file_name().unwrap().to_str().unwrap(),
                size / 1024,
                rows);
    }
    
    Ok(())
}

fn show_info(file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 File Info: {}", file);
    
    let file_handle = File::open(file)?;
    let reader = SerializedFileReader::new(file_handle)?;
    let metadata = reader.metadata();
    
    println!("Schema:");
    for field in metadata.file_metadata().schema().get_fields() {
        let type_info = if let Some(logical_type) = field.get_basic_info().logical_type() {
            format!("{:?}", logical_type)
        } else {
            format!("{:?}", field.get_basic_info().type_())
        };
        println!("  {} ({})", field.name(), type_info);
    }
    
    println!("\nFile Stats:");
    println!("  Row Groups: {}", metadata.num_row_groups());
    println!("  Total Rows: {}", metadata.file_metadata().num_rows());
    
    for (i, rg) in metadata.row_groups().iter().enumerate() {
        println!("  Row Group {}: {} rows, {} bytes", 
                i, rg.num_rows(), rg.total_byte_size());
    }
    
    Ok(())
}

fn show_data(file: &str, max_rows: usize, signal_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Data from: {}", file);
    if let Some(signal) = signal_filter {
        println!("🔍 Filtered by signal: {}", signal);
    }
    
    let file_handle = File::open(file)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file_handle)?;
    let mut reader = builder.build()?;
    
    let mut rows_shown = 0;
    
    while let Some(batch) = reader.next() {
        let batch = batch?;
        let num_rows = batch.num_rows();
        
        for row in 0..num_rows {
            if rows_shown >= max_rows {
                println!("... (showing first {} rows)", max_rows);
                return Ok(());
            }
            
            let timestamp = batch.column(0).as_any()
                .downcast_ref::<Float64Array>()
                .map(|arr| arr.value(row)).unwrap_or(0.0);
            
            let signal = batch.column(1).as_any()
                .downcast_ref::<StringArray>()
                .map(|arr| arr.value(row)).unwrap_or("unknown");
            
            // Apply signal filter
            if let Some(filter) = signal_filter {
                if signal != filter {
                    continue;
                }
            }
            
            let value_type = batch.column(2).as_any()
                .downcast_ref::<StringArray>()
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
            
            println!("[{:8.1}s] {:>15} = {:<10} ({})", timestamp, signal, value_str, value_type);
            rows_shown += 1;
        }
    }
    
    Ok(())
}

fn export_csv(file: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Exporting {} to {}", file, output);
    
    let file_handle = File::open(file)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file_handle)?;
    let mut reader = builder.build()?;
    
    let mut csv_writer = csv::Writer::from_path(output)?;
    
    // Write header
    csv_writer.write_record(&["timestamp", "signal", "value_type", "value"])?;
    
    while let Some(batch) = reader.next() {
        let batch = batch?;
        let num_rows = batch.num_rows();
        
        for row in 0..num_rows {
            let timestamp = batch.column(0).as_any()
                .downcast_ref::<Float64Array>()
                .map(|arr| arr.value(row)).unwrap_or(0.0);
            
            let signal = batch.column(1).as_any()
                .downcast_ref::<StringArray>()
                .map(|arr| arr.value(row)).unwrap_or("unknown");
            
            let value_type = batch.column(2).as_any()
                .downcast_ref::<StringArray>()
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
            
            csv_writer.write_record(&[
                timestamp.to_string(),
                signal.to_string(),
                value_type.to_string(),
                value_str,
            ])?;
        }
    }
    
    csv_writer.flush()?;
    println!("✅ Export complete!");
    
    Ok(())
}

fn show_stats(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📈 Statistics for: {}", path);
    
    let path = Path::new(path);
    let files = if path.is_dir() {
        std::fs::read_dir(path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "parquet" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    } else {
        vec![path.to_path_buf()]
    };
    
    let mut total_rows = 0;
    let mut total_size = 0;
    let mut signal_counts = std::collections::HashMap::new();
    
    for file_path in files {
        let metadata = std::fs::metadata(&file_path)?;
        total_size += metadata.len();
        
        let rows = count_rows(&file_path).unwrap_or(0);
        total_rows += rows;
        
        // Count signals
        if let Ok(counts) = count_signals(&file_path) {
            for (signal, count) in counts {
                *signal_counts.entry(signal).or_insert(0) += count;
            }
        }
    }
    
    println!("Total Rows: {}", total_rows);
    println!("Total Size: {} KB", total_size / 1024);
    println!("\nSignal Counts:");
    
    let mut sorted_signals: Vec<_> = signal_counts.iter().collect();
    sorted_signals.sort_by(|a, b| b.1.cmp(a.1));
    
    for (signal, count) in sorted_signals {
        println!("  {:20} {:>8} samples", signal, count);
    }
    
    Ok(())
}

fn count_rows(path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = SerializedFileReader::new(file)?;
    let metadata = reader.metadata();
    
    let mut total_rows = 0;
    for row_group in metadata.row_groups() {
        total_rows += row_group.num_rows() as usize;
    }
    
    Ok(total_rows)
}

fn count_signals(path: &Path) -> Result<std::collections::HashMap<String, usize>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let mut reader = builder.build()?;
    
    let mut signal_counts = std::collections::HashMap::new();
    
    while let Some(batch) = reader.next() {
        let batch = batch?;
        let num_rows = batch.num_rows();
        
        for row in 0..num_rows {
            if let Some(signal_array) = batch.column(1).as_any().downcast_ref::<StringArray>() {
                let signal = signal_array.value(row);
                *signal_counts.entry(signal.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    Ok(signal_counts)
}

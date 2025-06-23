// src/bin/storage_test.rs
use petra::{Config, Engine, HistoryManager, Result};
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tokio::task::LocalSet;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("storage_test=info,petra=debug")
        .init();

    info!("Petra Storage Test v{}", petra::VERSION);

    // Create test data directory
    std::fs::create_dir_all("./data/storage_test")?;
    std::fs::create_dir_all("./data/wal")?;

    let config = Config::from_file("configs/storage-test.yaml")?;
    let mut engine = Engine::new(config.clone())?;

    let local = LocalSet::new();

    local.run_until(async move {
    
    // Initialize history manager if history is configured
    let _history_handle = if let Some(history_config) = config.history {
        info!("Starting storage manager for testing");
        let mut history_manager = HistoryManager::new(history_config, engine.bus().clone())?;
        
        // Set up signal change tracking
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        engine.set_signal_change_channel(tx);
        history_manager.set_signal_change_channel(rx);
        
        Some(tokio::task::spawn_local(async move {
            if let Err(e) = history_manager.run().await {
                error!("History manager error: {}", e);
            }
        }))
    } else {
        None
    };

    // Run test for a limited time
    let test_duration = Duration::from_secs(30);
    info!("Running storage test for {:?}", test_duration);

    let _engine_handle = tokio::task::spawn_local(async move {
        if let Err(e) = engine.run().await {
            error!("Engine error: {}", e);
        }
    });

    // Let it run for the test duration
    sleep(test_duration).await;
    
    info!("Test completed, checking results...");
    
    // Check if parquet files were created
    check_storage_results().await?;
    
        Ok::<(), petra::PlcError>(())
    }).await?;

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
    
    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "parquet" {
                    parquet_files += 1;
                    if let Ok(metadata) = entry.metadata() {
                        total_size += metadata.len();
                    }
                    info!("Found parquet file: {:?}", entry.path());
                }
            }
        }
    }
    
    if parquet_files > 0 {
        info!("✅ Storage test PASSED:");
        info!("   - {} parquet files created", parquet_files);
        info!("   - Total size: {} bytes", total_size);
        info!("   - Average file size: {} bytes", total_size / parquet_files);
    } else {
        error!("❌ Storage test FAILED: No parquet files found");
    }
    
    Ok(())
}

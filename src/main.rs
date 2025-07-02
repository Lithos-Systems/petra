// Updates to src/main.rs

use clap::Parser;
use std::sync::{Arc, RwLock};
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "petra")]
#[command(about = "High-performance automation engine", long_about = None)]
struct Args {
    /// Configuration file path
    config: PathBuf,
    
    /// Override scan time in milliseconds
    #[arg(long)]
    scan_time: Option<u64>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Enable hot configuration reloading
    #[arg(long)]
    hot_reload: bool,
    
    /// Configuration reload validation mode
    #[arg(long, value_enum, default_value = "permissive")]
    reload_mode: ValidationMode,
    
    /// Enable status API server
    #[arg(long)]
    status_api: bool,
    
    /// Status API port
    #[arg(long, default_value = "8080")]
    status_port: u16,
    
    /// Enable real-time priority (requires root)
    #[arg(long)]
    realtime: bool,
    
    /// Real-time priority level (1-99)
    #[arg(long, default_value = "50")]
    rt_priority: u8,
    
    /// CPU affinity (comma-separated list of CPU cores)
    #[arg(long)]
    cpu_affinity: Option<String>,
    
    /// Lock memory to prevent paging (requires root)
    #[arg(long)]
    lock_memory: bool,
    
    /// Burn in configuration for maximum performance
    #[arg(long)]
    burn_in: bool,
    
    /// Delay before burning in configuration (seconds)
    #[arg(long, default_value = "30")]
    burn_in_delay: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("petra={},tower_http=info", log_level))
        .init();
    
    info!("PETRA v{} starting", env!("CARGO_PKG_VERSION"));
    
    // Set real-time priority if requested
    if args.realtime {
        set_realtime_priority(args.rt_priority)?;
    }
    
    // Set CPU affinity if specified
    if let Some(affinity) = args.cpu_affinity {
        set_cpu_affinity(&affinity)?;
    }
    
    // Lock memory if requested
    if args.lock_memory {
        lock_process_memory()?;
    }
    
    // Load initial configuration
    let mut config = Config::from_file(&args.config)?;
    
    // Override scan time if specified
    if let Some(scan_time) = args.scan_time {
        config.scan_time_ms = scan_time;
        info!("Overriding scan time to {}ms", scan_time);
    }
    
    // Create engine with Arc<RwLock> for sharing
    let engine = Arc::new(RwLock::new(Engine::new(config)?));
    
    // Initialize metrics if enabled
    #[cfg(feature = "metrics")]
    {
        let metrics_engine = engine.clone();
        tokio::spawn(async move {
            if let Err(e) = start_metrics_server(metrics_engine).await {
                error!("Failed to start metrics server: {}", e);
            }
        });
    }
    
    // Set up configuration hot reloading if enabled
    let config_manager = if args.hot_reload {
        info!("Hot reload enabled with {:?} validation", args.reload_mode);
        
        let mut manager = Arc::new(ConfigManager::new(
            args.config.clone(),
            args.reload_mode,
        )?);
        
        // Enable file watching
        Arc::get_mut(&mut manager)
            .unwrap()
            .enable_file_watching()?;
        
        // Start reload handler
        let handler_manager = manager.clone();
        let handler_engine = engine.clone();
        std::thread::spawn(move || {
            handler_manager.start_handler(handler_engine);
        });
        
        Some(manager)
    } else {
        None
    };
    
    // Start status API if enabled
    if args.status_api {
        let status_engine = engine.clone();
        let status_port = args.status_port;
        
        tokio::spawn(async move {
            let server = StatusServer::new(status_engine, status_port);
            if let Err(e) = server.start().await {
                error!("Failed to start status API: {}", e);
            }
        });
        
        info!("Status API available at http://0.0.0.0:{}", args.status_port);
        info!("  REST API: http://0.0.0.0:{}/api/status", args.status_port);
        info!("  WebSocket: ws://0.0.0.0:{}/ws/realtime", args.status_port);
        info!("  SSE: http://0.0.0.0:{}/api/events", args.status_port);
    }
    
    // Set up burn-in if requested
    if args.burn_in {
        #[cfg(feature = "burn-in")]
        {
            let burn_engine = engine.clone();
            let burn_manager = config_manager.clone();
            let burn_delay = args.burn_in_delay;
            
            tokio::spawn(async move {
                info!("Configuration burn-in scheduled in {} seconds", burn_delay);
                tokio::time::sleep(tokio::time::Duration::from_secs(burn_delay)).await;
                
                if let Some(manager) = burn_manager {
                    if let Err(e) = manager.burn_in() {
                        error!("Failed to trigger burn-in: {}", e);
                    }
                } else {
                    // Direct burn-in without config manager
                    if let Ok(mut eng) = burn_engine.write() {
                        eng.burn_in_configuration();
                    }
                }
            });
        }
        
        #[cfg(not(feature = "burn-in"))]
        {
            warn!("Burn-in requested but feature not enabled. Rebuild with --features burn-in");
        }
    }
    
    // Set up signal handlers
    let shutdown_engine = engine.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Shutdown signal received");
                if let Ok(mut eng) = shutdown_engine.write() {
                    eng.stop();
                }
            }
            Err(e) => error!("Failed to listen for shutdown signal: {}", e),
        }
    });
    
    // Start the engine
    {
        let mut eng = engine.write().unwrap();
        eng.start()?;
    }
    
    // Print startup summary
    print_startup_summary(&args, &engine);
    
    // Run the engine (this blocks)
    loop {
        {
            let eng = engine.read().unwrap();
            if !eng.is_running() {
                break;
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    info!("Engine stopped");
    Ok(())
}

fn print_startup_summary(args: &Args, engine: &Arc<RwLock<Engine>>) {
    let eng = engine.read().unwrap();
    let stats = eng.get_stats();
    
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║              PETRA ENGINE STARTED                      ║");
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║ Configuration:                                         ║");
    println!("║   File: {:45} ║", args.config.display().to_string());
    println!("║   Signals: {:>5} | Blocks: {:>5}                     ║", 
        stats.signal_count, stats.block_count);
    println!("║   Scan Time: {:>4} ms                                  ║", 
        eng.config.scan_time_ms);
    
    if args.hot_reload {
        println!("║   Hot Reload: ENABLED ({:?} mode)              ║", args.reload_mode);
    } else {
        println!("║   Hot Reload: DISABLED                                ║");
    }
    
    if args.status_api {
        println!("║   Status API: http://0.0.0.0:{:>5}                   ║", args.status_port);
    }
    
    #[cfg(feature = "metrics")]
    println!("║   Metrics: http://0.0.0.0:9090/metrics               ║");
    
    if args.realtime {
        println!("║   Real-time: ENABLED (priority {})                    ║", args.rt_priority);
    }
    
    if args.burn_in {
        println!("║   Burn-in: SCHEDULED ({}s delay)                     ║", args.burn_in_delay);
    }
    
    println!("╚════════════════════════════════════════════════════════╝\n");
}

// Platform-specific implementations
#[cfg(target_os = "linux")]
fn set_realtime_priority(priority: u8) -> Result<()> {
    use libc::{sched_param, sched_setscheduler, SCHED_FIFO};
    use std::os::raw::c_int;
    
    let param = sched_param {
        sched_priority: priority as c_int,
    };
    
    unsafe {
        if sched_setscheduler(0, SCHED_FIFO, &param) == -1 {
            return Err(PlcError::Runtime(
                "Failed to set real-time priority (are you root?)".into()
            ));
        }
    }
    
    info!("Set real-time priority to {}", priority);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn set_realtime_priority(_priority: u8) -> Result<()> {
    warn!("Real-time priority not supported on this platform");
    Ok(())
}

#[cfg(target_os = "linux")]
fn set_cpu_affinity(affinity_str: &str) -> Result<()> {
    use libc::{cpu_set_t, CPU_SET, CPU_ZERO, sched_setaffinity};
    use std::mem;
    
    let mut cpu_set: cpu_set_t = unsafe { mem::zeroed() };
    unsafe { CPU_ZERO(&mut cpu_set) };
    
    for cpu_str in affinity_str.split(',') {
        let cpu: usize = cpu_str.trim().parse()
            .map_err(|_| PlcError::Config(format!("Invalid CPU number: {}", cpu_str)))?;
        unsafe { CPU_SET(cpu, &mut cpu_set) };
    }
    
    unsafe {
        if sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set) == -1 {
            return Err(PlcError::Runtime("Failed to set CPU affinity".into()));
        }
    }
    
    info!("Set CPU affinity to: {}", affinity_str);
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn set_cpu_affinity(_affinity_str: &str) -> Result<()> {
    warn!("CPU affinity not supported on this platform");
    Ok(())
}

#[cfg(target_os = "linux")]
fn lock_process_memory() -> Result<()> {
    use libc::{mlockall, MCL_CURRENT, MCL_FUTURE};
    
    unsafe {
        if mlockall(MCL_CURRENT | MCL_FUTURE) == -1 {
            return Err(PlcError::Runtime(
                "Failed to lock memory (are you root?)".into()
            ));
        }
    }
    
    info!("Process memory locked");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn lock_process_memory() -> Result<()> {
    warn!("Memory locking not supported on this platform");
    Ok(())
}

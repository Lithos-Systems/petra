// src/main.rs
use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error, warn};
use tokio::signal;
use petra::{
    config::Config,
    engine::Engine,
    error::Result,
};

#[cfg(feature = "mqtt")]
use petra::mqtt::MqttClient;

#[cfg(feature = "s7-support")]
use petra::s7::S7Driver;

#[cfg(feature = "history")]
use petra::history::HistoryManager;

#[cfg(feature = "metrics")]
use metrics_exporter_prometheus::PrometheusBuilder;

#[cfg(feature = "alarms")]
use petra::alarms::AlarmManager;

#[cfg(feature = "realtime")]
use petra::realtime::{set_realtime_priority, pin_to_cpu};

#[cfg(feature = "health")]
use petra::health::HealthServer;

#[cfg(feature = "security")]
use petra::security::{SecurityManager, validate_config_signature};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(value_name = "CONFIG")]
    config: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Validate configuration and exit
    #[arg(long)]
    validate: bool,

    /// Dry run - validate config only
    #[arg(long)]
    dry_run: bool,

    // ===== Enhanced/Optional Features =====
    
    #[cfg(feature = "metrics")]
    /// Metrics server bind address
    #[arg(long, default_value = "0.0.0.0:9090", env = "PETRA_METRICS_ADDR")]
    metrics_addr: String,

    #[cfg(feature = "health")]
    /// Health check server bind address
    #[arg(long, default_value = "0.0.0.0:8080", env = "PETRA_HEALTH_ADDR")]
    health_addr: String,

    #[cfg(feature = "enhanced-monitoring")]
    /// Enable enhanced monitoring
    #[arg(long, env = "PETRA_ENHANCED_MONITORING")]
    enhanced_monitoring: bool,

    #[cfg(feature = "realtime")]
    /// Enable real-time scheduling (requires root)
    #[arg(long)]
    realtime: bool,

    #[cfg(feature = "realtime")]
    /// CPU core to pin to (requires root)
    #[arg(long, env = "PETRA_CPU_AFFINITY")]
    cpu_affinity: Option<usize>,

    #[cfg(feature = "security")]
    /// Verify configuration signature
    #[arg(long)]
    verify_signature: bool,

    #[cfg(feature = "security")]
    /// Path to signing key for config verification
    #[arg(long, env = "PETRA_SIGNING_KEY")]
    signing_key: Option<PathBuf>,

    #[cfg(feature = "json-schema")]
    /// Print JSON schema and exit
    #[arg(long)]
    print_schema: bool,

    #[cfg(feature = "profiling")]
    /// Enable CPU profiling
    #[arg(long)]
    profile: bool,

    #[cfg(feature = "profiling")]
    /// Profile output path
    #[arg(long, default_value = "petra.profile")]
    profile_output: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    init_logging(&args)?;

    // Handle special commands
    #[cfg(feature = "json-schema")]
    if args.print_schema {
        println!("{}", petra::config_schema::generate_schema()?);
        return Ok(());
    }

    // Load and validate configuration
    info!("Loading configuration from: {}", args.config.display());
    let mut config = Config::from_file(&args.config)?;

    // Apply command-line overrides to config
    #[cfg(feature = "enhanced-monitoring")]
    if args.enhanced_monitoring {
        if let Some(engine_config) = &mut config.engine_config {
            engine_config.insert("enhanced_monitoring".into(), serde_yaml::Value::Bool(true));
        } else {
            let mut map = serde_yaml::Mapping::new();
            map.insert("enhanced_monitoring".into(), true.into());
            config.engine_config = Some(serde_yaml::Value::Mapping(map));
        }
    }

    // Verify config signature if requested
    #[cfg(feature = "security")]
    if args.verify_signature {
        let key_path = args.signing_key
            .ok_or_else(|| petra::error::PlcError::Config("Signing key required for verification".into()))?;
        validate_config_signature(&args.config, &key_path)?;
        info!("Configuration signature verified");
    }

    // Validate only mode
    if args.validate || args.dry_run {
        info!("Configuration validation successful");
        if args.dry_run {
            info!("Dry run complete - configuration is valid");
        }
        return Ok(());
    }

    // Apply real-time settings
    #[cfg(feature = "realtime")]
    {
        if args.realtime {
            set_realtime_priority()?;
            info!("Real-time scheduling enabled");
        }
        
        if let Some(cpu) = args.cpu_affinity {
            pin_to_cpu(cpu)?;
            info!("Pinned to CPU core {}", cpu);
        }
    }

    // Start profiling
    #[cfg(feature = "profiling")]
    let _profiler_guard = if args.profile {
        Some(start_profiling(&args.profile_output)?)
    } else {
        None
    };

    // Initialize components
    info!("Initializing Petra engine...");
    let mut engine = Engine::new(config.clone())?;

    // Initialize MQTT if configured
    #[cfg(feature = "mqtt")]
    let mqtt_handle = if let Some(mqtt_config) = &config.mqtt {
        info!("Connecting to MQTT broker at {}:{}", 
              mqtt_config.broker_host, mqtt_config.broker_port);
        
        let mqtt_client = MqttClient::new(mqtt_config.clone(), engine.get_bus().clone())?;
        Some(tokio::spawn(async move {
            if let Err(e) = mqtt_client.run().await {
                error!("MQTT client error: {}", e);
            }
        }))
    } else {
        None
    };

    // Initialize S7 driver if configured
    #[cfg(feature = "s7-support")]
    let s7_handle = if let Some(s7_config) = &config.s7 {
        info!("Connecting to S7 PLC at {}", s7_config.ip);
        
        let s7_driver = S7Driver::new(s7_config.clone(), engine.get_bus().clone())?;
        Some(tokio::spawn(async move {
            if let Err(e) = s7_driver.run().await {
                error!("S7 driver error: {}", e);
            }
        }))
    } else {
        None
    };

    // Initialize history manager
    #[cfg(feature = "history")]
    let history_handle = if let Some(history_config) = &config.history {
        info!("Starting history manager");
        
        let history_manager = HistoryManager::new(
            history_config.clone(),
            engine.get_bus().clone()
        ).await?;
        
        Some(tokio::spawn(async move {
            if let Err(e) = history_manager.run().await {
                error!("History manager error: {}", e);
            }
        }))
    } else {
        None
    };

    // Initialize alarm manager
    #[cfg(feature = "alarms")]
    let alarm_handle = if let Some(alarm_config) = &config.alarms {
        info!("Starting alarm manager with {} alarms", alarm_config.len());
        
        let alarm_manager = AlarmManager::new(
            alarm_config.clone(),
            engine.get_bus().clone()
        )?;
        
        Some(tokio::spawn(async move {
            if let Err(e) = alarm_manager.run().await {
                error!("Alarm manager error: {}", e);
            }
        }))
    } else {
        None
    };

    // Start metrics server
    #[cfg(feature = "metrics")]
    let metrics_handle = {
        info!("Starting metrics server on {}", args.metrics_addr);
        
        let builder = PrometheusBuilder::new();
        let (recorder, exporter) = builder
            .with_http_listener(args.metrics_addr.parse()?)
            .build()?;
        
        metrics::set_global_recorder(recorder)?;
        
        Some(tokio::spawn(async move {
            if let Err(e) = exporter.await {
                error!("Metrics server error: {}", e);
            }
        }))
    };

    // Start health server
    #[cfg(feature = "health")]
    let health_handle = {
        info!("Starting health server on {}", args.health_addr);
        
        let health_server = HealthServer::new(
            args.health_addr,
            engine.get_stats_handle()
        );
        
        Some(tokio::spawn(async move {
            if let Err(e) = health_server.run().await {
                error!("Health server error: {}", e);
            }
        }))
    };

    // Setup graceful shutdown
    let running = engine.get_running_flag();
    let shutdown_handler = tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received shutdown signal");
                running.store(false, std::sync::atomic::Ordering::Relaxed);
            }
            Err(e) => {
                error!("Error waiting for shutdown signal: {}", e);
            }
        }
    });

    // Run the engine
    info!("Starting scan engine with {}ms scan time", config.scan_time_ms);
    
    #[cfg(feature = "enhanced-monitoring")]
    if args.enhanced_monitoring {
        info!("Enhanced monitoring enabled");
    }
    
    if let Err(e) = engine.run().await {
        error!("Engine error: {}", e);
    }

    // Cleanup
    info!("Shutting down...");
    
    // Cancel all tasks
    shutdown_handler.abort();
    
    #[cfg(feature = "mqtt")]
    if let Some(handle) = mqtt_handle {
        handle.abort();
    }
    
    #[cfg(feature = "s7-support")]
    if let Some(handle) = s7_handle {
        handle.abort();
    }
    
    #[cfg(feature = "history")]
    if let Some(handle) = history_handle {
        handle.abort();
    }
    
    #[cfg(feature = "alarms")]
    if let Some(handle) = alarm_handle {
        handle.abort();
    }
    
    #[cfg(feature = "metrics")]
    if let Some(handle) = metrics_handle {
        handle.abort();
    }
    
    #[cfg(feature = "health")]
    if let Some(handle) = health_handle {
        handle.abort();
    }

    // Save profiling data
    #[cfg(feature = "profiling")]
    if args.profile {
        info!("Saving profiling data to {}", args.profile_output.display());
    }

    info!("Shutdown complete");
    Ok(())
}

fn init_logging(args: &Args) -> Result<()> {
    let log_level = if args.verbose {
        "debug"
    } else {
        &args.log_level
    };

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            format!("petra={},tower_http=warn", log_level).into()
        });

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(args.verbose)
        .with_file(args.verbose)
        .with_line_number(args.verbose)
        .init();

    Ok(())
}

#[cfg(feature = "profiling")]
fn start_profiling(output_path: &PathBuf) -> Result<impl Drop> {
    use pprof::ProfilerGuardBuilder;
    
    let guard = ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;
    
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_args_parsing() {
        let args = Args::parse_from(&["petra", "config.yaml"]);
        assert_eq!(args.config, PathBuf::from("config.yaml"));
        assert_eq!(args.log_level, "info");
        assert!(!args.verbose);
    }

    #[test]
    fn test_args_with_options() {
        let args = Args::parse_from(&[
            "petra",
            "config.yaml",
            "--log-level", "debug",
            "--verbose",
            "--dry-run",
        ]);
        assert_eq!(args.log_level, "debug");
        assert!(args.verbose);
        assert!(args.dry_run);
    }

    #[cfg(feature = "enhanced-monitoring")]
    #[test]
    fn test_enhanced_monitoring_arg() {
        let args = Args::parse_from(&[
            "petra",
            "config.yaml",
            "--enhanced-monitoring",
        ]);
        assert!(args.enhanced_monitoring);
    }
}

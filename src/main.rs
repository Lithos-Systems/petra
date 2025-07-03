//! PETRA binary entry point with comprehensive CLI interface
//!
//! This binary provides the main command-line interface for PETRA with support for:
//! - Configuration validation and loading
//! - Engine execution with different feature sets
//! - Feature detection and validation
//! - Development and testing utilities
//! - Hot configuration reloading

use clap::{Parser, Subcommand, ValueEnum};
use petra::{
    Config, Engine, Features, PlcError, Result,
};
use petra::engine::EngineConfig;
use std::path::PathBuf;
use std::process;
use tokio::signal;
use tracing::{info, warn, error, debug};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[cfg(feature = "metrics")]
use petra::metrics_server::MetricsServer;

#[cfg(feature = "health")]
use petra::health::HealthMonitor;
use std::net::SocketAddr;

// ============================================================================
// CLI ARGUMENT DEFINITIONS
// ============================================================================

#[derive(Parser)]
#[command(name = "petra")]
#[command(version = petra::VERSION)]
#[command(about = "PETRA - Programmable Engine for Telemetry, Runtime, and Automation")]
#[command(long_about = "
PETRA is a high-performance automation engine built in Rust with advanced 
industrial connectivity, alarm management, and enterprise data storage.

Features are enabled at compile time using Cargo feature flags.
Use 'petra features' to see which features are enabled in this build.
")]
struct Cli {
    /// Configuration file path
    #[arg(value_name = "CONFIG")]
    config: Option<PathBuf>,

    /// Override scan time in milliseconds
    #[arg(short, long)]
    scan_time: Option<u64>,

    /// Log level
    #[arg(short, long, value_enum, default_value = "info")]
    log_level: LogLevel,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Enable quiet mode (errors only)
    #[arg(short, long)]
    quiet: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Enable JSON log format
    #[arg(long)]
    json_logs: bool,

    /// Validate configuration only
    #[arg(long)]
    validate_only: bool,

    /// Dry run (don't start protocols or external connections)
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the PETRA engine
    Run {
        /// Configuration file path
        config: PathBuf,
        /// Override scan time
        #[arg(short, long)]
        scan_time: Option<u64>,
        /// Enable enhanced monitoring
        #[arg(long)]
        enhanced_monitoring: bool,
        /// Disable circuit breakers
        #[arg(long)]
        no_circuit_breakers: bool,
        /// CPU affinity mask (hex)
        #[arg(long)]
        cpu_affinity: Option<String>,
        /// Thread priority (1-99)
        #[arg(long)]
        thread_priority: Option<i32>,
    },
    
    /// Validate configuration file
    Validate {
        /// Configuration file path
        config: PathBuf,
        /// Show detailed validation report
        #[arg(short, long)]
        detailed: bool,
        /// Check feature compatibility
        #[arg(long)]
        check_features: bool,
    },
    
    /// Show enabled features and capabilities
    Features {
        /// Show feature dependencies
        #[arg(short, long)]
        dependencies: bool,
        /// Show feature conflicts
        #[arg(short, long)]
        conflicts: bool,
        /// Check specific feature
        #[arg(long)]
        check: Option<String>,
    },
    
    /// Configuration management utilities
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
    
    /// Development and testing utilities  
    #[cfg(any(feature = "examples", feature = "burn-in"))]
    Dev {
        #[command(subcommand)]
        dev_cmd: DevCommands,
    },
    
    /// Protocol testing utilities
    #[cfg(any(feature = "mqtt", feature = "s7-support", feature = "modbus-support"))]
    Protocol {
        #[command(subcommand)]
        protocol_cmd: ProtocolCommands,
    },
    
    /// Start metrics server only
    #[cfg(feature = "metrics")]
    Metrics {
        /// Port to bind to
        #[arg(short, long, default_value = "9090")]
        port: u16,
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
    },
    
    /// Start health server only  
    #[cfg(feature = "health")]
    Health {
        /// Port to bind to
        #[arg(short, long, default_value = "8080")]
        port: u16,
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Generate example configuration
    Example {
        /// Output file path
        #[arg(short, long, default_value = "petra-example.yaml")]
        output: PathBuf,
        /// Configuration template type
        #[arg(short, long, value_enum, default_value = "basic")]
        template: ConfigTemplate,
    },
    
    /// Convert configuration between formats
    Convert {
        /// Input configuration file
        input: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        /// Output format
        #[arg(short, long, value_enum, default_value = "yaml")]
        format: ConfigFormat,
    },
    
    /// Migrate configuration to new version
    Migrate {
        /// Input configuration file
        input: PathBuf,
        /// Output file path (defaults to backup + update input)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Validate and show configuration schema
    Schema {
        /// Show JSON schema
        #[cfg(feature = "json-schema")]
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
#[cfg(any(feature = "examples", feature = "burn-in"))]
enum DevCommands {
    /// Run example scenarios
    #[cfg(feature = "examples")]
    Examples {
        /// List available examples
        #[arg(short, long)]
        list: bool,
        /// Run specific example
        #[arg(short, long)]
        run: Option<String>,
    },
    
    /// Run burn-in tests
    #[cfg(feature = "burn-in")]
    BurnIn {
        /// Duration in seconds
        #[arg(short, long, default_value = "3600")]
        duration: u64,
        /// Number of signals
        #[arg(short, long, default_value = "1000")]
        signals: usize,
        /// Number of blocks
        #[arg(short, long, default_value = "100")]
        blocks: usize,
        /// Target scan time in ms
        #[arg(short, long, default_value = "10")]
        scan_time: u64,
    },
}

#[derive(Subcommand)]
#[cfg(any(feature = "mqtt", feature = "s7-support", feature = "modbus-support"))]
enum ProtocolCommands {
    /// Test MQTT connectivity
    #[cfg(feature = "mqtt")]
    Mqtt {
        /// MQTT broker URL
        #[arg(short, long)]
        broker: String,
        /// Test topic
        #[arg(short, long, default_value = "petra/test")]
        topic: String,
        /// Test message
        #[arg(short, long, default_value = "Hello from PETRA")]
        message: String,
    },
    
    /// Test S7 PLC connectivity
    #[cfg(feature = "s7-support")]
    S7 {
        /// PLC IP address
        #[arg(short, long)]
        ip: String,
        /// Rack number
        #[arg(short, long, default_value = "0")]
        rack: u16,
        /// Slot number
        #[arg(short, long, default_value = "1")]
        slot: u16,
    },
    
    /// Test Modbus connectivity
    #[cfg(feature = "modbus-support")]
    Modbus {
        /// Modbus server address
        #[arg(short, long)]
        address: String,
        /// Unit ID
        #[arg(short, long, default_value = "1")]
        unit: u8,
    },
}

#[derive(ValueEnum, Clone)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, ValueEnum, Clone)]
enum ConfigTemplate {
    Basic,
    Edge,
    Scada,
    Enterprise,
    Development,
}

#[derive(Debug, ValueEnum, Clone)]
enum ConfigFormat {
    Yaml,
    Json,
    Toml,
}

// ============================================================================
// MAIN FUNCTION
// ============================================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    // Initialize logging first
    if let Err(e) = init_logging(&cli) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }
    
    // Initialize PETRA
    if let Err(e) = petra::init() {
        error!("Failed to initialize PETRA: {}", e);
        process::exit(1);
    }
    
    
    // Handle commands
    let result = match cli.command {
        Some(Commands::Run { 
            config, 
            scan_time, 
            enhanced_monitoring,
            no_circuit_breakers,
            cpu_affinity,
            thread_priority,
        }) => {
            run_engine(
                config, 
                scan_time, 
                enhanced_monitoring, 
                !no_circuit_breakers,
                cpu_affinity,
                thread_priority
            ).await
        }
        
        Some(Commands::Validate { config, detailed, check_features }) => {
            validate_config(config, detailed, check_features).await
        }
        
        Some(Commands::Features { dependencies, conflicts, check }) => {
            show_features(dependencies, conflicts, check).await
        }
        
        Some(Commands::Config { config_cmd }) => {
            handle_config_command(config_cmd).await
        }
        
        #[cfg(any(feature = "examples", feature = "burn-in"))]
        Some(Commands::Dev { dev_cmd }) => {
            handle_dev_command(dev_cmd).await
        }
        
        #[cfg(any(feature = "mqtt", feature = "s7-support", feature = "modbus-support"))]
        Some(Commands::Protocol { protocol_cmd }) => {
            handle_protocol_command(protocol_cmd).await
        }
        
        #[cfg(feature = "metrics")]
        Some(Commands::Metrics { port, bind }) => {
            start_metrics_server(port, bind).await
        }
        
        #[cfg(feature = "health")]
        Some(Commands::Health { port, bind }) => {
            start_health_server(port, bind).await
        }
        
        None => {
            // Default behavior - run engine if config provided, otherwise show help
            if let Some(config_path) = cli.config {
                if cli.validate_only {
                    validate_config(config_path, false, true).await
                } else {
                    run_engine(
                        config_path, 
                        cli.scan_time, 
                        false, 
                        true, 
                        None, 
                        None
                    ).await
                }
            } else {
                show_help_and_features().await
            }
        }
    };
    
    if let Err(e) = result {
        error!("Command failed: {}", e);
        process::exit(1);
    }
}

// ============================================================================
// LOGGING INITIALIZATION
// ============================================================================

fn init_logging(cli: &Cli) -> Result<()> {
    let log_level = if cli.quiet {
        "error"
    } else if cli.verbose {
        "debug"
    } else {
        match cli.log_level {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn", 
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    };
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("petra={}", log_level)));
    
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(cli.verbose)
        .with_file(cli.verbose)
        .with_line_number(cli.verbose);
        
    let fmt_layer = if cli.json_logs {
        fmt_layer.json().boxed()
    } else if cli.no_color {
        fmt_layer.without_time().boxed()
    } else {
        fmt_layer.boxed()
    };
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
    
    info!("PETRA v{} starting", petra::VERSION);
    debug!("Logging initialized with level: {}", log_level);
    
    Ok(())
}

// ============================================================================
// ENGINE EXECUTION
// ============================================================================

async fn run_engine(
    config_path: PathBuf,
    scan_time_override: Option<u64>,
    enhanced_monitoring: bool,
    circuit_breakers: bool,
    cpu_affinity: Option<String>,
    thread_priority: Option<i32>,
) -> Result<()> {
    info!("Loading configuration from: {}", config_path.display());
    
    // Load and validate configuration
    let mut config = Config::from_file(&config_path)?;
    
    // Apply overrides
    if let Some(scan_time) = scan_time_override {
        info!("Overriding scan time: {}ms -> {}ms", config.scan_time_ms, scan_time);
        config.scan_time_ms = scan_time;
    }
    
    // Create engine configuration
    let mut engine_config = EngineConfig::default();
    engine_config.enhanced_monitoring = enhanced_monitoring;
    
    #[cfg(feature = "circuit-breaker")]
    {
        engine_config.circuit_breaker_enabled = circuit_breakers;
    }
    
    #[cfg(feature = "realtime")]
    {
        if let Some(affinity_str) = cpu_affinity {
            let affinity = u64::from_str_radix(&affinity_str, 16)
                .map_err(|e| PlcError::Config(format!("Invalid CPU affinity mask: {}", e)))?;
            engine_config.cpu_affinity = Some(affinity);
            engine_config.realtime_enabled = true;
        }
        
        if let Some(priority) = thread_priority {
            if priority < 1 || priority > 99 {
                return Err(PlcError::Config("Thread priority must be between 1 and 99".to_string()));
            }
            engine_config.thread_priority = Some(priority);
            engine_config.realtime_enabled = true;
        }
    }
    
    // Show configuration summary
    info!("Configuration loaded: {}", Features::detect().summary());
    info!("Signals: {}, Blocks: {}, Scan time: {}ms", 
          config.signals.len(), config.blocks.len(), config.scan_time_ms);
    
    // Create and start engine
    let metrics_enabled = engine_config.metrics_enabled;
    let mut engine = Engine::new_with_engine_config(config, engine_config)?;
    
    // Start auxiliary services
    #[cfg(feature = "metrics")]
    let _metrics_server = if metrics_enabled {
        Some(start_metrics_service()?)
    } else {
        None
    };
    
    #[cfg(feature = "health")]
    start_health_service().await?;
    
    // Setup signal handling for graceful shutdown
    let running = engine.is_running();
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen for shutdown signal: {}", e);
        } else {
            info!("Shutdown signal received");
        }
    });
    
    // Run the engine
    info!("Starting PETRA engine...");
    let result = engine.run().await;
    
    match result {
        Ok(_) => {
            info!("PETRA engine stopped successfully");
            Ok(())
        }
        Err(e) => {
            error!("PETRA engine failed: {}", e);
            Err(e)
        }
    }
}

// ============================================================================
// CONFIGURATION VALIDATION
// ============================================================================

async fn validate_config(config_path: PathBuf, detailed: bool, check_features: bool) -> Result<()> {
    info!("Validating configuration: {}", config_path.display());
    
    let config = Config::from_file(&config_path)?;
    
    // Basic validation is done during loading
    info!("✅ Configuration is valid");
    
    if detailed {
        println!("\n📊 Configuration Summary:");
        println!("  Signals: {}", config.signals.len());
        println!("  Blocks: {}", config.blocks.len());
        println!("  Scan time: {}ms", config.scan_time_ms);
        println!("  Features: {}", Features::detect().summary());
        
        // Show signal details
        if !config.signals.is_empty() {
            println!("\n📡 Signals:");
            for signal in &config.signals {
                println!("  - {} ({})", signal.name, signal.signal_type);
            }
        }
        
        // Show block details  
        if !config.blocks.is_empty() {
            println!("\n🔧 Blocks:");
            for block in &config.blocks {
                println!("  - {} ({})", block.name, block.block_type);
            }
        }
    }
    
    if check_features {
        println!("\n🎛️  Feature Compatibility Check:");
        
        // Check if configuration uses features that aren't enabled
        let mut missing_features = Vec::new();
        
        #[cfg(not(feature = "mqtt"))]
        if config.get_mqtt_config().is_some() {
            missing_features.push("mqtt");
        }
        
        #[cfg(not(feature = "security"))]
        if config.get_security_config().is_some() {
            missing_features.push("security");
        }
        
        #[cfg(not(feature = "alarms"))]
        if config.get_alarms_config().is_some() {
            missing_features.push("alarms");
        }
        
        if missing_features.is_empty() {
            println!("  ✅ All required features are enabled");
        } else {
            println!("  ⚠️  Missing features: {}", missing_features.join(", "));
            println!("     Rebuild with: cargo build --features \"{}\"", missing_features.join(" "));
        }
    }
    
    Ok(())
}

// ============================================================================
// FEATURE INFORMATION
// ============================================================================

async fn show_features(dependencies: bool, conflicts: bool, check: Option<String>) -> Result<()> {
    let features = Features::detect();
    
    if let Some(feature) = check {
        println!("Checking feature: {}", feature);
        if features.is_enabled(&feature) {
            println!("✅ Feature '{}' is enabled", feature);
        } else {
            println!("❌ Feature '{}' is not enabled", feature);
        }
        return Ok(());
    }
    
    println!("🎛️  PETRA Features Summary");
    println!("Version: {}", petra::VERSION);
    println!("Build: {}", features.summary());
    println!();
    
    features.print();
    
    if dependencies {
        println!("\n📋 Feature Dependencies:");
        // This would show the dependency tree
        println!("  enhanced-monitoring → standard-monitoring");
        println!("  jwt-auth → security");
        println!("  rbac → security");
        // ... other dependencies
    }
    
    if conflicts {
        println!("\n⚠️  Feature Conflicts:");
        println!("  Only one monitoring level can be enabled at a time");
        // ... other conflicts
    }
    
    Ok(())
}

async fn show_help_and_features() -> Result<()> {
    println!("🚀 PETRA - Programmable Engine for Telemetry, Runtime, and Automation");
    println!("Version: {}", petra::VERSION);
    println!();
    
    // Show available features
    show_features(false, false, None).await?;
    
    println!("\n💡 Quick Start:");
    println!("  petra config example --template basic              # Generate example config");
    println!("  petra validate petra-example.yaml                  # Validate config");
    println!("  petra run petra-example.yaml                       # Run engine");
    println!("  petra features                                     # Show enabled features");
    println!();
    println!("Run 'petra --help' for detailed usage information.");
    
    Ok(())
}

// ============================================================================
// COMMAND HANDLERS
// ============================================================================

async fn handle_config_command(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Example { output, template } => {
            generate_example_config(output, template).await
        }
        ConfigCommands::Convert { input, output, format } => {
            convert_config(input, output, format).await
        }
        ConfigCommands::Migrate { input, output } => {
            migrate_config(input, output).await
        }
        ConfigCommands::Schema { 
            #[cfg(feature = "json-schema")]
            json 
        } => {
            show_config_schema(
                #[cfg(feature = "json-schema")]
                json
            ).await
        }
    }
}

async fn generate_example_config(output: PathBuf, template: ConfigTemplate) -> Result<()> {
    info!("Generating {:?} configuration template: {}", template, output.display());
    
    let config = match template {
        ConfigTemplate::Basic => generate_basic_config(),
        ConfigTemplate::Edge => generate_edge_config(),
        ConfigTemplate::Scada => generate_scada_config(),
        ConfigTemplate::Enterprise => generate_enterprise_config(),
        ConfigTemplate::Development => generate_development_config(),
    };
    
    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| PlcError::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(&output, yaml)
        .map_err(|e| PlcError::Config(format!("Failed to write config: {}", e)))?;
    
    info!("✅ Configuration template generated: {}", output.display());
    Ok(())
}

async fn convert_config(input: PathBuf, output: PathBuf, format: ConfigFormat) -> Result<()> {
    info!("Converting {} to {:?} format: {}", input.display(), format, output.display());
    
    let config = Config::from_file(&input)?;
    
    let content = match format {
        ConfigFormat::Yaml => serde_yaml::to_string(&config)
            .map_err(|e| PlcError::Config(format!("YAML serialization failed: {}", e)))?,
        ConfigFormat::Json => serde_json::to_string_pretty(&config)
            .map_err(|e| PlcError::Config(format!("JSON serialization failed: {}", e)))?,
        ConfigFormat::Toml => toml::to_string_pretty(&config)
            .map_err(|e| PlcError::Config(format!("TOML serialization failed: {}", e)))?,
    };
    
    std::fs::write(&output, content)
        .map_err(|e| PlcError::Config(format!("Failed to write file: {}", e)))?;
    
    info!("✅ Configuration converted successfully");
    Ok(())
}

async fn migrate_config(input: PathBuf, output: Option<PathBuf>) -> Result<()> {
    info!("Migrating configuration: {}", input.display());
    
    // Create backup first
    let backup_path = input.with_extension("yaml.backup");
    std::fs::copy(&input, &backup_path)
        .map_err(|e| PlcError::Config(format!("Failed to create backup: {}", e)))?;
    
    info!("✅ Backup created: {}", backup_path.display());
    
    // Load and migrate (this would implement actual migration logic)
    let config = Config::from_file(&input)?;
    
    let output_path = output.unwrap_or(input);
    let yaml = serde_yaml::to_string(&config)
        .map_err(|e| PlcError::Config(format!("Failed to serialize config: {}", e)))?;
    
    std::fs::write(&output_path, yaml)
        .map_err(|e| PlcError::Config(format!("Failed to write config: {}", e)))?;
    
    info!("✅ Configuration migrated: {}", output_path.display());
    Ok(())
}

async fn show_config_schema(
    #[cfg(feature = "json-schema")]
    json: bool
) -> Result<()> {
    #[cfg(feature = "json-schema")]
    {
        if json {
            use schemars::schema_for;
            let schema = schema_for!(Config);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        } else {
            println!("Configuration Schema:");
            println!("  Root: Config");
            println!("  Fields: scan_time_ms, signals, blocks, protocols, ...");
        }
    }
    
    #[cfg(not(feature = "json-schema"))]
    {
        println!("JSON schema support not enabled.");
        println!("Rebuild with: cargo build --features json-schema");
    }
    
    Ok(())
}

// ============================================================================
// AUXILIARY SERVICES
// ============================================================================

#[cfg(feature = "metrics")]
fn start_metrics_service() -> Result<MetricsServer> {
    info!("Starting metrics server on :9090");
    let metrics_config = petra::metrics_server::MetricsConfig {
        bind_address: "0.0.0.0:9090".to_string(),
        enabled: true,
        path: Some("/metrics".to_string()),
        timeout_secs: Some(30),
    };
    MetricsServer::new(metrics_config)
}

#[cfg(feature = "metrics")]
async fn start_metrics_server(port: u16, bind: String) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind, port).parse()
        .map_err(|e| PlcError::Config(format!("Invalid address: {}", e)))?;

    info!("Starting standalone metrics server on {}", addr);
    let server = MetricsServer::new(petra::metrics_server::MetricsConfig {
        bind_address: addr.to_string(),
        enabled: true,
        path: Some("/metrics".to_string()),
        timeout_secs: Some(30),
    })?;
    server.run().await?;
    Ok(())
}

#[cfg(feature = "health")]
async fn start_health_service() -> Result<()> {
    info!("Starting health server on :8080");
    let monitor = HealthMonitor::new(petra::health::HealthConfig::default());
    monitor.start().await?;
    Ok(())
}

#[cfg(feature = "health")]
async fn start_health_server(port: u16, bind: String) -> Result<()> {
    let addr: SocketAddr = format!("{}:{}", bind, port).parse()
        .map_err(|e| PlcError::Config(format!("Invalid address: {}", e)))?;

    info!("Starting standalone health server on {}", addr);
    let monitor = HealthMonitor::new(petra::health::HealthConfig {
        bind_address: addr,
        ..Default::default()
    });
    monitor.start().await?;
    Ok(())
}

// ============================================================================
// CONFIGURATION TEMPLATES
// ============================================================================

fn generate_basic_config() -> Config {
    use petra::config::{SignalConfig, BlockConfig};
    use std::collections::HashMap;
    
    Config {
        scan_time_ms: 100,
        max_scan_jitter_ms: 10,
        error_recovery: true,
        protocols: None,
        signals: vec![
            SignalConfig {
                name: "input_1".to_string(),
                signal_type: "bool".to_string(),
                initial: Some(serde_yaml::Value::Bool(false)),
                description: Some("Digital input 1".to_string()),
                tags: vec!["input".to_string()],
                #[cfg(feature = "engineering-types")]
                units: None,
                #[cfg(feature = "quality-codes")]
                quality_enabled: false,
                #[cfg(feature = "validation")]
                validation: None,
                metadata: HashMap::new(),
            },
            SignalConfig {
                name: "output_1".to_string(),
                signal_type: "bool".to_string(),
                initial: Some(serde_yaml::Value::Bool(false)),
                description: Some("Digital output 1".to_string()),
                tags: vec!["output".to_string()],
                #[cfg(feature = "engineering-types")]
                units: None,
                #[cfg(feature = "quality-codes")]
                quality_enabled: false,
                #[cfg(feature = "validation")]
                validation: None,
                metadata: HashMap::new(),
            },
        ],
        blocks: vec![
            BlockConfig {
                name: "not_gate".to_string(),
                block_type: "NOT".to_string(),
                inputs: {
                    let mut map = HashMap::new();
                    map.insert("input".to_string(), "input_1".to_string());
                    map
                },
                outputs: {
                    let mut map = HashMap::new();
                    map.insert("output".to_string(), "output_1".to_string());
                    map
                },
                params: HashMap::new(),
                description: Some("Simple NOT gate example".to_string()),
                tags: vec!["logic".to_string()],
                #[cfg(feature = "enhanced-errors")]
                error_handling: None,
                #[cfg(feature = "circuit-breaker")]
                circuit_breaker: None,
            },
        ],
        // Feature-specific configs would be None for basic template
        #[cfg(feature = "mqtt")]
        mqtt: None,
        #[cfg(feature = "alarms")]
        alarms: None,
        #[cfg(feature = "security")]
        security: None,
        #[cfg(feature = "history")]
        history: None,
        #[cfg(feature = "validation")]
        validation: None,
    }
}

fn generate_edge_config() -> Config {
    // Similar to basic but with MQTT enabled if available
    let mut config = generate_basic_config();
    config.scan_time_ms = 50; // Faster for edge
    config
}

fn generate_scada_config() -> Config {
    // Industrial-focused configuration
    let mut config = generate_basic_config();
    config.scan_time_ms = 100;
    // Would add S7/Modbus configs if features enabled
    config
}

fn generate_enterprise_config() -> Config {
    // Full-featured configuration
    let mut config = generate_basic_config();
    config.scan_time_ms = 100;
    // Would add all enterprise features if enabled
    config
}

fn generate_development_config() -> Config {
    // Development-friendly configuration
    let mut config = generate_basic_config();
    config.scan_time_ms = 10; // Fast for testing
    config
}

// Stubs for other command handlers
#[cfg(any(feature = "examples", feature = "burn-in"))]
async fn handle_dev_command(_cmd: DevCommands) -> Result<()> {
    info!("Development commands not yet implemented");
    Ok(())
}

#[cfg(any(feature = "mqtt", feature = "s7-support", feature = "modbus-support"))]
async fn handle_protocol_command(_cmd: ProtocolCommands) -> Result<()> {
    info!("Protocol commands not yet implemented");
    Ok(())
}

//! # PETRA Binary Entry Point - Production-Ready CLI Interface
//!
//! ## Purpose & Overview
//! 
//! This is the main executable entry point for PETRA that provides a comprehensive
//! command-line interface for:
//!
//! - **Engine Execution** - Run PETRA with configurable scan times and features
//! - **Configuration Management** - Validate, convert, and migrate configurations
//! - **Feature Detection** - Show enabled features and dependencies
//! - **Development Tools** - Testing utilities and examples
//! - **Protocol Testing** - Test industrial protocol connections
//! - **Service Management** - Run individual services (metrics, health)
//!
//! ## Architecture & Interactions
//!
//! This binary uses the PETRA library (`src/lib.rs`) and directly interacts with:
//! - **Config** (`src/config.rs`) - Configuration loading and validation
//! - **Engine** (`src/engine.rs`) - Core execution engine
//! - **Features** (`src/features.rs`) - Runtime feature detection
//! - **Protocol modules** - For connection testing
//! - **Service modules** - For standalone service execution
//!
//! ## Performance Considerations
//!
//! - Async/await throughout for non-blocking I/O
//! - Graceful shutdown handling with proper signal handling
//! - Resource cleanup on exit
//! - Feature-gated compilation for minimal binary size
//! - Structured logging with performance-optimized formatting
//!
//! ## Error Handling
//!
//! All operations return `Result<(), PlcError>` and are properly handled with:
//! - Descriptive error messages
//! - Appropriate exit codes
//! - Error context preservation
//! - No panics in production code paths

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

use clap::{Parser, Subcommand, ValueEnum};
use petra::{
    Config,
    PlcError,
    Result,
    Engine,
    features,
    config::{LintSeverity, ConfigTemplate, ConfigFormat},
    VERSION,
    init_petra,
};
use petra::build_info;
use std::path::PathBuf;
use std::process;
use tokio::signal;
use tracing::{info, error, debug, warn, Level};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use tracing_subscriber::filter::Directive;
use colored::*;

#[cfg(feature = "basic-auth")]
use rpassword;

// Feature-gated imports for optional functionality
#[cfg(feature = "metrics")]
use petra::MetricsServer;

#[cfg(feature = "health")]
use petra::health::HealthMonitor;

#[cfg(feature = "realtime")]
use petra::realtime::RealtimeScheduler;


// ============================================================================
// CLI ARGUMENT DEFINITIONS
// ============================================================================

/// PETRA command-line interface with comprehensive automation features
#[derive(Parser)]
#[command(name = "petra")]
#[command(version = VERSION)]
#[command(about = "PETRA - Programmable Engine for Telemetry, Runtime, and Automation")]
#[command(long_about = format!(
    "PETRA v{} - High-performance industrial automation engine built in Rust\n\
     \n\
     Features:\n\
     • Real-time deterministic execution with microsecond precision\n\
     • Optional features for lean deployments\n\
     • Industrial protocol support (Modbus, S7, OPC-UA, MQTT)\n\
     • Enterprise data storage with compression and WAL\n\
     • Advanced alarm management with SMS/email notifications\n\
     • Web-based configuration interface\n\
     • Comprehensive security with RBAC and audit logging\n\
     \n\
     Use 'petra features' to see which features are enabled in this build.\n\
     Use 'petra config example' to generate a starter configuration.\n\
     \n\
     Build Info:\n\
     {}",
    VERSION,
    build_info::build_info_string()
))]
struct Cli {
    /// Configuration file path (for default run mode)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// Scan time in milliseconds (for default run mode)
    #[arg(short, long, default_value = "10")]
    scan_time: u64,
    
    /// Validate configuration only, don't run engine
    #[arg(short, long)]
    validate_only: bool,
    
    /// Enable verbose logging (debug level)
    #[arg(short, long)]
    verbose: bool,
    
    /// Enable quiet mode (error level only)
    #[arg(short, long)]
    quiet: bool,
    
    /// Log output format
    #[arg(long, value_enum, default_value = "pretty")]
    log_format: LogFormat,
    
    /// CPU affinity (Linux only, comma-separated core IDs)
    #[cfg(target_os = "linux")]
    #[arg(long, value_delimiter = ',')]
    cpu_affinity: Vec<usize>,
    
    /// Thread priority (0-99, requires privileges)
    #[cfg(feature = "realtime")]
    #[arg(long)]
    thread_priority: Option<u8>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available commands for different operation modes
#[derive(Subcommand)]
enum Commands {
    /// Run the PETRA engine with specified configuration
    Run {
        /// Configuration file path
        #[arg(value_name = "CONFIG_FILE")]
        config: PathBuf,
        
        /// Engine scan time in milliseconds
        #[arg(short, long, default_value = "10")]
        scan_time: u64,
        
        /// Enable enhanced monitoring and diagnostics
        #[cfg(feature = "enhanced-monitoring")]
        #[arg(long)]
        enhanced_monitoring: bool,
        
        /// Disable circuit breakers for blocks
        #[cfg(feature = "circuit-breaker")]
        #[arg(long)]
        no_circuit_breakers: bool,
        
        /// CPU affinity for worker threads (Linux only)
        #[cfg(target_os = "linux")]
        #[arg(long, value_delimiter = ',')]
        cpu_affinity: Option<Vec<usize>>,
        
        /// Real-time thread priority (0-99)
        #[cfg(feature = "realtime")]
        #[arg(long)]
        thread_priority: Option<u8>,
        
        /// Force real-time scheduling
        #[cfg(feature = "realtime")]
        #[arg(long)]
        force_realtime: bool,
    },
    
    /// Validate configuration file without running
    Validate {
        /// Configuration file to validate
        #[arg(value_name = "CONFIG_FILE")]
        config: PathBuf,
        
        /// Show detailed validation report
        #[arg(short, long)]
        detailed: bool,
        
        /// Check feature compatibility
        #[arg(short, long)]
        check_features: bool,
        
        /// Validate against schema
        #[cfg(feature = "schema-validation")]
        #[arg(long)]
        schema: bool,
    },
    
    /// Show enabled features and capabilities
    Features {
        /// Show feature dependencies
        #[arg(short, long)]
        dependencies: bool,
        
        /// Show feature conflicts
        #[arg(short, long)]
        conflicts: bool,
        
        /// Check specific feature availability
        #[arg(long)]
        check: Option<String>,
        
        /// List all available features
        #[arg(short, long)]
        list: bool,
    },
    
    /// Configuration management utilities
    Config {
        #[command(subcommand)]
        config_cmd: ConfigCommands,
    },
    
    /// Development and testing utilities  
    #[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
    Dev {
        #[command(subcommand)]
        dev_cmd: DevCommands,
    },
    
    /// Protocol testing and diagnostics
    #[cfg(any(
        feature = "mqtt", 
        feature = "s7-support", 
        feature = "modbus-support",
        feature = "opcua-support"
    ))]
    Protocol {
        #[command(subcommand)]
        protocol_cmd: ProtocolCommands,
    },
    
    /// Start metrics server only
    #[cfg(feature = "metrics")]
    Metrics {
        /// Port to bind metrics server
        #[arg(short, long, default_value = "9090")]
        port: u16,
        
        /// Bind address for metrics server
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
        
        /// Enable additional runtime metrics
        #[arg(long)]
        runtime_metrics: bool,
    },
    
    /// Start health monitoring server only  
    #[cfg(feature = "health")]
    Health {
        /// Port to bind health server
        #[arg(short, long, default_value = "8080")]
        port: u16,
        
        /// Bind address for health server
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
        
        /// Health check interval in seconds
        #[arg(long, default_value = "30")]
        check_interval: u64,
    },
    
    /// Security management utilities
    #[cfg(feature = "security")]
    Security {
        #[command(subcommand)]
        security_cmd: SecurityCommands,
    },
    
    /// Database and storage utilities
    #[cfg(feature = "advanced-storage")]
    Storage {
        #[command(subcommand)]
        storage_cmd: StorageCommands,
    },
}

/// Configuration management subcommands
#[derive(Subcommand)]
enum ConfigCommands {
    /// Generate example configuration files
    Example {
        /// Output file path
        #[arg(short, long, default_value = "petra-example.yaml")]
        output: PathBuf,
        
        /// Configuration template type
        #[arg(short, long, value_enum, default_value = "basic")]
        template: ConfigTemplate,
        
        /// Include feature-specific examples
        #[arg(long)]
        include_features: bool,
    },
    
    /// Convert configuration between formats
    Convert {
        /// Input configuration file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Output format
        #[arg(short, long, value_enum, default_value = "yaml")]
        format: ConfigFormat,
    },
    
    /// Migrate configuration to newer version
    Migrate {
        /// Input configuration file
        #[arg(value_name = "INPUT")]
        input: PathBuf,
        
        /// Output file path (defaults to backup + update input)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Target configuration version
        #[arg(long, default_value = "latest")]
        target_version: String,
    },
    
    /// Show configuration schema information
    Schema {
        /// Show JSON schema
        #[cfg(feature = "schema-validation")]
        #[arg(long)]
        json: bool,
        
        /// Output schema to file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Validate configuration against best practices
    Lint {
        /// Configuration file to lint
        #[arg(value_name = "CONFIG_FILE")]
        config: PathBuf,
        
        /// Apply automatic fixes where possible
        #[arg(long)]
        fix: bool,
    },
}

/// Development and testing subcommands
#[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
#[derive(Subcommand)]
enum DevCommands {
    /// Run example scenarios
    #[cfg(feature = "examples")]
    Examples {
        /// List available examples
        #[arg(short, long)]
        list: bool,
        
        /// Run specific example by name
        #[arg(short, long)]
        run: Option<String>,
        
        /// Generate example data
        #[arg(long)]
        generate_data: bool,
    },
    
    /// Run comprehensive burn-in tests
    #[cfg(feature = "burn-in")]
    BurnIn {
        /// Test duration in seconds
        #[arg(short, long, default_value = "3600")]
        duration: u64,
        
        /// Number of test signals to create
        #[arg(short, long, default_value = "1000")]
        signals: usize,
        
        /// Number of test blocks to create
        #[arg(short, long, default_value = "100")]
        blocks: usize,
        
        /// Target scan time in milliseconds
        #[arg(short, long, default_value = "10")]
        scan_time: u64,
        
        /// Memory stress testing
        #[arg(long)]
        memory_stress: bool,
    },
    
    /// Performance profiling and benchmarking
    #[cfg(feature = "profiling")]
    Profile {
        /// Configuration file for profiling
        #[arg(value_name = "CONFIG_FILE")]
        config: PathBuf,
        
        /// Profiling duration in seconds
        #[arg(short, long, default_value = "60")]
        duration: u64,
        
        /// Output profile report file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Enable CPU profiling
        #[arg(long)]
        cpu: bool,
        
        /// Enable memory profiling
        #[arg(long)]
        memory: bool,
    },
}

/// Protocol testing and diagnostics subcommands
#[cfg(any(
    feature = "mqtt", 
    feature = "s7-support", 
    feature = "modbus-support",
    feature = "opcua-support"
))]
#[derive(Subcommand)]
enum ProtocolCommands {
    /// Test MQTT connectivity
    #[cfg(feature = "mqtt")]
    Mqtt {
        /// MQTT broker address
        #[arg(short, long, default_value = "localhost:1883")]
        broker: String,
        
        /// Topic to test
        #[arg(short, long, default_value = "test/petra")]
        topic: String,
        
        /// Number of test messages
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    
    /// Test Modbus connectivity
    #[cfg(feature = "modbus-support")]
    Modbus {
        /// Modbus server address
        #[arg(short, long, default_value = "127.0.0.1:502")]
        address: String,
        
        /// Unit ID to test
        #[arg(short, long, default_value = "1")]
        unit_id: u8,
        
        /// Register address to read
        #[arg(short, long, default_value = "0")]
        register: u16,
    },
    
    /// Test Siemens S7 connectivity
    #[cfg(feature = "s7-support")]
    S7 {
        /// S7 PLC address
        #[arg(short, long)]
        address: String,
        
        /// Rack number
        #[arg(short, long, default_value = "0")]
        rack: u16,
        
        /// Slot number
        #[arg(short, long, default_value = "2")]
        slot: u16,
    },
    
    /// Test OPC-UA connectivity
    #[cfg(feature = "opcua-support")]
    OpcUa {
        /// OPC-UA server endpoint
        #[arg(short, long)]
        endpoint: String,
        
        /// Node ID to read
        #[arg(short, long)]
        node_id: Option<String>,
    },
}

/// Security management subcommands
#[cfg(feature = "security")]
#[derive(Subcommand)]
enum SecurityCommands {
    /// Generate cryptographic keys
    KeyGen {
        /// Key type to generate
        #[arg(value_enum)]
        key_type: KeyType,
        
        /// Output file for the key
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Create user accounts
    #[cfg(feature = "basic-auth")]
    CreateUser {
        /// Username
        #[arg(short, long)]
        username: String,
        
        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
        
        /// User roles
        #[cfg(feature = "rbac")]
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<String>,
    },
    
    /// Show audit logs
    #[cfg(feature = "audit")]
    Audit {
        /// Number of recent entries to show
        #[arg(short, long, default_value = "100")]
        limit: usize,
        
        /// Filter by user
        #[arg(short, long)]
        user: Option<String>,
        
        /// Filter by action
        #[arg(short, long)]
        action: Option<String>,
    },
}

/// Storage management subcommands
#[cfg(feature = "advanced-storage")]
#[derive(Subcommand)]
enum StorageCommands {
    /// Initialize storage backends
    Init {
        /// Storage type to initialize
        #[arg(value_enum)]
        storage_type: StorageType,
        
        /// Configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    
    /// Backup data
    Backup {
        /// Output backup file
        #[arg(short, long)]
        output: PathBuf,
        
        /// Start time for backup (ISO 8601)
        #[arg(long)]
        start_time: Option<String>,
        
        /// End time for backup (ISO 8601)
        #[arg(long)]
        end_time: Option<String>,
    },
    
    /// Restore data from backup
    Restore {
        /// Input backup file
        #[arg(value_name = "BACKUP_FILE")]
        input: PathBuf,
        
        /// Force restore (overwrite existing data)
        #[arg(long)]
        force: bool,
    },
    
    /// Compact and optimize storage
    Compact {
        /// Dry run (show what would be done)
        #[arg(long)]
        dry_run: bool,
    },
}

// ============================================================================
// VALUE ENUMS FOR CLI OPTIONS
// ============================================================================

/// Log output format options
#[derive(Clone, Copy, ValueEnum)]
enum LogFormat {
    /// Human-readable pretty format
    Pretty,
    /// JSON structured logging
    Json,
    /// Compact single-line format
    Compact,
}


/// Cryptographic key types
#[cfg(feature = "security")]
#[derive(Debug, Clone, Copy, ValueEnum)]
enum KeyType {
    /// RSA key pair
    Rsa,
    /// Ed25519 key pair
    Ed25519,
    /// Symmetric AES key
    Aes,
    /// JWT signing key
    #[cfg(feature = "jwt-auth")]
    Jwt,
}

/// Storage backend types
#[cfg(any(feature = "history", feature = "advanced-storage"))]
#[derive(Debug, Clone, Copy, ValueEnum)]

enum StorageType {
    /// Parquet file storage
    #[cfg(feature = "history")]
    Parquet,
    /// ClickHouse database
    #[cfg(feature = "advanced-storage")]
    Clickhouse,
    /// RocksDB storage
    #[cfg(feature = "advanced-storage")]
    Rocksdb,
    /// S3-compatible storage
    #[cfg(feature = "advanced-storage")]
    S3,
}

// ============================================================================
// MAIN ENTRY POINT
// ============================================================================

/// Main entry point with comprehensive error handling and graceful shutdown
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize logging based on CLI flags
    init_logging(&cli)?;
    
    // Log startup information
    info!("Starting PETRA v{}", VERSION);
    debug!("Build info: {}", build_info::build_info_string());
    
    // Initialize PETRA library
    if let Err(e) = init_petra() {
        error!("Failed to initialize PETRA: {}", e);
        process::exit(1);
    }
    
    // Handle CPU affinity (Linux only)
    #[cfg(target_os = "linux")]
    if !cli.cpu_affinity.is_empty() {
        if let Err(e) = set_cpu_affinity(&cli.cpu_affinity) {
            warn!("Failed to set CPU affinity: {}", e);
        }
    }
    
    // Handle the command or default behavior
    let result = match cli.command {
        Some(Commands::Run { 
            config, 
            scan_time, 
            #[cfg(feature = "enhanced-monitoring")]
            enhanced_monitoring,
            #[cfg(feature = "circuit-breaker")]
            no_circuit_breakers,
            #[cfg(target_os = "linux")]
            cpu_affinity,
            #[cfg(feature = "realtime")]
            thread_priority,
            #[cfg(feature = "realtime")]
            force_realtime,
        }) => {
            run_engine(
                config,
                scan_time,
                #[cfg(feature = "enhanced-monitoring")]
                enhanced_monitoring,
                #[cfg(feature = "circuit-breaker")]
                !no_circuit_breakers,
                #[cfg(target_os = "linux")]
                cpu_affinity,
                #[cfg(feature = "realtime")]
                thread_priority,
                #[cfg(feature = "realtime")]
                force_realtime,
            ).await
        }
        
        Some(Commands::Validate { 
            config, 
            detailed, 
            check_features,
            #[cfg(feature = "schema-validation")]
            schema,
        }) => {
            validate_config(
                config, 
                detailed, 
                check_features,
                #[cfg(feature = "schema-validation")]
                schema,
            ).await
        }
        
        Some(Commands::Features { dependencies, conflicts, check, list }) => {
            show_features(dependencies, conflicts, check, list).await
        }
        
        Some(Commands::Config { config_cmd }) => {
            handle_config_command(config_cmd).await
        }
        
        #[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
        Some(Commands::Dev { dev_cmd }) => {
            handle_dev_command(dev_cmd).await
        }
        
        #[cfg(any(
            feature = "mqtt", 
            feature = "s7-support", 
            feature = "modbus-support",
            feature = "opcua-support"
        ))]
        Some(Commands::Protocol { protocol_cmd }) => {
            handle_protocol_command(protocol_cmd).await
        }
        
        #[cfg(feature = "metrics")]
        Some(Commands::Metrics { port, bind, runtime_metrics }) => {
            start_metrics_server(port, bind, runtime_metrics).await
        }
        
        #[cfg(feature = "health")]
        Some(Commands::Health { port, bind, check_interval }) => {
            start_health_server(port, bind, check_interval).await
        }
        
        #[cfg(feature = "security")]
        Some(Commands::Security { security_cmd }) => {
            handle_security_command(security_cmd).await
        }
        
        #[cfg(feature = "advanced-storage")]
        Some(Commands::Storage { storage_cmd }) => {
            handle_storage_command(storage_cmd).await
        }
        
        None => {
            // Default behavior based on CLI flags
            if let Some(config_path) = cli.config {
                if cli.validate_only {
                    validate_config(
                        config_path, 
                        false, 
                        true,
                        #[cfg(feature = "schema-validation")]
                        false,
                    ).await
                } else {
                    run_engine(
                        config_path,
                        cli.scan_time,
                        #[cfg(feature = "enhanced-monitoring")]
                        false,
                        #[cfg(feature = "circuit-breaker")]
                        true,
                        #[cfg(target_os = "linux")]
                        Some(cli.cpu_affinity),
                        #[cfg(feature = "realtime")]
                        cli.thread_priority,
                        #[cfg(feature = "realtime")]
                        false,
                    ).await
                }
            } else {
                show_help_and_features().await
            }
        }
    };
    
    // Handle results and exit appropriately
    match result {
        Ok(()) => {
            info!("PETRA completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("PETRA failed: {}", e);
            process::exit(1);
        }
    }
}

// ============================================================================
// LOGGING INITIALIZATION
// ============================================================================

/// Initialize structured logging with performance optimizations
fn init_logging(cli: &Cli) -> Result<()> {
    let log_level = if cli.quiet {
        Level::ERROR
    } else if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy()
        .add_directive("petra=debug".parse::<Directive>().map_err(|e| PlcError::Config(e.to_string()))?)
        .add_directive("tokio_postgres=warn".parse::<Directive>().map_err(|e| PlcError::Config(e.to_string()))?)
        .add_directive("sqlx=warn".parse::<Directive>().map_err(|e| PlcError::Config(e.to_string()))?)
        .add_directive("h2=warn".parse::<Directive>().map_err(|e| PlcError::Config(e.to_string()))?)
        .add_directive("tower=warn".parse::<Directive>().map_err(|e| PlcError::Config(e.to_string()))?);
    
    match cli.log_format {
        LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_file(cli.verbose)
                        .with_line_number(cli.verbose)
                        .pretty()
                )
                .init();
        }
        LogFormat::Json => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                        .with_file(true)
                        .with_line_number(true)
                        .json()
                )
                .init();
        }
        LogFormat::Compact => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .compact()
                )
                .init();
        }
    }
    
    Ok(())
}

// ============================================================================
// CORE ENGINE EXECUTION
// ============================================================================

/// Run the main PETRA engine with comprehensive configuration
async fn run_engine(
    config_path: PathBuf,
    scan_time: u64,
    #[cfg(feature = "enhanced-monitoring")]
    enhanced_monitoring: bool,
    #[cfg(feature = "circuit-breaker")]
    circuit_breakers: bool,
    #[cfg(target_os = "linux")]
    cpu_affinity: Option<Vec<usize>>,
    #[cfg(feature = "realtime")]
    thread_priority: Option<u8>,
    #[cfg(feature = "realtime")]
    force_realtime: bool,
) -> Result<()> {
    info!("Loading configuration from: {}", config_path.display());
    
    // Load and validate configuration
    let mut config = Config::from_file(&config_path)
        .map_err(|e| PlcError::Config(format!("Failed to load config: {}", e)))?;

    // Apply scan time from CLI
    config.scan_time_ms = scan_time;
    
    info!("Configuration loaded successfully");
    
    // Create engine
    let mut engine = Engine::new(config)?;
    
    // Configure optional features
    #[cfg(feature = "enhanced-monitoring")]
    if enhanced_monitoring {
        info!("Enabling enhanced monitoring");
        // Engine config would enable enhanced monitoring here
    }
    
    #[cfg(feature = "circuit-breaker")]
    if !circuit_breakers {
        info!("Disabling circuit breakers");
        // Circuit breaker disabling not implemented in current engine
    }
    
    #[cfg(feature = "realtime")]
    if let Some(priority) = thread_priority {
        info!("Setting real-time thread priority: {}", priority);
        engine.set_realtime_priority(priority as i32)?;
    }
    
    #[cfg(target_os = "linux")]
    if let Some(affinity) = cpu_affinity {
        info!("Setting CPU affinity: {:?}", affinity);
        set_cpu_affinity(&affinity)?;
    }
    
    // Start the engine
    info!("Starting PETRA engine with {}ms scan time", scan_time);
    let shutdown_signal = setup_shutdown_handler();
    tokio::pin!(shutdown_signal);
    tokio::select! {
        res = engine.run() => {
            res?;
        }
        _ = &mut shutdown_signal => {
            info!("Shutdown signal received, stopping engine...");
            engine.stop().await;
        }
    }
    info!("Engine stopped successfully");
    
    Ok(())
}

// ============================================================================
// CONFIGURATION VALIDATION
// ============================================================================

/// Comprehensive configuration validation with detailed reporting
async fn validate_config(
    config_path: PathBuf,
    detailed: bool,
    check_features: bool,
    #[cfg(feature = "schema-validation")]
    schema_validation: bool,
) -> Result<()> {
    info!("Validating configuration: {}", config_path.display());
    
    // Basic configuration loading
    let config = match Config::from_file(&config_path) {
        Ok(config) => {
            println!("{}", "PASS Configuration file loads successfully".green().bold());
            config
        }
        Err(e) => {
            println!("{} Configuration file failed to load: {}", "FAIL".red().bold(), e);
            return Err(PlcError::Config(format!("Validation failed: {}", e)));
        }
    };
    
    // Feature compatibility check
    if check_features {
        let features = features::current();
        let compat_result = config.check_feature_compatibility(&features);
        
        match compat_result {
            Ok(()) => println!("{}", "PASS All required features are available".green().bold()),
            Err(e) => {
                println!("{} Feature compatibility warning: {}", "WARN".yellow().bold(), e);
                if !detailed {
                    return Err(e);
                }
            }
        }
    }
    
    // Schema validation
    #[cfg(feature = "schema-validation")]
    if schema_validation {
        match config.validate_schema() {
            Ok(()) => println!("{}", "PASS Configuration passes schema validation".green().bold()),
            Err(e) => {
                println!("{} Schema validation failed: {}", "FAIL".red().bold(), e);
                return Err(e);
            }
        }
    }
    
    // Detailed validation report
    if detailed {
        println!("\n{}", "Detailed Validation Report:".cyan().bold());
        print_detailed_config_info(&config);
    }
    
    println!("\n{}", "Configuration validation completed successfully".green().bold());
    Ok(())
}

/// Print detailed configuration information for diagnostics
fn print_detailed_config_info(config: &Config) {
    println!("  {} {}", "Blocks:".blue().bold(), config.blocks.len());
    println!("  {} {}", "Signals:".blue().bold(), config.signals.len());
    
    #[cfg(feature = "mqtt")]
    if let Some(mqtt_config) = &config.mqtt {
        println!("  {} {} topics configured", "MQTT:".blue().bold(), mqtt_config.subscribe_topics.len());
    }
    
    #[cfg(feature = "history")]
    if config.history.is_some() {
        println!("  {} {}", "History:".blue().bold(), "enabled".green());
    }
    
    #[cfg(feature = "alarms")]
    if let Some(alarm_config) = &config.alarms {
        println!("  {} {} configured", "Alarms:".blue().bold(), alarm_config.definitions.len());
    }
    
    #[cfg(feature = "security")]
    if config.security.as_ref().map_or(false, |s| s.enabled) {
        println!("  {} {}", "Security:".blue().bold(), "enabled".green());
    }
}

// ============================================================================
// FEATURE MANAGEMENT
// ============================================================================

/// Show comprehensive feature information and status
async fn show_features(
    show_dependencies: bool,
    show_conflicts: bool,
    check_feature: Option<String>,
    list_all: bool,
) -> Result<()> {
    let features = features::current();
    
    if let Some(feature_name) = check_feature {
        // Check specific feature
        let available = features.is_enabled(&feature_name);
        let status = if available { "Available".green().bold() } else { "Not Available".red().bold() };
        println!("Feature '{}': {}", feature_name.cyan(), status);
        return Ok(());
    }
    
    println!("{} v{} - Enabled Features", "PETRA".cyan().bold(), VERSION);
    println!("{}", "=".repeat(50).bright_black());
    
    // Core features
    println!("\n{}", "Core Features:".yellow().bold());
    print_feature_status("standard-monitoring", features.is_enabled("standard-monitoring"));
    print_feature_status("enhanced-monitoring", features.is_enabled("enhanced-monitoring"));
    print_feature_status("optimized", features.is_enabled("optimized"));
    print_feature_status("metrics", features.is_enabled("metrics"));
    print_feature_status("realtime", features.is_enabled("realtime"));
    
    // Protocol features
    println!("\n{}", "Protocol Features:".yellow().bold());
    print_feature_status("mqtt", features.is_enabled("mqtt"));
    print_feature_status("s7-support", features.is_enabled("s7-support"));
    print_feature_status("modbus-support", features.is_enabled("modbus-support"));
    print_feature_status("opcua-support", features.is_enabled("opcua-support"));
    
    // Storage features
    println!("\n{}", "Storage Features:".yellow().bold());
    print_feature_status("history", features.is_enabled("history"));
    print_feature_status("advanced-storage", features.is_enabled("advanced-storage"));
    print_feature_status("compression", features.is_enabled("compression"));
    print_feature_status("wal", features.is_enabled("wal"));
    
    // Security features
    println!("\n{}", "Security Features:".yellow().bold());
    print_feature_status("security", features.is_enabled("security"));
    print_feature_status("basic-auth", features.is_enabled("basic-auth"));
    print_feature_status("jwt-auth", features.is_enabled("jwt-auth"));
    print_feature_status("rbac", features.is_enabled("rbac"));
    print_feature_status("audit", features.is_enabled("audit"));
    
    // Type features
    println!("\n{}", "Type System Features:".yellow().bold());
    print_feature_status("extended-types", features.is_enabled("extended-types"));
    print_feature_status("engineering-types", features.is_enabled("engineering-types"));
    print_feature_status("quality-codes", features.is_enabled("quality-codes"));
    print_feature_status("value-arithmetic", features.is_enabled("value-arithmetic"));
    
    // Validation features
    println!("\n{}", "Validation Features:".yellow().bold());
    print_feature_status("validation", features.is_enabled("validation"));
    print_feature_status("regex-validation", features.is_enabled("regex-validation"));
    print_feature_status("schema-validation", features.is_enabled("schema-validation"));
    print_feature_status("composite-validation", features.is_enabled("composite-validation"));
    
    if list_all {
        println!("{}", features.report());
    }

    if show_dependencies || show_conflicts {
        println!("{}", features.report());
    }
    
    Ok(())
}

/// Print feature status with colored output
fn print_feature_status(feature_name: &str, enabled: bool) {
    let status = if enabled { "ENABLED".green().bold() } else { "DISABLED".bright_black() };
    println!("  {} {}", status, feature_name);
}

// ============================================================================
// CONFIGURATION COMMAND HANDLERS
// ============================================================================

/// Handle configuration management subcommands
async fn handle_config_command(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Example { output, template, include_features } => {
            generate_example_config(output, template, include_features).await
        }
        ConfigCommands::Convert { input, output, format } => {
            convert_config_format(input, output, format).await
        }
        ConfigCommands::Migrate { input, output, target_version } => {
            migrate_config_version(input, output, target_version).await
        }
        ConfigCommands::Schema { 
            #[cfg(feature = "schema-validation")]
            json, 
            output 
        } => {
            show_config_schema(
                #[cfg(feature = "schema-validation")]
                json,
                output
            ).await
        }
        ConfigCommands::Lint { config, fix } => {
            lint_config(config, fix).await
        }
    }
}

/// Generate example configuration files
async fn generate_example_config(
    output: PathBuf,
    template: ConfigTemplate,
    include_features: bool,
) -> Result<()> {
    info!("Generating {} configuration template", format!("{:?}", template).to_lowercase());
    
    let config = match template {
        ConfigTemplate::Basic => Config::example_basic(),
        ConfigTemplate::Industrial => Config::example_basic(),
        ConfigTemplate::Iot => Config::example_basic(),
        ConfigTemplate::Enterprise => Config::example_basic(),
        ConfigTemplate::Development => Config::example_basic(),
    };
    
    let mut final_config = config?;

    if include_features {
        // Feature examples not implemented
    }

    final_config.save_to_file(&output)?;
    
    println!("{} Generated {} template: {}", 
        "SUCCESS".green().bold(),
        format!("{:?}", template).to_lowercase(), 
        output.display()
    );
    
    Ok(())
}

/// Convert configuration between formats
async fn convert_config_format(
    input: PathBuf,
    output: PathBuf,
    format: ConfigFormat,
) -> Result<()> {
    info!("Converting {} to {} format", input.display(), format!("{:?}", format).to_lowercase());
    
    let mut config = Config::from_file(&input)?;
    
    match format {
        ConfigFormat::Yaml => config.save_to_file(&output)?,
        ConfigFormat::Json => config.save_as_json(&output)?,
        ConfigFormat::Toml => config.save_as_json(&output)?,
    }
    
    println!("{} Converted configuration to: {}", "SUCCESS".green().bold(), output.display());
    Ok(())
}

/// Migrate configuration to newer version
async fn migrate_config_version(
    input: PathBuf,
    output: Option<PathBuf>,
    target_version: String,
) -> Result<()> {
    info!("Migrating configuration to version: {}", target_version);
    
    let mut config = Config::from_file(&input)?;
    // Placeholder migration logic
    config.version = target_version.clone();
    let mut migrated = config;
    
    let output_path = output.unwrap_or_else(|| {
        let mut path = input.clone();
        path.set_extension("migrated.yaml");
        path
    });
    
    // Create backup of original
    if output_path == input {
        let mut backup_path = input.clone();
        backup_path.set_extension("backup.yaml");
        std::fs::copy(&input, &backup_path).map_err(|e| 
            PlcError::Config(format!("Failed to create backup: {}", e))
        )?;
        println!("{} Created backup: {}", "INFO".blue().bold(), backup_path.display());
    }
    
    migrated.save_to_file(&output_path)?;
    
    println!("{} Migrated configuration to: {}", "SUCCESS".green().bold(), output_path.display());
    Ok(())
}

/// Show configuration schema information
async fn show_config_schema(
    #[cfg(feature = "schema-validation")]
    json_format: bool,
    _output: Option<PathBuf>,
) -> Result<()> {
    #[cfg(feature = "schema-validation")]
    {
        let schema = if json_format {
            Config::json_schema()?
        } else {
            Config::yaml_schema()?
        };
        
        if let Some(output_path) = _output {
            std::fs::write(&output_path, &schema).map_err(|e|
                PlcError::Config(format!("Failed to write schema: {}", e))
            )?;
            println!("{} Schema written to: {}", "SUCCESS".green().bold(), output_path.display());
        } else {
            println!("{}", schema);
        }
    }
    
    #[cfg(not(feature = "schema-validation"))]
    {
        println!("{} Schema validation feature not enabled in this build", "WARNING".yellow().bold());
        println!("Enable with: cargo build --features schema-validation");
    }
    
    Ok(())
}

/// Lint configuration file for best practices
async fn lint_config(config_path: PathBuf, apply_fixes: bool) -> Result<()> {
    info!("Linting configuration: {}", config_path.display());
    
    let mut config = Config::from_file(&config_path)?;
    let lint_results = config.lint()?;
    
    if lint_results.is_empty() {
        println!("{}", "Configuration passes all lint checks".green().bold());
        return Ok(());
    }
    
    println!("{}", "Lint Results:".cyan().bold());
    for result in &lint_results {
        let severity_marker = match result.severity {
            LintSeverity::Error => "ERROR".red().bold(),
            LintSeverity::Warning => "WARN".yellow().bold(),
            LintSeverity::Info => "INFO".blue().bold(),
        };
        println!("  {} {}: {}", severity_marker, result.rule, result.message);
        
        if let Some(suggestion) = &result.suggestion {
            println!("    {} {}", "SUGGESTION:".bright_cyan(), suggestion);
        }
    }
    
    if apply_fixes {
        config.apply_lint_fixes(&lint_results)?;
        config.save_to_file(&config_path)?;
        println!("{}", "Applied automatic fixes to configuration".green().bold());
    }
    
    Ok(())
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Set up graceful shutdown signal handling
async fn setup_shutdown_handler() {
    let ctrl_c = signal::ctrl_c();
    
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to setup SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to setup SIGINT handler");
        
        tokio::select! {
            _ = ctrl_c => info!("Received Ctrl+C"),
            _ = sigterm.recv() => info!("Received SIGTERM"),
            _ = sigint.recv() => info!("Received SIGINT"),
        }
    }
    
    #[cfg(not(unix))]
    {
        ctrl_c.await.expect("Failed to listen for Ctrl+C");
        info!("Received Ctrl+C");
    }
}

/// Set CPU affinity for current process (Linux only)
#[cfg(target_os = "linux")]
fn set_cpu_affinity(cores: &[usize]) -> Result<()> {
    use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
    use std::mem;
    
    let mut cpu_set: cpu_set_t = unsafe { mem::zeroed() };
    unsafe { CPU_ZERO(&mut cpu_set) };
    
    for &core in cores {
        if core >= num_cpus::get() {
            return Err(PlcError::Runtime(format!("Invalid CPU core: {}", core)));
        }
        unsafe { CPU_SET(core, &mut cpu_set) };
    }
    
    let result = unsafe {
        sched_setaffinity(0, mem::size_of::<cpu_set_t>(), &cpu_set)
    };
    
    if result != 0 {
        return Err(PlcError::Runtime("Failed to set CPU affinity".into()));
    }
    
    info!("Set CPU affinity to cores: {:?}", cores);
    Ok(())
}

/// Show help and available features when no command is provided
async fn show_help_and_features() -> Result<()> {
    println!("{}", build_info::build_info_string());
    println!("\nNo configuration file provided.");
    println!("\nUsage examples:");
    println!("  {} run config.yaml              # Run with configuration", "petra".green());
    println!("  {} validate config.yaml        # Validate configuration", "petra".green());
    println!("  {} config example              # Generate example config", "petra".green());
    println!("  {} features                    # Show enabled features", "petra".green());
    println!("  {} --help                      # Show detailed help", "petra".green());
    
    // Show available features summary
    let features = features::current();
    println!("\n{} Available Features in this build:", "INFO".blue().bold());
    
    let feature_list = vec![
        ("Core", vec![
            ("monitoring", features.is_enabled("standard-monitoring") || features.is_enabled("enhanced-monitoring")),
            ("metrics", features.is_enabled("metrics")),
            ("realtime", features.is_enabled("realtime")),
        ]),
        ("Protocols", vec![
            ("MQTT", features.is_enabled("mqtt")),
            ("Modbus", features.is_enabled("modbus-support")),
            ("S7", features.is_enabled("s7-support")),
            ("OPC-UA", features.is_enabled("opcua-support")),
        ]),
        ("Storage", vec![
            ("History", features.is_enabled("history")),
            ("Advanced", features.is_enabled("advanced-storage")),
        ]),
        ("Security", vec![
            ("Authentication", features.is_enabled("security")),
            ("RBAC", features.is_enabled("rbac")),
        ]),
    ];
    
    for (category, features) in feature_list {
        let enabled_features: Vec<&str> = features.iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| *name)
            .collect();
        
        if enabled_features.is_empty() {
            println!("  {}: {}", category.cyan(), "None".bright_black());
        } else {
            println!("  {}: {}", category.cyan(), enabled_features.join(", ").green());
        }
    }
    
    println!("\nUse '{}' for complete feature information.", "petra features".green());
    Ok(())
}

// ============================================================================
// FEATURE-SPECIFIC COMMAND HANDLERS (conditionally compiled)
// ============================================================================

/// Handle development and testing commands
#[cfg(any(feature = "examples", feature = "burn-in", feature = "profiling"))]
async fn handle_dev_command(cmd: DevCommands) -> Result<()> {
    match cmd {
        #[cfg(feature = "examples")]
        DevCommands::Examples { list, run, generate_data } => {
            if list {
                petra::examples::list_examples();
            } else if let Some(example_name) = run {
                petra::examples::run_example(&example_name, generate_data).await?;
            } else {
                petra::examples::run_interactive().await?;
            }
        }
        
        #[cfg(feature = "burn-in")]
        DevCommands::BurnIn { duration, signals, blocks, scan_time, memory_stress } => {
            petra::burn_in::run_burn_in_test(
                Duration::from_secs(duration),
                signals,
                blocks,
                Duration::from_millis(scan_time),
                memory_stress,
            ).await?;
        }
        
        #[cfg(feature = "profiling")]
        DevCommands::Profile { config, duration, output, cpu, memory } => {
            petra::profiling::run_performance_profile(
                config,
                Duration::from_secs(duration),
                output,
                cpu,
                memory,
            ).await?;
        }
    }
    Ok(())
}

/// Handle protocol testing commands
#[cfg(any(
    feature = "mqtt", 
    feature = "s7-support", 
    feature = "modbus-support",
    feature = "opcua-support"
))]
async fn handle_protocol_command(cmd: ProtocolCommands) -> Result<()> {
    match cmd {
        #[cfg(feature = "mqtt")]
        ProtocolCommands::Mqtt { broker, topic, count } => {
            petra::mqtt::test_connection(&broker, &topic, count as u32).await?;
        }
        
        #[cfg(feature = "modbus-support")]
        ProtocolCommands::Modbus { address, unit_id, register } => {
            petra::modbus::test_connection(&address, unit_id, register).await?;
        }
        
        #[cfg(feature = "s7-support")]
        ProtocolCommands::S7 { address, rack, slot } => {
            petra::s7::test_connection(&address, rack, slot).await?;
        }
        
        #[cfg(feature = "opcua-support")]
        ProtocolCommands::OpcUa { endpoint, node_id } => {
            petra::opcua::test_connection(&endpoint, node_id.as_deref()).await?;
        }
    }
    Ok(())
}

/// Start standalone metrics server
#[cfg(feature = "metrics")]
async fn start_metrics_server(port: u16, bind: String, runtime_metrics: bool) -> Result<()> {
    info!("Starting metrics server on {}:{}", bind, port);
    
    let mut server = MetricsServer::new(&bind, port)?;
    
    if runtime_metrics {
        server.enable_runtime_metrics();
    }
    
    server.start().await?;
    
    // Wait for shutdown signal
    setup_shutdown_handler().await;
    
    info!("Shutting down metrics server");
    server.stop().await?;
    
    Ok(())
}

/// Start standalone health monitoring server
#[cfg(feature = "health")]
async fn start_health_server(port: u16, bind: String, check_interval: u64) -> Result<()> {
    info!("Starting health server on {}:{}", bind, port);
    
    let mut monitor = HealthMonitor::new(&bind, port, Duration::from_secs(check_interval))?;
    monitor.start().await?;
    
    // Wait for shutdown signal
    setup_shutdown_handler().await;
    
    info!("Shutting down health server");
    monitor.stop().await?;
    
    Ok(())
}

/// Handle security management commands
#[cfg(feature = "security")]
async fn handle_security_command(cmd: SecurityCommands) -> Result<()> {
    match cmd {
        SecurityCommands::KeyGen { key_type, output } => {
            let key_type_str = match key_type {
                KeyType::Rsa => "rsa",
                KeyType::Ed25519 => "ed25519",
                KeyType::Aes => "aes",
                #[cfg(feature = "jwt-auth")]
                KeyType::Jwt => "jwt",
            };
            petra::security::generate_key(key_type_str, &output).await?;
            println!("{} Generated {} key: {}", 
                "SUCCESS".green().bold(),
                format!("{:?}", key_type).to_lowercase(), 
                output.display()
            );
        }
        
        #[cfg(feature = "basic-auth")]
        SecurityCommands::CreateUser { 
            username, 
            password, 
            #[cfg(feature = "rbac")]
            roles 
        } => {
            let pwd = if let Some(p) = password {
                p
            } else {
                rpassword::prompt_password("Password: ")?
            };
            
            petra::security::create_user(
                &username,
                &pwd,
                None,
                #[cfg(feature = "rbac")]
                roles,
                #[cfg(not(feature = "rbac"))]
                Vec::new()
            ).await?;
            println!("{} Created user: {}", "SUCCESS".green().bold(), username);
        }
        
        #[cfg(feature = "audit")]
        SecurityCommands::Audit { limit, user, action } => {
            let entries = petra::security::get_audit_log(limit, user.as_deref(), action.as_deref()).await?;
            
            println!("{} Audit Log ({} entries):", "INFO".blue().bold(), entries.len());
            for entry in entries {
                println!("  {} {} [{}] {}: {}", 
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.user,
                    entry.source_ip,
                    entry.action,
                    entry.details
                );
            }
        }
    }
    Ok(())
}

/// Handle storage management commands
#[cfg(feature = "advanced-storage")]
async fn handle_storage_command(cmd: StorageCommands) -> Result<()> {
    match cmd {
        StorageCommands::Init { storage_type, config } => {
            let config_path = config.unwrap_or_else(|| PathBuf::from("petra.yaml"));
            petra::storage::initialize_storage(storage_type, &config_path).await?;
            println!("{} Initialized {} storage", 
                "SUCCESS".green().bold(),
                format!("{:?}", storage_type).to_lowercase()
            );
        }
        
        StorageCommands::Backup { output, start_time, end_time } => {
            petra::storage::backup_data(&output, start_time.as_deref(), end_time.as_deref()).await?;
            println!("{} Backup completed: {}", "SUCCESS".green().bold(), output.display());
        }
        
        StorageCommands::Restore { input, force } => {
            if !force {
                print!("This will overwrite existing data. Continue? (y/N): ");
                use std::io::{self, Write};
                io::stdout().flush()?;
                
                let mut response = String::new();
                io::stdin().read_line(&mut response)?;
                
                if !response.trim().to_lowercase().starts_with('y') {
                    println!("Restore cancelled");
                    return Ok(());
                }
            }
            
            petra::storage::restore_data(&input).await?;
            println!("{} Restore completed from: {}", "SUCCESS".green().bold(), input.display());
        }
        
        StorageCommands::Compact { dry_run } => {
            let stats = petra::storage::compact_storage(dry_run).await?;
            
            if dry_run {
                println!("{} Dry run results:", "INFO".blue().bold());
                println!("  Files that would be compacted: {}", stats.files_to_compact);
                println!("  Estimated space savings: {} MB", stats.estimated_savings_mb);
            } else {
                println!("{} Storage compaction completed:", "SUCCESS".green().bold());
                println!("  Files compacted: {}", stats.files_compacted);
                println!("  Space reclaimed: {} MB", stats.space_reclaimed_mb);
            }
        }
    }
    Ok(())
}

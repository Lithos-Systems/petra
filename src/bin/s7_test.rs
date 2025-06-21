// src/bin/s7_test.rs
use petra::s7::{S7Config, S7Mapping, S7Area, S7DataType, Direction};
use petra::{SignalBus, S7Connector, Value};
use tracing::{info, error};
use tracing_subscriber;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "s7_test")]
#[command(about = "Test S7 PLC connectivity", long_about = None)]
struct Cli {
    /// PLC IP address
    #[arg(short, long)]
    ip: String,
    
    /// Rack number
    #[arg(short, long, default_value = "0")]
    rack: u16,
    
    /// Slot number
    #[arg(short, long, default_value = "2")]
    slot: u16,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test connection to PLC
    Connect,
    
    /// Read a value from PLC
    Read {
        /// Memory area (DB, I, Q, M)
        #[arg(short, long)]
        area: String,
        
        /// DB number (for DB area)
        #[arg(short, long, default_value = "0")]
        db: u16,
        
        /// Address
        #[arg(short = 'A', long)]
        address: u32,
        
        /// Data type (bool, byte, word, int, dint, real)
        #[arg(short, long)]
        data_type: String,
        
        /// Bit offset (for bool)
        #[arg(short, long, default_value = "0")]
        bit: u8,
    },
    
    /// Write a value to PLC
    Write {
        /// Memory area (DB, Q, M)
        #[arg(short, long)]
        area: String,
        
        /// DB number (for DB area)
        #[arg(short, long, default_value = "0")]
        db: u16,
        
        /// Address
        #[arg(short = 'A', long)]
        address: u32,
        
        /// Data type (bool, byte, word, int, dint, real)
        #[arg(short, long)]
        data_type: String,
        
        /// Bit offset (for bool)
        #[arg(short, long, default_value = "0")]
        bit: u8,
        
        /// Value to write
        #[arg(short, long)]
        value: String,
    },
    
    /// Monitor values continuously
    Monitor {
        /// Config file with mappings
        #[arg(short, long)]
        config: String,
    },
    
    /// Get PLC info
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("s7_test=debug,petra=debug")
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Connect => test_connection(&cli).await?,
        Commands::Read { area, db, address, data_type, bit } => {
            test_read(&cli, area.clone(), *db, *address, data_type.clone(), *bit).await?
        },
        Commands::Write { area, db, address, data_type, bit, value } => {
            test_write(&cli, area.clone(), *db, *address, data_type.clone(), *bit, value.clone()).await?
        },
        Commands::Monitor { config } => {
            monitor_values(&cli, config.clone()).await?
        },
        Commands::Info => {
            test_info(&cli).await?
        }
    }
    
    Ok(())
}

async fn test_connection(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing connection to {}:{}:{}", cli.ip, cli.rack, cli.slot);
    
    let config = S7Config {
        ip: cli.ip.clone(),
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".to_string(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![],
    };
    
    let bus = SignalBus::new();
    let connector = S7Connector::new(config, bus)?;
    
    match connector.connect().await {
        Ok(_) => {
            info!("✓ Successfully connected to PLC!");
            info!("  IP: {}", cli.ip);
            info!("  Rack: {}", cli.rack);
            info!("  Slot: {}", cli.slot);
            info!("  Ready for data exchange");
        },
        Err(e) => {
            error!("✗ Connection failed: {}", e);
            error!("  Check IP address, rack/slot numbers, and PLC accessibility");
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn test_info(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    info!("Getting PLC info from {}", cli.ip);
    
    // Create a snap7 client directly

// src/bin/s7_test.rs
use petra::s7::{S7Config, S7Mapping, S7Area, S7DataType, Direction};
use petra::{SignalBus, S7Connector, Value};
use tracing::{info, error};
use tracing_subscriber;
use clap::{Parser, Subcommand};
use rust_snap7::{S7Client, InternalParam, InternalParamValue};

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
    #[arg(short, long, default_value = "1")]
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
        #[arg(short = 't', long)]
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
        #[arg(short = 't', long)]
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
    
    // Create a simple client for testing
    let client = S7Client::create();
    
    // Set only the remote port parameter
    client.set_param(InternalParam::RemotePort, InternalParamValue::U16(102))?;
    
    match client.connect_to(&cli.ip, cli.rack as i32, cli.slot as i32) {
        Ok(_) => {
            info!("✓ Successfully connected to PLC!");
            info!("  IP: {}", cli.ip);
            info!("  Rack: {}", cli.rack);
            info!("  Slot: {}", cli.slot);
            info!("  Ready for data exchange");
            
            // Test disconnection
            client.disconnect()?;
            info!("✓ Disconnected successfully");
        },
        Err(e) => {
            error!("✗ Connection failed: {:?}", e);
            error!("  Check IP address, rack/slot numbers, and PLC accessibility");
            return Err(e.into());
        }
    }
    
    Ok(())
}

async fn test_info(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    info!("Getting PLC info from {}", cli.ip);
    
    // Create a client directly for info commands
    let client = S7Client::create();
    client.set_param(InternalParam::RemotePort, InternalParamValue::U16(102))?;
    client.connect_to(&cli.ip, cli.rack as i32, cli.slot as i32)?;
    
    // Get PLC status
    let mut status: i32 = 0;
    if let Ok(_) = client.get_plc_status(&mut status) {
        let status_text = match status {
            0x00 => "Unknown",
            0x04 => "Stop",
            0x08 => "Run",
            _ => "Other",
        };
        info!("PLC Status: {} (0x{:02X})", status_text, status);
    }
    
    client.disconnect()?;
    Ok(())
}

async fn test_read(
    cli: &Cli,
    area: String,
    db: u16,
    address: u32,
    data_type: String,
    bit: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let area_enum = parse_area(&area)?;
    let type_enum = parse_data_type(&data_type)?;
    
    info!("Reading {} from {}{}:{} bit {}", 
        data_type, area, 
        if matches!(area_enum, S7Area::DB) { format!("{}", db) } else { "".to_string() },
        address, bit
    );
    
    let mapping = S7Mapping {
        signal: "test_signal".to_string(),
        area: area_enum,
        db_number: db,
        address,
        data_type: type_enum,
        bit,
        direction: Direction::Read,
    };

    let config = S7Config {
        ip: cli.ip.clone(),
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".to_string(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![mapping.clone()],
    };
    
    let bus = SignalBus::new();
    let connector = S7Connector::new(config, bus.clone())?;
    connector.connect().await?;

    // Do one read cycle
    connector.read_mapping(&mapping).await?;
    
    let value = bus.get("test_signal").ok_or("Signal not found")?;
    info!("✓ Read value: {}", value);
    
    Ok(())
}

async fn test_write(
    cli: &Cli,
    area: String,
    db: u16,
    address: u32,
    data_type: String,
    bit: u8,
    value_str: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let area_enum = parse_area(&area)?;
    let type_enum = parse_data_type(&data_type)?;
    
    // Parse value based on type
    let value = match type_enum {
        S7DataType::Bool => Value::Bool(value_str.parse::<bool>()?),
        S7DataType::Real => Value::Float(value_str.parse::<f64>()?),
        _ => Value::Int(value_str.parse::<i64>()?),
    };
    
    info!("Writing {} = {} to {}{}:{} bit {}", 
        data_type, value, area,
        if matches!(area_enum, S7Area::DB) { format!("{}", db) } else { "".to_string() },
        address, bit
    );
    
    let mapping = S7Mapping {
        signal: "test_signal".to_string(),
        area: area_enum,
        db_number: db,
        address,
        data_type: type_enum,
        bit,
        direction: Direction::Write,
    };
    
    let config = S7Config {
        ip: cli.ip.clone(),
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".to_string(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![mapping.clone()],
    };
    
    let bus = SignalBus::new();
    bus.set("test_signal", value.clone())?;
    
    let connector = S7Connector::new(config, bus)?;
    connector.connect().await?;

    // Do one write cycle
    connector.write_mapping(&mapping).await?;
    
    info!("✓ Successfully wrote value");
    
    Ok(())
}

async fn monitor_values(
    cli: &Cli,
    config_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Monitoring values from config: {}", config_file);
    
    // Load and parse config
    let config_str = std::fs::read_to_string(&config_file)?;
    let config: petra::Config = serde_yaml::from_str(&config_str)?;
    
    let mut s7_config = match config.protocols.and_then(|p| p.s7) {
        Some(cfg) => cfg,
        None => {
            error!("No S7 configuration found in file");
            return Ok(());
        }
    };
    s7_config.ip = cli.ip.clone();
    s7_config.rack = cli.rack;
    s7_config.slot = cli.slot;
    
    let bus = SignalBus::new();
    
    // Initialize signals
    for mapping in &s7_config.mappings {
        let initial = match mapping.data_type {
            S7DataType::Bool => Value::Bool(false),
            S7DataType::Real => Value::Float(0.0),
            _ => Value::Int(0),
        };
        bus.set(&mapping.signal, initial)?;
    }
    
    let connector = S7Connector::new(s7_config.clone(), bus.clone())?;
    connector.connect().await?;
    
    info!("Starting monitor mode (Ctrl+C to stop)");
    info!("{:<20} {:<10} {:<15}", "Signal", "Type", "Value");
    info!("{:-<45}", "");
    
    // Run monitoring loop
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(s7_config.poll_interval_ms));
    
    loop {
        interval.tick().await;
        
        // Read all mappings
        for mapping in &s7_config.mappings {
            if matches!(mapping.direction, Direction::Read | Direction::ReadWrite) {
                if let Err(e) = connector.read_mapping(mapping).await {
                    error!("Read error for {}: {}", mapping.signal, e);
                }
            }
        }
        
        // Display current values
        print!("\x1B[{}A", s7_config.mappings.len()); // Move cursor up
        
        for mapping in &s7_config.mappings {
            if let Some(value) = bus.get(&mapping.signal) {
                println!("{:<20} {:<10} {:<15}",
                    mapping.signal,
                    format!("{:?}", mapping.data_type),
                    value
                );
            }
        }
    }
}

fn parse_area(area: &str) -> Result<S7Area, String> {
    match area.to_uppercase().as_str() {
        "DB" => Ok(S7Area::DB),
        "I" => Ok(S7Area::I),
        "Q" => Ok(S7Area::Q),
        "M" => Ok(S7Area::M),
        "C" => Ok(S7Area::C),
        "T" => Ok(S7Area::T),
        _ => Err(format!("Invalid area: {}", area)),
    }
}

fn parse_data_type(dtype: &str) -> Result<S7DataType, String> {
   match dtype.to_lowercase().as_str() {
       "bool" => Ok(S7DataType::Bool),
       "byte" => Ok(S7DataType::Byte),
       "word" => Ok(S7DataType::Word),
       "int" => Ok(S7DataType::Int),
       "dword" => Ok(S7DataType::DWord),
       "dint" => Ok(S7DataType::DInt),
       "real" => Ok(S7DataType::Real),
       _ => Err(format!("Invalid data type: {}", dtype)),
   }
}

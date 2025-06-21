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
    Connect,

    Read {
        #[arg(short, long)]
        area: String,
        #[arg(short, long, default_value = "0")]
        db: u16,
        #[arg(short = 'A', long)]
        address: u32,
        #[arg(short, long)]
        data_type: String,
        #[arg(short, long, default_value = "0")]
        bit: u8,
    },

    Write {
        #[arg(short, long)]
        area: String,
        #[arg(short, long, default_value = "0")]
        db: u16,
        #[arg(short = 'A', long)]
        address: u32,
        #[arg(short, long)]
        data_type: String,
        #[arg(short, long, default_value = "0")]
        bit: u8,
        #[arg(short, long)]
        value: String,
    },

    Monitor {
        #[arg(short, long)]
        config: String,
    },
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
            test_read(cli, area, *db, *address, data_type, *bit).await?
        },
        Commands::Write { area, db, address, data_type, bit, value } => {
            test_write(cli, area, *db, *address, data_type, *bit, value).await?
        },
        Commands::Monitor { config } => monitor_values(cli, config).await?,
    }

    Ok(())
}

async fn test_connection(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing connection to {}:{}:{}", cli.ip, cli.rack, cli.slot);

    let config = S7Config {
        ip: cli.ip.clone(),
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".into(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![],
    };

    let bus = SignalBus::new();
    let connector = S7Connector::new(config, bus)?;

    connector.connect().await.map_err(|e| {
        error!("✗ Connection failed: {}", e);
        e
    })?;

    info!("✓ Successfully connected to PLC!");

    Ok(())
}

async fn test_read(
    cli: Cli,
    area: &str,
    db: u16,
    address: u32,
    data_type: &str,
    bit: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let mapping = S7Mapping {
        signal: "test_signal".into(),
        area: parse_area(area)?,
        db_number: db,
        address,
        data_type: parse_data_type(data_type)?,
        bit,
        direction: Direction::Read,
    };

    let config = S7Config {
        ip: cli.ip,
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".into(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![mapping.clone()],
    };

    let bus = SignalBus::new();
    let connector = S7Connector::new(config, bus.clone())?;
    connector.connect().await?;

    connector.read_mapping(&mapping).await?;

    let value = bus.get("test_signal")?;
    info!("✓ Read value: {}", value);

    Ok(())
}

async fn test_write(
    cli: Cli,
    area: &str,
    db: u16,
    address: u32,
    data_type: &str,
    bit: u8,
    value_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mapping = S7Mapping {
        signal: "test_signal".into(),
        area: parse_area(area)?,
        db_number: db,
        address,
        data_type: parse_data_type(data_type)?,
        bit,
        direction: Direction::Write,
    };

    let value = match mapping.data_type {
        S7DataType::Bool => Value::Bool(value_str.parse::<bool>()?),
        S7DataType::Real => Value::Float(value_str.parse::<f64>()?),
        _ => Value::Int(value_str.parse::<i32>()?),
    };

    let config = S7Config {
        ip: cli.ip,
        rack: cli.rack,
        slot: cli.slot,
        connection_type: "PG".into(),
        poll_interval_ms: 1000,
        timeout_ms: 5000,
        mappings: vec![mapping.clone()],
    };

    let bus = SignalBus::new();
    bus.set("test_signal", value)?;

    let connector = S7Connector::new(config, bus)?;
    connector.connect().await?;

    connector.write_mapping(&mapping).await?;

    info!("✓ Successfully wrote value");

    Ok(())
}

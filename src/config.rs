// Updates to src/config.rs
use crate::error::*;
use crate::mqtt::MqttConfig;
use crate::s7::S7Config;  // Add this import
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub signals: Vec<SignalConfig>,
    pub blocks: Vec<BlockConfig>,
    #[serde(default = "default_scan_time")]
    pub scan_time_ms: u64,
    #[serde(default)]
    pub mqtt: MqttConfig,
    #[serde(default)]
    pub s7: Option<S7Config>,  // Add S7 configuration
}

// ... rest of the config.rs file remains the same ...

// Updates to src/main.rs
use petra::{Config, Engine, MqttHandler, S7Connector, Result};
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("petra=info".parse().unwrap()),
        )
        .init();

    info!("Petra v{} starting", petra::VERSION);

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            error!("Usage: {} <config.yaml>", std::env::args().next().unwrap());
            std::process::exit(1);
        });

    let config = Config::from_file(&config_path)?;
    info!("Loaded {} signals, {} blocks", config.signals.len(), config.blocks.len());

    let mut engine = Engine::new(config.clone())?;
    let bus = engine.bus().clone();

    // Create channel for signal changes
    let (signal_tx, signal_rx) = mpsc::channel(100);
    
    engine.set_signal_change_channel(signal_tx.clone());
    
    // Start MQTT if configured
    let mqtt_handle = if !config.mqtt.broker_host.is_empty() {
        let mut mqtt = MqttHandler::new(bus.clone(), config.mqtt)?;
        mqtt.set_signal_change_channel(signal_rx);
        mqtt.start().await?;
        
        Some(tokio::spawn(async move {
            if let Err(e) = mqtt.run().await {
                error!("MQTT error: {}", e);
            }
        }))
    } else {
        None
    };
    
    // Start S7 connector if configured
    let s7_handle = if let Some(s7_config) = config.s7 {
        info!("Starting S7 connector for {}", s7_config.ip);
        let s7 = S7Connector::new(s7_config, bus.clone())?;
        s7.connect().await?;
        
        Some(tokio::spawn(async move {
            if let Err(e) = s7.run().await {
                error!("S7 error: {}", e);
            }
        }))
    } else {
        None
    };

    let ctrl_c = signal::ctrl_c();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received shutdown signal");
            engine.stop();
        }
        res = engine.run() => {
            if let Err(e) = res {
                error!("Engine error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Clean shutdown
    if let Some(handle) = mqtt_handle {
        handle.abort();
    }
    if let Some(handle) = s7_handle {
        handle.abort();
    }

    info!("Engine stopped normally");

    let stats = engine.stats();
    info!(
        "Final stats: {} scans, {} errors, uptime: {}s",
        stats.scan_count, stats.error_count, stats.uptime_secs
    );

    Ok(())
}

// Updates to src/lib.rs
//! Petra v2.0 - Production-ready PLC with MQTT and S7 integration

pub mod error;
pub mod value;
pub mod signal;
pub mod block;
pub mod config;
pub mod engine;
pub mod mqtt;
pub mod s7;  // Add S7 module

pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use config::Config;
pub use engine::{Engine, EngineStats};
pub use mqtt::{MqttHandler, MqttMessage};
pub use s7::{S7Connector, S7Config};  // Export S7 types

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

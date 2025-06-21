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
    
    engine.set_signal_change_channel(signal_tx);
    
    // Setup MQTT if configured
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
        info!("MQTT not configured, skipping");
        None
    };

    // Setup S7 if configured
    let s7_handle = if let Some(s7_config) = config.s7 {
        info!("Starting S7 connector");
        let s7 = S7Connector::new(s7_config, bus.clone())?;
        
        // Connect to PLC
        match s7.connect().await {
            Ok(_) => {
                info!("S7 PLC connected successfully");
                Some(tokio::spawn(async move {
                    if let Err(e) = s7.run().await {
                        error!("S7 error: {}", e);
                    }
                }))
            }
            Err(e) => {
                error!("Failed to connect to S7 PLC: {}", e);
                None
            }
        }
    } else {
        info!("S7 not configured, skipping");
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

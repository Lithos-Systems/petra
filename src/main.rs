use petra::{Config, Engine, MqttHandler, Result};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber;
use std::sync::Arc;

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

    let mut mqtt = MqttHandler::new(bus, config.mqtt)?;
    mqtt.start().await?;

    tokio::spawn(async move {
        if let Err(e) = mqtt.run().await {
            error!("MQTT error: {}", e);
        }
    });

    let engine_arc = Arc::new(&engine);
    let engine_handle = engine_arc.clone();

    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("Received shutdown signal");
                engine_handle.stop();
            }
            Err(e) => {
                error!("Failed to listen for shutdown signal: {}", e);
            }
        }
    });

    match engine.run().await {
        Ok(()) => info!("Engine stopped normally"),
        Err(e) => {
            error!("Engine error: {}", e);
            std::process::exit(1);
        }
    }

    let stats = engine.stats();
    info!(
        "Final stats: {} scans, {} errors, uptime: {}s",
        stats.scan_count, stats.error_count, stats.uptime_secs
    );

    Ok(())
}

use petra::{Config, Engine, MqttHandler, Result};
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
    
    let mut mqtt = MqttHandler::new(bus, config.mqtt)?;
    mqtt.set_signal_change_channel(signal_rx);
    mqtt.start().await?;

    let ctrl_c = signal::ctrl_c();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received shutdown signal");
            engine.stop();
        }
        res = async {
            // Run MQTT and Engine concurrently
            let mqtt_task = tokio::spawn(async move {
                if let Err(e) = mqtt.run().await {
                    error!("MQTT error: {}", e);
                }
            });
            
            let engine_result = engine.run().await;
            
            // Clean shutdown
            mqtt_task.abort();
            engine_result
        } => {
            if let Err(e) = res {
                error!("Engine error: {}", e);
                std::process::exit(1);
            }
        }
    }

    info!("Engine stopped normally");

    let stats = engine.stats();
    info!(
        "Final stats: {} scans, {} errors, uptime: {}s",
        stats.scan_count, stats.error_count, stats.uptime_secs
    );

    Ok(())
}

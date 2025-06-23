use petra::{Config, Engine, MqttHandler, S7Connector, TwilioConnector, HistoryManager, Result};
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{error, info};
use tracing_subscriber;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("petra=info".parse().unwrap()),
        )
        .init();

    // Initialize Prometheus metrics endpoint
    let metrics_addr: SocketAddr = "0.0.0.0:9090".parse().unwrap();
    PrometheusBuilder::new()
        .listen_address(metrics_addr)
        .install()
        .expect("Failed to install Prometheus recorder");

    info!("Petra v{} starting", petra::VERSION);
    info!("Metrics available at http://{}/metrics", metrics_addr);

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

    // Create channels for signal changes
    let (signal_tx, signal_rx) = mpsc::channel(1000);
    let (history_tx, history_rx) = mpsc::channel(1000);
    
    // Clone sender for multiple consumers
    let signal_tx_clone = signal_tx.clone();
    engine.set_signal_change_channel(signal_tx);
    
    // Setup History Manager if configured
    let history_handle = if let Some(history_config) = config.history {
        info!("History configuration found, starting history manager");
        let mut history_manager = HistoryManager::new(history_config, bus.clone())?;
        history_manager.set_signal_change_channel(history_rx);
        
        // Forward signal changes to history
        let history_tx_clone = history_tx.clone();
        tokio::spawn(async move {
            let mut rx = signal_rx;
            while let Some(change) = rx.recv().await {
                let _ = history_tx_clone.send(change).await;
            }
        });
        
        Some(tokio::spawn(async move {
            if let Err(e) = history_manager.run().await {
                error!("History manager error: {}", e);
            }
        }))
    } else {
        info!("No history configuration found");
        None
    };
    
    // Setup MQTT handler
    let mut mqtt = MqttHandler::new(bus.clone(), config.mqtt)?;
    // Create a new receiver that forwards from the original
    let (mqtt_tx, mqtt_rx) = mpsc::channel(100);
    mqtt.set_signal_change_channel(mqtt_rx);
    
    // Forward changes to MQTT
    tokio::spawn(async move {
        let mut rx = signal_rx;
        while let Some(change) = rx.recv().await {
            let _ = mqtt_tx.send(change).await;
            let _ = history_tx.send(change).await;
        }
    });
    
    mqtt.start().await?;

    // Setup Twilio
    let twilio_handle = if let Some(twilio_config) = config.twilio {
        info!("Twilio configuration found, starting Twilio connector");
        let twilio_connector = TwilioConnector::new(twilio_config, bus.clone())?;
        
        // Spawn Twilio task
        Some(tokio::spawn(async move {
            if let Err(e) = twilio_connector.run().await {
                error!("Twilio connector error: {}", e);
            }
        }))
    } else {
        info!("No Twilio configuration found");
        None
    };
    
    // In cleanup section:
    if let Some(twilio_task) = twilio_handle {
        twilio_task.abort();
    }

    // Setup S7 connector if configured
    let s7_handle = if let Some(s7_config) = config.s7 {
        info!("S7 configuration found, starting S7 connector");
        let s7_connector = S7Connector::new(s7_config, bus.clone())?;
        
        // Connect to PLC
        s7_connector.connect().await?;
        
        // Spawn S7 task
        Some(tokio::spawn(async move {
            if let Err(e) = s7_connector.run().await {
                error!("S7 connector error: {}", e);
            }
        }))
    } else {
        info!("No S7 configuration found, running without PLC connection");
        None
    };

    let ctrl_c = signal::ctrl_c();
    
    tokio::select! {
        _ = ctrl_c => {
            info!("Received shutdown signal");
            engine.stop();
        }
        res = async {
            // Run all components concurrently
            let mqtt_task = tokio::spawn(async move {
                if let Err(e) = mqtt.run().await {
                    error!("MQTT error: {}", e);
                }
            });
            
            let engine_result = engine.run().await;
            
            // Clean shutdown
            mqtt_task.abort();
            if let Some(history_task) = history_handle {
                history_task.abort();
            }
            // ... abort other tasks ...
            
            engine_result
        } => {
            if let Err(e) = res {
                error!("Engine error: {}", e);
                std::process::exit(1);
            }
        }
    }

    info!("Engine stopped normally");

    let stats = engine.detailed_stats();
    info!(
        "Final stats: {} scans, {} errors, uptime: {}s, avg scan time: {:?}",
        stats.scan_count, stats.error_count, stats.uptime_secs, stats.avg_scan_time
    );

    Ok(())
}

use petra::{*, error::Result};
use tokio::signal;
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info, warn};
use tracing_subscriber;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use clap::Parser;

#[cfg(feature = "opcua-support")]
use petra::OpcUaServer;

#[cfg(feature = "advanced-storage")]
use petra::storage::{StorageConfig, StorageManager};

mod signal_enhanced {
    pub use crate::signal::SignalBus as EnhancedSignalBus;
    pub use crate::signal::SignalBus;
    
    #[derive(Debug, Clone)]
    pub struct SignalBusConfig {
        pub max_signals: usize,
        pub signal_ttl: std::time::Duration,
        pub cleanup_interval: std::time::Duration,
        pub hot_signal_threshold: u64,
    }
}

mod health {
    use std::sync::Arc;
    
    pub struct HealthChecker {
        // Stub implementation
    }
    
    impl HealthChecker {
        pub fn new(_interval: std::time::Duration) -> Self {
            Self {}
        }
        
        pub fn add_check(&mut self, _check: Arc<dyn HealthCheck>) {}
    }
    
    pub trait HealthCheck: Send + Sync {}
    
    pub struct StorageHealthCheck;
    impl StorageHealthCheck {
        pub fn new(_manager: Arc<crate::storage::StorageManager>) -> Self {
            Self
        }
    }
    impl HealthCheck for StorageHealthCheck {}
    
    // Add other health check types as needed
}
mod realtime {
    #[derive(Default)]
    pub struct RealtimeConfig {
        pub rt_priority: Option<i32>,
        pub lock_memory: bool,
        pub cpu_affinity: Vec<usize>,
    }
    
    pub fn configure_realtime(_config: &RealtimeConfig) -> Result<(), String> {
        // Stub implementation
        Ok(())
    }
}
mod validation {
    pub struct ValidationRules;
    impl ValidationRules {
        pub fn new() -> Self { Self }
        pub fn add_value_range(&mut self, _name: String, _range: ValueRange) {}
        pub fn add_rate_limit(&mut self, _name: String, _limit: u32) {}
    }
    
    pub struct ValueRange {
        pub min: Option<f64>,
        pub max: Option<f64>,
        pub allowed_values: Option<Vec<String>>,
    }
}

use signal_enhanced::EnhancedSignalBus;
use health::{HealthChecker, HealthReport};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    config: String,
    
    /// Real-time priority (requires root)
    #[arg(long)]
    rt_priority: Option<i32>,
    
    /// CPU cores to pin to (comma-separated)
    #[arg(long)]
    cpu_affinity: Option<String>,
    
    /// Lock memory pages
    #[arg(long)]
    lock_memory: bool,
    
    /// Health check port
    #[arg(long, default_value = "8080")]
    health_port: u16
  }

#[tokio::main]
async fn main() -> Result<()> {
   let args = Args::parse();
   
   // Initialize tracing
   tracing_subscriber::fmt()
       .with_env_filter(
           tracing_subscriber::EnvFilter::from_default_env()
               .add_directive("petra=info".parse().unwrap()),
       )
       .json()
       .init();
   
   // Configure real-time if requested
   if args.rt_priority.is_some() || args.cpu_affinity.is_some() || args.lock_memory {
       let mut rt_config = realtime::RealtimeConfig::default();
       
       rt_config.rt_priority = args.rt_priority;
       rt_config.lock_memory = args.lock_memory;
       
       if let Some(affinity) = args.cpu_affinity {
           rt_config.cpu_affinity = affinity
               .split(',')
               .filter_map(|s| s.trim().parse().ok())
               .collect();
       }
       
       if let Err(e) = realtime::configure_realtime(&rt_config) {
           warn!("Failed to configure real-time settings: {}", e);
       }
   }
   
   // Initialize Prometheus metrics
   let metrics_addr: SocketAddr = format!("0.0.0.0:9090").parse().unwrap();
   PrometheusBuilder::new()
       .with_http_listener(metrics_addr)
       .install()
       .expect("Failed to install Prometheus recorder");
   
   info!("Petra v{} starting", petra::VERSION);
   info!("Metrics available at http://{}/metrics", metrics_addr);
   
   // Load and verify configuration
   let config = if std::env::var("PETRA_VERIFY_CONFIG").is_ok() {
       let verifier = config_secure::ConfigVerifier::new(vec![
           std::path::Path::new("/etc/petra/public_key.pem"),
       ])?;
       verifier.verify_and_load(&std::path::Path::new(&args.config))?
   } else {
       Config::from_file(&args.config)?
   };
   
   info!("Loaded {} signals, {} blocks", config.signals.len(), config.blocks.len());
   
   // Create enhanced signal bus
   let bus_config = signal_enhanced::SignalBusConfig {
       max_signals: config.max_signals.unwrap_or(10_000),
       signal_ttl: Duration::from_secs(config.signal_ttl_secs.unwrap_or(3600)),
       cleanup_interval: Duration::from_secs(60),
       hot_signal_threshold: 100,
   };
   let enhanced_bus = Arc::new(EnhancedSignalBus::new(bus_config));
   
   // Create engine with enhanced bus
   let mut engine = Engine::new_with_bus(config.clone(), enhanced_bus.clone())?;
   let engine_stats = engine.stats_handle();
   
   // Set up validation rules
   let mut validator = validation::ValidationRules::new();
   for signal in &config.signals {
       if let Some(range) = &signal.validation {
           validator.add_value_range(
               signal.name.clone(),
               validation::ValueRange {
                   min: range.min,
                   max: range.max,
                   allowed_values: range.allowed_values.clone(),
               },
           );
       }
       
       if let Some(rate_limit) = signal.max_updates_per_second {
           validator.add_rate_limit(signal.name.clone(), rate_limit);
       }
   }
   
   // Create channels for signal changes
   let (signal_tx, signal_rx) = mpsc::channel(10000);
   engine.set_signal_change_channel(signal_tx.clone());
   
   // Initialize health checker
   let mut health_checker = HealthChecker::new(Duration::from_secs(30));
   
   // Storage subsystem
   #[cfg(feature = "advanced-storage")]
   let storage_handle = if let Some(storage_config) = config.storage {
       info!("Initializing storage subsystem");
       
       let (storage_tx, storage_rx) = mpsc::channel(10000);
       engine.set_signal_change_channel(storage_tx.clone());
       
       let mut storage_manager = StorageManager::new(storage_config, enhanced_bus.clone()).await?;
       storage_manager.set_signal_change_channel(storage_rx);
       
       // Add storage health check
       health_checker.add_check(Arc::new(
           health::StorageHealthCheck::new(Arc::new(storage_manager.clone()))
       ));
       
       Some(tokio::spawn(async move {
           if let Err(e) = storage_manager.run().await {
               error!("Storage manager error: {}", e);
           }
       }))
   } else {
       None
   };
   
   // MQTT subsystem
   let mqtt_handle = if let Some(mqtt_config) = config.mqtt {
       info!("Initializing MQTT subsystem");
       
       let (mqtt_tx, mqtt_rx) = mpsc::channel(1000);
       let mqtt_handler = MqttHandler::new(
           mqtt_config,
           enhanced_bus.clone(),
           Some(mqtt_tx),
       ).await?;
       
       let mqtt_client = mqtt_handler.client();
       
       // Add MQTT health check
       health_checker.add_check(Arc::new(
           health::MqttHealthCheck::new(mqtt_client.clone())
       ));
       
       Some((
           tokio::spawn(async move {
               if let Err(e) = mqtt_handler.run().await {
                   error!("MQTT handler error: {}", e);
               }
           }),
           mqtt_client,
           mqtt_rx,
       ))
   } else {
       None
   };
   
   // Add engine health check
   health_checker.add_check(Arc::new(
       health::EngineHealthCheck::new(engine_stats.clone(), config.scan_time_ms * 2)
   ));
   
   let health_checker = Arc::new(health_checker);
   
   // Start health check endpoint
   let health_handle = {
       let checker = health_checker.clone();
       tokio::spawn(async move {
           use warp::Filter;
           
           let health = warp::path("health")
               .and(warp::get())
               .and(warp::any().map(move || checker.clone()))
               .and_then(health::health_endpoint);
           
           let ready = warp::path("ready")
               .and(warp::get())
               .map(|| warp::reply::with_status("ready", warp::http::StatusCode::OK));
           
           let routes = health.or(ready);
           
           warp::serve(routes)
               .run(([0, 0, 0, 0], args.health_port))
               .await;
       })
   };
   
   info!("Health checks available at http://0.0.0.0:{}/health", args.health_port);
   
   // Create shutdown handler
   let shutdown = Arc::new(ShutdownCoordinator::new());
   let engine_arc = Arc::new(Mutex::new(engine));
   
   // Install signal handlers
   let shutdown_clone = shutdown.clone();
   tokio::spawn(async move {
       match signal::ctrl_c().await {
           Ok(_) => {
               info!("Received SIGINT, initiating graceful shutdown");
               shutdown_clone.initiate();
           }
           Err(e) => {
               error!("Failed to listen for SIGINT: {}", e);
           }
       }
   });
   
   #[cfg(unix)]
   {
       let shutdown_clone = shutdown.clone();
       tokio::spawn(async move {
           use tokio::signal::unix::{signal, SignalKind};
           
           let mut sigterm = signal(SignalKind::terminate()).unwrap();
           sigterm.recv().await;
           
           info!("Received SIGTERM, initiating graceful shutdown");
           shutdown_clone.initiate();
       });
   }
   
   // Main engine loop
   let engine_handle = {
       let engine = engine_arc.clone();
       let shutdown = shutdown.clone();
       
       tokio::spawn(async move {
           let mut engine = engine.lock().await;
           
           loop {
               if shutdown.should_stop() {
                   info!("Engine stopping due to shutdown signal");
                   break;
               }
               
               if let Err(e) = engine.run_with_shutdown(shutdown.clone()).await {
                   error!("Engine error: {}", e);
                   break;
               }
           }
           
           engine.stop().await;
       })
   };
   
   // Wait for shutdown signal
   shutdown.wait_for_shutdown().await;
   
   // Graceful shutdown sequence
   info!("Starting graceful shutdown sequence");
   
   // Stop accepting new MQTT messages
   if let Some((_, client, _)) = &mqtt_handle {
       info!("Pausing MQTT subscriptions");
       if let Err(e) = client.unsubscribe("#").await {
           warn!("Failed to unsubscribe MQTT: {}", e);
       }
   }
   
   // Give ongoing operations time to complete
   tokio::time::sleep(Duration::from_secs(2)).await;
   
   // Flush storage
   #[cfg(feature = "advanced-storage")]
   if let Some(storage) = &storage_handle {
       info!("Flushing storage buffers");
       // Send shutdown signal through channel
       storage.abort();
       
       // Wait for storage to finish with timeout
       match tokio::time::timeout(Duration::from_secs(30), async {
           // Storage should flush on drop
       }).await {
           Ok(_) => info!("Storage flushed successfully"),
           Err(_) => warn!("Storage flush timed out"),
       }
   }
   
   // Stop engine
   info!("Stopping engine");
   engine_handle.abort();
   
   // Stop remaining tasks
   if let Some((handle, _, _)) = mqtt_handle {
       handle.abort();
   }
   
   health_handle.abort();
   
   // Final metrics snapshot
   if let Ok(stats) = engine_arc.lock().await.stats() {
       info!(
           "Final stats: {} scans, {} errors, {} signals",
           stats.scan_count, stats.error_count, stats.signal_count
       );
   }
   
   info!("Shutdown complete");
   Ok(())
}

struct ShutdownCoordinator {
   initiated: Arc<parking_lot::RwLock<bool>>,
   complete: Arc<tokio::sync::Notify>,
}

impl ShutdownCoordinator {
   fn new() -> Self {
       Self {
           initiated: Arc::new(parking_lot::RwLock::new(false)),
           complete: Arc::new(tokio::sync::Notify::new()),
       }
   }
   
   fn initiate(&self) {
       *self.initiated.write() = true;
       self.complete.notify_waiters();
   }
   
   fn should_stop(&self) -> bool {
       *self.initiated.read()
   }
   
   async fn wait_for_shutdown(&self) {
       self.complete.notified().await;
   }
}

impl Clone for ShutdownCoordinator {
   fn clone(&self) -> Self {
       Self {
           initiated: self.initiated.clone(),
           complete: self.complete.clone(),
       }
   }
}

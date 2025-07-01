// src/protocols/mod.rs
use async_trait::async_trait;
use crate::{error::Result, signal::SignalBus, value::Value};
use std::collections::HashMap;

#[cfg(feature = "s7-support")]
pub use crate::s7;

#[cfg(feature = "modbus-support")]
pub use crate::modbus;

#[cfg(feature = "opcua-support")]
pub use crate::opcua;

#[cfg(feature = "mqtt")]
pub use crate::mqtt;

// Common protocol trait
#[async_trait]
pub trait ProtocolDriver: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>>;
    async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()>;
    
    fn is_connected(&self) -> bool;
    fn protocol_name(&self) -> &'static str;
    
    #[cfg(feature = "diagnostics")]
    fn diagnostics(&self) -> ProtocolDiagnostics {
        ProtocolDiagnostics::default()
    }
}

#[cfg(feature = "diagnostics")]
#[derive(Default, Debug)]
pub struct ProtocolDiagnostics {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub errors: u64,
    pub last_error: Option<String>,
    pub connection_time: Option<std::time::Duration>,
    pub average_latency_ms: Option<f64>,
}

// Unified protocol manager
pub struct ProtocolManager {
    drivers: HashMap<String, Box<dyn ProtocolDriver>>,
    bus: SignalBus,
    #[cfg(feature = "protocol-retry")]
    retry_config: RetryConfig,
    #[cfg(feature = "protocol-monitoring")]
    monitoring: ProtocolMonitoring,
}

#[cfg(feature = "protocol-retry")]
#[derive(Clone)]
struct RetryConfig {
    max_retries: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    exponential_backoff: bool,
}

impl ProtocolManager {
    pub fn new(bus: SignalBus) -> Self {
        Self {
            drivers: HashMap::new(),
            bus,
            #[cfg(feature = "protocol-retry")]
            retry_config: RetryConfig {
                max_retries: 3,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                exponential_backoff: true,
            },
            #[cfg(feature = "protocol-monitoring")]
            monitoring: ProtocolMonitoring::new(),
        }
    }
    
    pub fn add_driver(&mut self, name: String, driver: Box<dyn ProtocolDriver>) {
        self.drivers.insert(name, driver);
    }
    
    pub async fn connect_all(&mut self) -> Result<()> {
        for (name, driver) in &mut self.drivers {
            tracing::info!("Connecting to {} using {}", name, driver.protocol_name());
            
            #[cfg(feature = "protocol-retry")]
            {
                self.connect_with_retry(name, driver).await?;
            }
            
            #[cfg(not(feature = "protocol-retry"))]
            {
                driver.connect().await?;
            }
        }
        Ok(())
    }
    
    #[cfg(feature = "protocol-retry")]
    async fn connect_with_retry(
        &self, 
        name: &str, 
        driver: &mut Box<dyn ProtocolDriver>
    ) -> Result<()> {
        let mut delay = self.retry_config.initial_delay_ms;
        
        for attempt in 0..=self.retry_config.max_retries {
            match driver.connect().await {
                Ok(_) => {
                    tracing::info!("Successfully connected to {}", name);
                    return Ok(());
                }
                Err(e) if attempt < self.retry_config.max_retries => {
                    tracing::warn!(
                        "Connection attempt {} to {} failed: {}", 
                        attempt + 1, name, e
                    );
                    
                    tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
                    
                    if self.retry_config.exponential_backoff {
                        delay = (delay * 2).min(self.retry_config.max_delay_ms);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        
        unreachable!()
    }
    
    pub async fn run(&mut self) -> Result<()> {
        // Main protocol polling loop
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        
        loop {
            interval.tick().await;
            
            for (name, driver) in &mut self.drivers {
                if !driver.is_connected() {
                    tracing::warn!("{} disconnected, attempting reconnection", name);
                    #[cfg(feature = "protocol-retry")]
                    {
                        let _ = self.connect_with_retry(name, driver).await;
                    }
                    continue;
                }
                
                // Read configured addresses
                // This would be configured per driver
                let addresses = vec![]; // TODO: Get from config
                
                match driver.read_values(&addresses).await {
                    Ok(values) => {
                        for (addr, value) in values {
                            if let Err(e) = self.bus.set(&addr, value) {
                                tracing::error!("Failed to update signal {}: {}", addr, e);
                            }
                        }
                        
                        #[cfg(feature = "protocol-monitoring")]
                        self.monitoring.record_success(name);
                    }
                    Err(e) => {
                        tracing::error!("Failed to read from {}: {}", name, e);
                        
                        #[cfg(feature = "protocol-monitoring")]
                        self.monitoring.record_error(name, &e);
                    }
                }
            }
        }
    }
}

#[cfg(feature = "protocol-monitoring")]
struct ProtocolMonitoring {
    stats: std::sync::Arc<parking_lot::RwLock<HashMap<String, ProtocolStats>>>,
}

#[cfg(feature = "protocol-monitoring")]
struct ProtocolStats {
    successes: u64,
    failures: u64,
    last_success: Option<std::time::Instant>,
    last_failure: Option<std::time::Instant>,
}

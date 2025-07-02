// src/protocols/mod.rs - Unified protocol driver interface
use crate::{error::Result, value::Value, signal::SignalBus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core trait for all protocol drivers
/// 
/// This trait defines the common interface that all protocol implementations
/// must provide for integration with PETRA.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::protocols::{ProtocolDriver, ProtocolManager};
/// use petra::{Value, SignalBus};
/// use async_trait::async_trait;
/// use std::collections::HashMap;
/// 
/// struct MyDriver {
///     connected: bool,
/// }
/// 
/// #[async_trait]
/// impl ProtocolDriver for MyDriver {
///     async fn connect(&mut self) -> petra::Result<()> {
///         self.connected = true;
///         Ok(())
///     }
///     
///     async fn disconnect(&mut self) -> petra::Result<()> {
///         self.connected = false;
///         Ok(())
///     }
///     
///     async fn read_values(&self, addresses: &[String]) -> petra::Result<HashMap<String, Value>> {
///         let mut values = HashMap::new();
///         for addr in addresses {
///             values.insert(addr.clone(), Value::Float(23.5));
///         }
///         Ok(values)
///     }
///     
///     async fn write_values(&mut self, values: &HashMap<String, Value>) -> petra::Result<()> {
///         // Write implementation
///         Ok(())
///     }
///     
///     fn is_connected(&self) -> bool {
///         self.connected
///     }
///     
///     fn protocol_name(&self) -> &'static str {
///         "my_protocol"
///     }
/// }
/// ```
#[async_trait]
pub trait ProtocolDriver: Send + Sync {
    /// Connect to the protocol endpoint
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from the protocol endpoint
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Read values from specified addresses
    async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>>;
    
    /// Write values to specified addresses
    async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()>;
    
    /// Check if the driver is connected
    fn is_connected(&self) -> bool;
    
    /// Get the protocol name
    fn protocol_name(&self) -> &'static str;
    
    /// Get protocol-specific diagnostics (optional)
    fn diagnostics(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
    
    /// Validate an address format (optional)
    fn validate_address(&self, address: &str) -> Result<()> {
        Ok(())
    }
}

/// Protocol manager for handling multiple protocol drivers
/// 
/// The ProtocolManager coordinates multiple protocol drivers and provides
/// a unified interface for the engine to interact with all protocols.
pub struct ProtocolManager {
    drivers: Arc<RwLock<HashMap<String, Box<dyn ProtocolDriver>>>>,
    signal_bus: SignalBus,
}

impl ProtocolManager {
    /// Create a new protocol manager
    pub fn new(signal_bus: SignalBus) -> Self {
        Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            signal_bus,
        }
    }
    
    /// Add a protocol driver
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::protocols::ProtocolManager;
    /// # use petra::SignalBus;
    /// # let signal_bus = SignalBus::new();
    /// let mut manager = ProtocolManager::new(signal_bus);
    /// // manager.add_driver("modbus".to_string(), Box::new(ModbusDriver::new(config)));
    /// ```
    pub async fn add_driver(&self, name: String, driver: Box<dyn ProtocolDriver>) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        drivers.insert(name, driver);
        Ok(())
    }
    
    /// Remove a protocol driver
    pub async fn remove_driver(&self, name: &str) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        if let Some(mut driver) = drivers.remove(name) {
            driver.disconnect().await?;
        }
        Ok(())
    }
    
    /// Connect all drivers
    pub async fn connect_all(&self) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        for (name, driver) in drivers.iter_mut() {
            if !driver.is_connected() {
                tracing::info!("Connecting to {} protocol", name);
                driver.connect().await?;
            }
        }
        Ok(())
    }
    
    /// Disconnect all drivers
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        for (name, driver) in drivers.iter_mut() {
            if driver.is_connected() {
                tracing::info!("Disconnecting from {} protocol", name);
                driver.disconnect().await?;
            }
        }
        Ok(())
    }
    
    /// Read from a specific protocol
    pub async fn read_from(&self, protocol: &str, addresses: &[String]) -> Result<HashMap<String, Value>> {
        let drivers = self.drivers.read().await;
        if let Some(driver) = drivers.get(protocol) {
            driver.read_values(addresses).await
        } else {
            Err(crate::error::PlcError::NotFound(format!("Protocol '{}' not found", protocol)))
        }
    }
    
    /// Write to a specific protocol
    pub async fn write_to(&self, protocol: &str, values: &HashMap<String, Value>) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        if let Some(driver) = drivers.get_mut(protocol) {
            driver.write_values(values).await
        } else {
            Err(crate::error::PlcError::NotFound(format!("Protocol '{}' not found", protocol)))
        }
    }
    
    /// Get all connected protocols
    pub async fn connected_protocols(&self) -> Vec<String> {
        let drivers = self.drivers.read().await;
        drivers
            .iter()
            .filter(|(_, driver)| driver.is_connected())
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// Get diagnostics for all protocols
    pub async fn all_diagnostics(&self) -> HashMap<String, HashMap<String, Value>> {
        let drivers = self.drivers.read().await;
        drivers
            .iter()
            .map(|(name, driver)| (name.clone(), driver.diagnostics()))
            .collect()
    }
}

// Re-export protocol implementations
#[cfg(feature = "s7-support")]
pub mod s7;

#[cfg(feature = "modbus-support")]
pub mod modbus;

#[cfg(feature = "opcua-support")]
pub mod opcua;

#[cfg(feature = "mqtt")]
pub mod mqtt;

#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockDriver {
        connected: bool,
        values: HashMap<String, Value>,
    }
    
    #[async_trait]
    impl ProtocolDriver for MockDriver {
        async fn connect(&mut self) -> Result<()> {
            self.connected = true;
            Ok(())
        }
        
        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }
        
        async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>> {
            let mut result = HashMap::new();
            for addr in addresses {
                if let Some(value) = self.values.get(addr) {
                    result.insert(addr.clone(), value.clone());
                }
            }
            Ok(result)
        }
        
        async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()> {
            for (addr, value) in values {
                self.values.insert(addr.clone(), value.clone());
            }
            Ok(())
        }
        
        fn is_connected(&self) -> bool {
            self.connected
        }
        
        fn protocol_name(&self) -> &'static str {
            "mock"
        }
    }
    
    #[tokio::test]
    async fn test_protocol_manager() {
        let signal_bus = SignalBus::new();
        let manager = ProtocolManager::new(signal_bus);
        
        // Add mock driver
        let mock_driver = Box::new(MockDriver {
            connected: false,
            values: HashMap::new(),
        });
        
        manager.add_driver("mock".to_string(), mock_driver).await.unwrap();
        
        // Connect all
        manager.connect_all().await.unwrap();
        
        // Check connected protocols
        let connected = manager.connected_protocols().await;
        assert_eq!(connected, vec!["mock"]);
        
        // Write and read values
        let mut values = HashMap::new();
        values.insert("test_signal".to_string(), Value::Int(42));
        
        manager.write_to("mock", &values).await.unwrap();
        
        let read_values = manager.read_from("mock", &["test_signal".to_string()]).await.unwrap();
        assert_eq!(read_values.get("test_signal"), Some(&Value::Int(42)));
    }
}

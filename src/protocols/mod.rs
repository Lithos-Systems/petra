// ================================================================================
// PETRA - Industrial Automation System
// Protocol Driver Framework Module
// ================================================================================
//
// PURPOSE:
// This module provides the unified protocol driver interface for PETRA, defining
// the common traits and management infrastructure that all protocol implementations
// must follow. It serves as the abstraction layer between the PETRA engine and
// various industrial/IoT communication protocols.
//
// INTERACTIONS:
// - Used by: engine.rs (for protocol communication during scan cycles)
// - Uses: error.rs (PlcError types), value.rs (Value enum), signal.rs (SignalBus)
// - Implemented by: mqtt.rs, modbus.rs, s7.rs, opcua.rs (protocol drivers)
// - Configured by: config.rs (protocol configurations)
//
// DESIGN PRINCIPLES:
// - Protocol agnostic: Core system doesn't need to know protocol specifics
// - Async-first: All I/O operations are async for non-blocking operation
// - Error resilient: Graceful handling of connection failures
// - Thread-safe: Multiple threads can safely interact with protocols
// - Feature-gated: Each protocol is optional via feature flags
//
// ================================================================================

use crate::{error::Result, value::Value, signal::SignalBus};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ================================================================================
// PROTOCOL DRIVER TRAIT
// ================================================================================

/// Core trait that all protocol drivers must implement
/// 
/// This trait defines the common interface for protocol implementations,
/// ensuring consistent behavior across different communication protocols.
/// All methods are designed to be non-blocking and thread-safe.
/// 
/// # Implementation Guidelines
/// 
/// - **Connection Management**: Implement automatic reconnection logic
/// - **Error Handling**: Convert protocol-specific errors to PlcError::Protocol
/// - **Value Mapping**: Convert protocol data types to PETRA Value types
/// - **Thread Safety**: Ensure all methods are safe for concurrent access
/// - **Performance**: Minimize allocations and use efficient data structures
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
///     endpoint: String,
///     timeout_ms: u64,
/// }
/// 
/// #[async_trait]
/// impl ProtocolDriver for MyDriver {
///     async fn connect(&mut self) -> petra::Result<()> {
///         // Implement connection logic with timeout
///         self.connected = true;
///         log::info!("Connected to {} endpoint", self.endpoint);
///         Ok(())
///     }
///     
///     async fn disconnect(&mut self) -> petra::Result<()> {
///         // Implement graceful disconnection
///         self.connected = false;
///         log::info!("Disconnected from {} endpoint", self.endpoint);
///         Ok(())
///     }
///     
///     async fn read_values(&self, addresses: &[String]) -> petra::Result<HashMap<String, Value>> {
///         // Validate connection state
///         if !self.connected {
///             return Err(petra::error::PlcError::Protocol(
///                 "Not connected".to_string()
///             ));
///         }
///         
///         // Read values with error handling
///         let mut values = HashMap::new();
///         for addr in addresses {
///             // Protocol-specific read logic
///             values.insert(addr.clone(), Value::Float(23.5));
///         }
///         Ok(values)
///     }
///     
///     async fn write_values(&mut self, values: &HashMap<String, Value>) -> petra::Result<()> {
///         // Validate connection and write values
///         if !self.connected {
///             return Err(petra::error::PlcError::Protocol(
///                 "Not connected".to_string()
///             ));
///         }
///         
///         // Protocol-specific write logic
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
    /// Establish connection to the protocol endpoint
    /// 
    /// This method should handle all connection setup including:
    /// - Network connection establishment
    /// - Authentication/authorization
    /// - Initial handshaking
    /// - Connection parameter negotiation
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Protocol` if connection fails
    async fn connect(&mut self) -> Result<()>;
    
    /// Gracefully disconnect from the protocol endpoint
    /// 
    /// This method should:
    /// - Send any necessary disconnect messages
    /// - Clean up resources
    /// - Reset internal state
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Protocol` if disconnection fails
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Read values from specified addresses
    /// 
    /// Addresses are protocol-specific strings that identify data points.
    /// The implementation should batch reads when possible for efficiency.
    /// 
    /// # Arguments
    /// 
    /// * `addresses` - Protocol-specific address strings
    /// 
    /// # Returns
    /// 
    /// HashMap mapping addresses to their current values
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Protocol` if read operation fails
    async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>>;
    
    /// Write values to specified addresses
    /// 
    /// The implementation should validate values before writing and
    /// batch writes when possible for efficiency.
    /// 
    /// # Arguments
    /// 
    /// * `values` - HashMap of address to value mappings
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Protocol` if write operation fails
    async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()>;
    
    /// Check if the driver is currently connected
    /// 
    /// This should return the actual connection state, not just
    /// whether connect() was called.
    fn is_connected(&self) -> bool;
    
    /// Get the protocol name for identification
    /// 
    /// This should be a unique, human-readable identifier
    fn protocol_name(&self) -> &'static str;
    
    /// Get protocol-specific diagnostics (optional)
    /// 
    /// Returns diagnostic information such as:
    /// - Connection statistics
    /// - Error counters
    /// - Performance metrics
    /// - Protocol-specific status
    /// 
    /// Default implementation returns empty diagnostics
    fn diagnostics(&self) -> HashMap<String, Value> {
        HashMap::new()
    }
    
    /// Validate an address format (optional)
    /// 
    /// This method can be used to validate address formats before
    /// attempting to read/write, providing early error detection.
    /// 
    /// # Arguments
    /// 
    /// * `address` - The address string to validate
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::Validation` if address format is invalid
    /// 
    /// Default implementation accepts all addresses
    fn validate_address(&self, _address: &str) -> Result<()> {
        Ok(())
    }
    
    /// Get protocol capabilities (optional)
    /// 
    /// Returns information about what the protocol supports:
    /// - Maximum read/write size
    /// - Supported data types
    /// - Feature flags
    fn capabilities(&self) -> HashMap<&'static str, Value> {
        HashMap::new()
    }
}

// ================================================================================
// PROTOCOL MANAGER
// ================================================================================

/// Centralized manager for all protocol drivers
/// 
/// The ProtocolManager coordinates multiple protocol drivers and provides
/// a unified interface for the engine to interact with all protocols.
/// It handles driver lifecycle, connection management, and routing of
/// read/write operations to the appropriate drivers.
/// 
/// # Thread Safety
/// 
/// All methods are thread-safe and can be called concurrently from
/// multiple tasks or threads.
/// 
/// # Examples
/// 
/// ```rust
/// use petra::protocols::ProtocolManager;
/// use petra::SignalBus;
/// 
/// #[tokio::main]
/// async fn main() -> petra::Result<()> {
///     let signal_bus = SignalBus::new();
///     let manager = ProtocolManager::new(signal_bus);
///     
///     // Add protocol drivers
///     // manager.add_driver("modbus".to_string(), Box::new(modbus_driver)).await?;
///     // manager.add_driver("mqtt".to_string(), Box::new(mqtt_driver)).await?;
///     
///     // Connect all protocols
///     manager.connect_all().await?;
///     
///     // Use protocols...
///     
///     // Graceful shutdown
///     manager.disconnect_all().await?;
///     Ok(())
/// }
/// ```
pub struct ProtocolManager {
    /// Thread-safe collection of protocol drivers
    drivers: Arc<RwLock<HashMap<String, Box<dyn ProtocolDriver>>>>,
    
    /// Reference to the signal bus for data exchange
    signal_bus: SignalBus,
    
    /// Performance metrics (when monitoring features are enabled)
    #[cfg(feature = "enhanced-monitoring")]
    metrics: Arc<RwLock<ProtocolMetrics>>,
}

#[cfg(feature = "enhanced-monitoring")]
struct ProtocolMetrics {
    read_count: HashMap<String, u64>,
    write_count: HashMap<String, u64>,
    error_count: HashMap<String, u64>,
    last_error: HashMap<String, String>,
}

impl ProtocolManager {
    /// Create a new protocol manager
    /// 
    /// # Arguments
    /// 
    /// * `signal_bus` - Reference to the system signal bus
    pub fn new(signal_bus: SignalBus) -> Self {
        Self {
            drivers: Arc::new(RwLock::new(HashMap::new())),
            signal_bus,
            #[cfg(feature = "enhanced-monitoring")]
            metrics: Arc::new(RwLock::new(ProtocolMetrics {
                read_count: HashMap::new(),
                write_count: HashMap::new(),
                error_count: HashMap::new(),
                last_error: HashMap::new(),
            })),
        }
    }
    
    /// Add a protocol driver to the manager
    /// 
    /// The driver name must be unique. If a driver with the same name
    /// already exists, it will be replaced after disconnecting the old one.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Unique identifier for the driver
    /// * `driver` - The protocol driver implementation
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// # use petra::protocols::ProtocolManager;
    /// # use petra::SignalBus;
    /// # let signal_bus = SignalBus::new();
    /// let manager = ProtocolManager::new(signal_bus);
    /// // manager.add_driver("modbus".to_string(), Box::new(ModbusDriver::new(config))).await?;
    /// ```
    pub async fn add_driver(&self, name: String, driver: Box<dyn ProtocolDriver>) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        
        // Disconnect existing driver if present
        if let Some(mut old_driver) = drivers.remove(&name) {
            log::info!("Replacing existing {} driver", name);
            if old_driver.is_connected() {
                old_driver.disconnect().await?;
            }
        }
        
        log::info!("Adding {} protocol driver", name);
        drivers.insert(name, driver);
        Ok(())
    }
    
    /// Remove a protocol driver from the manager
    /// 
    /// The driver will be disconnected before removal.
    /// 
    /// # Arguments
    /// 
    /// * `name` - Name of the driver to remove
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::NotFound` if driver doesn't exist
    pub async fn remove_driver(&self, name: &str) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        
        if let Some(mut driver) = drivers.remove(name) {
            log::info!("Removing {} protocol driver", name);
            if driver.is_connected() {
                driver.disconnect().await?;
            }
            Ok(())
        } else {
            Err(crate::error::PlcError::NotFound(
                format!("Protocol driver '{}' not found", name)
            ))
        }
    }
    
    /// Connect all registered drivers
    /// 
    /// Attempts to connect all drivers that are not already connected.
    /// If any driver fails to connect, the error is logged but the
    /// process continues with other drivers.
    /// 
    /// # Returns
    /// 
    /// Ok if at least one driver connected successfully
    /// 
    /// # Errors
    /// 
    /// Returns error if all drivers fail to connect
    pub async fn connect_all(&self) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        let mut any_connected = false;
        let mut all_errors = Vec::new();
        
        for (name, driver) in drivers.iter_mut() {
            if !driver.is_connected() {
                log::info!("Connecting to {} protocol", name);
                match driver.connect().await {
                    Ok(()) => {
                        log::info!("Successfully connected to {} protocol", name);
                        any_connected = true;
                    }
                    Err(e) => {
                        log::error!("Failed to connect to {} protocol: {}", name, e);
                        all_errors.push(format!("{}: {}", name, e));
                        
                        #[cfg(feature = "enhanced-monitoring")]
                        {
                            let mut metrics = self.metrics.write().await;
                            *metrics.error_count.entry(name.clone()).or_insert(0) += 1;
                            metrics.last_error.insert(name.clone(), e.to_string());
                        }
                    }
                }
            }
        }
        
        if !any_connected && !all_errors.is_empty() {
            Err(crate::error::PlcError::Protocol(
                format!("All protocol connections failed: {}", all_errors.join(", "))
            ))
        } else {
            Ok(())
        }
    }
    
    /// Disconnect all connected drivers
    /// 
    /// Attempts to gracefully disconnect all connected drivers.
    /// Errors are logged but don't stop the disconnection process.
    pub async fn disconnect_all(&self) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        
        for (name, driver) in drivers.iter_mut() {
            if driver.is_connected() {
                log::info!("Disconnecting from {} protocol", name);
                match driver.disconnect().await {
                    Ok(()) => log::info!("Successfully disconnected from {} protocol", name),
                    Err(e) => log::error!("Error disconnecting from {} protocol: {}", name, e),
                }
            }
        }
        
        Ok(())
    }
    
    /// Read values from a specific protocol
    /// 
    /// Routes the read request to the appropriate driver based on protocol name.
    /// 
    /// # Arguments
    /// 
    /// * `protocol` - Name of the protocol driver
    /// * `addresses` - Protocol-specific addresses to read
    /// 
    /// # Returns
    /// 
    /// HashMap of address to value mappings
    /// 
    /// # Errors
    /// 
    /// - `PlcError::NotFound` if protocol doesn't exist
    /// - `PlcError::Protocol` if read operation fails
    pub async fn read_from(
        &self, 
        protocol: &str, 
        addresses: &[String]
    ) -> Result<HashMap<String, Value>> {
        let drivers = self.drivers.read().await;
        
        if let Some(driver) = drivers.get(protocol) {
            if !driver.is_connected() {
                return Err(crate::error::PlcError::Protocol(
                    format!("Protocol '{}' is not connected", protocol)
                ));
            }
            
            let result = driver.read_values(addresses).await;
            
            #[cfg(feature = "enhanced-monitoring")]
            {
                let mut metrics = self.metrics.write().await;
                match &result {
                    Ok(_) => {
                        *metrics.read_count.entry(protocol.to_string()).or_insert(0) += 1;
                    }
                    Err(e) => {
                        *metrics.error_count.entry(protocol.to_string()).or_insert(0) += 1;
                        metrics.last_error.insert(protocol.to_string(), e.to_string());
                    }
                }
            }
            
            result
        } else {
            Err(crate::error::PlcError::NotFound(
                format!("Protocol '{}' not found", protocol)
            ))
        }
    }
    
    /// Write values to a specific protocol
    /// 
    /// Routes the write request to the appropriate driver based on protocol name.
    /// 
    /// # Arguments
    /// 
    /// * `protocol` - Name of the protocol driver
    /// * `values` - HashMap of address to value mappings
    /// 
    /// # Errors
    /// 
    /// - `PlcError::NotFound` if protocol doesn't exist
    /// - `PlcError::Protocol` if write operation fails
    pub async fn write_to(
        &self, 
        protocol: &str, 
        values: &HashMap<String, Value>
    ) -> Result<()> {
        let mut drivers = self.drivers.write().await;
        
        if let Some(driver) = drivers.get_mut(protocol) {
            if !driver.is_connected() {
                return Err(crate::error::PlcError::Protocol(
                    format!("Protocol '{}' is not connected", protocol)
                ));
            }
            
            let result = driver.write_values(values).await;
            
            #[cfg(feature = "enhanced-monitoring")]
            {
                let mut metrics = self.metrics.write().await;
                match &result {
                    Ok(()) => {
                        *metrics.write_count.entry(protocol.to_string()).or_insert(0) += 1;
                    }
                    Err(e) => {
                        *metrics.error_count.entry(protocol.to_string()).or_insert(0) += 1;
                        metrics.last_error.insert(protocol.to_string(), e.to_string());
                    }
                }
            }
            
            result
        } else {
            Err(crate::error::PlcError::NotFound(
                format!("Protocol '{}' not found", protocol)
            ))
        }
    }
    
    /// Get list of all connected protocols
    /// 
    /// Returns the names of all protocol drivers that are currently connected.
    pub async fn connected_protocols(&self) -> Vec<String> {
        let drivers = self.drivers.read().await;
        drivers
            .iter()
            .filter(|(_, driver)| driver.is_connected())
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    /// Get list of all registered protocols
    /// 
    /// Returns the names of all protocol drivers, connected or not.
    pub async fn all_protocols(&self) -> Vec<String> {
        let drivers = self.drivers.read().await;
        drivers.keys().cloned().collect()
    }
    
    /// Get diagnostics for all protocols
    /// 
    /// Collects diagnostic information from all registered drivers.
    /// 
    /// # Returns
    /// 
    /// HashMap where keys are protocol names and values are their diagnostics
    pub async fn all_diagnostics(&self) -> HashMap<String, HashMap<String, Value>> {
        let drivers = self.drivers.read().await;
        let mut diagnostics = HashMap::new();
        
        for (name, driver) in drivers.iter() {
            let mut diag = driver.diagnostics();
            
            // Add connection status
            diag.insert(
                "connected".to_string(), 
                Value::Bool(driver.is_connected())
            );
            
            // Add protocol name
            #[cfg(feature = "extended-types")]
            diag.insert(
                "protocol".to_string(),
                Value::String(driver.protocol_name().to_string()),
            );
            
            #[cfg(feature = "enhanced-monitoring")]
            {
                let metrics = self.metrics.read().await;
                if let Some(read_count) = metrics.read_count.get(name) {
                    diag.insert("read_count".to_string(), Value::Integer(*read_count as i64));
                }
                if let Some(write_count) = metrics.write_count.get(name) {
                    diag.insert("write_count".to_string(), Value::Integer(*write_count as i64));
                }
                if let Some(error_count) = metrics.error_count.get(name) {
                    diag.insert("error_count".to_string(), Value::Integer(*error_count as i64));
                }
                if let Some(_last_error) = metrics.last_error.get(name) {
                    #[cfg(feature = "extended-types")]
                    {
                        diag.insert(
                            "last_error".to_string(),
                            Value::String(_last_error.clone()),
                        );
                    }
                }
            }
            
            diagnostics.insert(name.clone(), diag);
        }
        
        diagnostics
    }
    
    /// Get diagnostics for a specific protocol
    /// 
    /// # Arguments
    /// 
    /// * `protocol` - Name of the protocol driver
    /// 
    /// # Returns
    /// 
    /// Diagnostic information for the specified protocol
    /// 
    /// # Errors
    /// 
    /// Returns `PlcError::NotFound` if protocol doesn't exist
    pub async fn protocol_diagnostics(&self, protocol: &str) -> Result<HashMap<String, Value>> {
        let drivers = self.drivers.read().await;
        
        if let Some(driver) = drivers.get(protocol) {
            let mut diag = driver.diagnostics();
            diag.insert("connected".to_string(), Value::Bool(driver.is_connected()));
            #[cfg(feature = "extended-types")]
            diag.insert(
                "protocol".to_string(),
                Value::String(driver.protocol_name().to_string()),
            );
            Ok(diag)
        } else {
            Err(crate::error::PlcError::NotFound(
                format!("Protocol '{}' not found", protocol)
            ))
        }
    }
}

// ================================================================================
// PROTOCOL IMPLEMENTATIONS
// ================================================================================
// Each protocol implementation is feature-gated to minimize binary size

#[cfg(feature = "s7-support")]
pub mod s7;

#[cfg(feature = "modbus-support")]
pub mod modbus;

#[cfg(feature = "opcua-support")]
pub mod opcua;

#[cfg(feature = "mqtt")]
pub mod mqtt;

#[cfg(feature = "zero-copy-protocols")]
pub mod zero_copy;

// ================================================================================
// TESTS
// ================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Mock driver for testing protocol manager functionality
    struct MockDriver {
        connected: bool,
        values: HashMap<String, Value>,
        connect_fail: bool,
        read_fail: bool,
        write_fail: bool,
    }
    
    impl MockDriver {
        fn new() -> Self {
            Self {
                connected: false,
                values: HashMap::new(),
                connect_fail: false,
                read_fail: false,
                write_fail: false,
            }
        }
        
        fn with_failure(mut self, connect: bool, read: bool, write: bool) -> Self {
            self.connect_fail = connect;
            self.read_fail = read;
            self.write_fail = write;
            self
        }
    }
    
    #[async_trait]
    impl ProtocolDriver for MockDriver {
        async fn connect(&mut self) -> Result<()> {
            if self.connect_fail {
                Err(crate::error::PlcError::Protocol("Mock connection failed".to_string()))
            } else {
                self.connected = true;
                Ok(())
            }
        }
        
        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }
        
        async fn read_values(&self, addresses: &[String]) -> Result<HashMap<String, Value>> {
            if self.read_fail {
                return Err(crate::error::PlcError::Protocol("Mock read failed".to_string()));
            }
            
            let mut result = HashMap::new();
            for addr in addresses {
                if let Some(value) = self.values.get(addr) {
                    result.insert(addr.clone(), value.clone());
                } else {
                    result.insert(addr.clone(), Value::Integer(0));
                }
            }
            Ok(result)
        }
        
        async fn write_values(&mut self, values: &HashMap<String, Value>) -> Result<()> {
            if self.write_fail {
                return Err(crate::error::PlcError::Protocol("Mock write failed".to_string()));
            }
            
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
        
        fn diagnostics(&self) -> HashMap<String, Value> {
            let mut diag = HashMap::new();
            diag.insert("test_mode".to_string(), Value::Bool(true));
            diag
        }
    }
    
    #[tokio::test]
    async fn test_protocol_manager_basic() {
        let signal_bus = SignalBus::new();
        let manager = ProtocolManager::new(signal_bus);
        
        // Add mock driver
        let mock_driver = Box::new(MockDriver::new());
        manager.add_driver("mock".to_string(), mock_driver).await.unwrap();
        
        // Verify driver was added
        let all_protocols = manager.all_protocols().await;
        assert_eq!(all_protocols, vec!["mock"]);
        
        // Connect all
        manager.connect_all().await.unwrap();
        
        // Check connected protocols
        let connected = manager.connected_protocols().await;
        assert_eq!(connected, vec!["mock"]);
        
        // Write and read values
        let mut values = HashMap::new();
        values.insert("test_signal".to_string(), Value::Integer(42));
        
        manager.write_to("mock", &values).await.unwrap();
        
        let read_values = manager.read_from("mock", &["test_signal".to_string()]).await.unwrap();
        assert_eq!(read_values.get("test_signal"), Some(&Value::Integer(42)));
        
        // Disconnect all
        manager.disconnect_all().await.unwrap();
        let connected = manager.connected_protocols().await;
        assert!(connected.is_empty());
    }
    
    #[tokio::test]
    async fn test_protocol_manager_errors() {
        let signal_bus = SignalBus::new();
        let manager = ProtocolManager::new(signal_bus);
        
        // Test non-existent protocol
        let result = manager.read_from("nonexistent", &["test".to_string()]).await;
        assert!(matches!(result, Err(crate::error::PlcError::NotFound(_))));
        
        // Add driver that fails to connect
        let fail_driver = Box::new(MockDriver::new().with_failure(true, false, false));
        manager.add_driver("fail".to_string(), fail_driver).await.unwrap();
        
        // Connect should handle the failure gracefully
        let result = manager.connect_all().await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_protocol_manager_diagnostics() {
        let signal_bus = SignalBus::new();
        let manager = ProtocolManager::new(signal_bus);
        
        // Add and connect driver
        let mock_driver = Box::new(MockDriver::new());
        manager.add_driver("mock".to_string(), mock_driver).await.unwrap();
        manager.connect_all().await.unwrap();
        
        // Get diagnostics
        let diag = manager.protocol_diagnostics("mock").await.unwrap();
        assert_eq!(diag.get("connected"), Some(&Value::Bool(true)));
        #[cfg(feature = "extended-types")]
        assert_eq!(
            diag.get("protocol"),
            Some(&Value::String("mock".to_string()))
        );
        assert_eq!(diag.get("test_mode"), Some(&Value::Bool(true)));
        
        // Get all diagnostics
        let all_diag = manager.all_diagnostics().await;
        assert!(all_diag.contains_key("mock"));
    }
    
    #[tokio::test]
    async fn test_protocol_manager_replace_driver() {
        let signal_bus = SignalBus::new();
        let manager = ProtocolManager::new(signal_bus);
        
        // Add first driver
        let driver1 = Box::new(MockDriver::new());
        manager.add_driver("test".to_string(), driver1).await.unwrap();
        manager.connect_all().await.unwrap();
        
        // Replace with second driver
        let driver2 = Box::new(MockDriver::new());
        manager.add_driver("test".to_string(), driver2).await.unwrap();
        
        // Verify still only one driver
        let all_protocols = manager.all_protocols().await;
        assert_eq!(all_protocols.len(), 1);
    }
}

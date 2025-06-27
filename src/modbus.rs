// Add src/modbus.rs
#[cfg(feature = "modbus-support")]
pub mod modbus {
    use tokio_modbus::prelude::*;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ModbusConfig {
        pub connections: Vec<ModbusConnection>,
        pub scan_interval_ms: u64,
        pub timeout_ms: u64,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ModbusConnection {
        pub name: String,
        pub transport: ModbusTransport,
        pub mappings: Vec<ModbusMapping>,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "type")]
    pub enum ModbusTransport {
        Tcp { host: String, port: u16 },
        Rtu { device: String, baud_rate: u32, slave: u8 },
    }
}

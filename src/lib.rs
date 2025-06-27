//! Petra v2.1 - Production-ready PLC with MQTT, S7, and historical data support

pub mod error;
pub mod value;
pub mod signal;
pub mod block;
pub mod config;
pub mod alarms;
pub mod engine;
#[cfg(feature = "enhanced")]
pub mod engine_enhanced;
#[cfg(feature = "enhanced")]
pub mod signal_optimized;
pub mod mqtt;
pub mod twilio;
pub mod twilio_block;
#[cfg(feature = "opcua-support")]
pub mod opcua;

#[cfg(feature = "modbus-support")]
pub mod modbus;

pub mod security;

#[cfg(feature = "json-schema")]
pub mod config_schema;

#[cfg(feature = "history")]
pub mod history;

#[cfg(feature = "s7-support")]
pub mod s7;

#[cfg(feature = "advanced-storage")]
pub mod storage;
#[cfg(feature = "advanced-storage")]
pub use storage::{StorageConfig, StorageManager, WriteAheadLog};

pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
#[cfg(feature = "enhanced")]
pub use signal_optimized::OptimizedSignalBus;
pub use config::Config;
pub use engine::{Engine, EngineStats};
pub use mqtt::{MqttHandler, MqttMessage};

#[cfg(feature = "history")]
pub use history::{HistoryManager, HistoryConfig, SignalHistory};

#[cfg(feature = "s7-support")]
pub use s7::{S7Connector, S7Config, S7Mapping, S7Area, S7DataType, Direction};

pub use twilio::{TwilioConnector, TwilioConfig, TwilioAction, TwilioActionType};

#[cfg(feature = "opcua-support")]
pub use opcua::{OpcUaConfig, OpcUaServer};

#[cfg(feature = "modbus-support")]
pub use modbus::modbus::{ModbusConfig, ModbusConnection, ModbusTransport};

pub use security::{SecurityConfig, AuthToken, UserRole};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

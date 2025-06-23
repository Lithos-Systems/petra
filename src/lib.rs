//! Petra v2.1 - Production-ready PLC with MQTT and S7 integration

pub mod error;
pub mod value;
pub mod signal;
pub mod block;
pub mod config;
pub mod engine;
pub mod mqtt;
pub mod twilio;
pub mod twilio_block;
pub mod history;

#[cfg(feature = "s7-support")]
pub mod s7;

pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use config::Config;
pub use engine::{Engine, EngineStats};
pub use mqtt::{MqttHandler, MqttMessage};

#[cfg(feature = "s7-support")]
pub use s7::{S7Connector, S7Config, S7Mapping, S7Area, S7DataType, Direction};
pub use history::{HistoryManager, HistoryConfig, SignalHistory};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub use twilio::{TwilioConnector, TwilioConfig, TwilioAction, TwilioActionType};

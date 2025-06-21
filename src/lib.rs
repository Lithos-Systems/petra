//! Petra v2.0 - Production-ready PLC with MQTT integration

pub mod error;
pub mod value;
pub mod signal;
pub mod block;
pub mod config;
pub mod s7;
pub mod engine;
pub mod mqtt;

pub use error::{PlcError, Result};
pub use value::Value;
pub use signal::SignalBus;
pub use config::Config;
pub use engine::{Engine, EngineStats};
pub use mqtt::{MqttHandler, MqttMessage};
pub use s7::S7Connector;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

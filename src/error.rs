use thiserror::Error;

/// Application level error type used throughout the crate.
#[derive(Error, Debug)]
pub enum PlcError {
    /// I/O related failure
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid or inconsistent configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error while parsing YAML configuration files
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// Requested signal was not found in the bus
    #[error("Signal not found: {0}")]
    SignalNotFound(String),

    /// Returned value type does not match the expected type
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: &'static str, actual: &'static str },
}

/// Convenient alias over [`Result`] using [`PlcError`]
pub type Result<T> = std::result::Result<T, PlcError>;

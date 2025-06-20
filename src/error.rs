use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlcError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Signal '{0}' not found")]
    SignalNotFound(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: &'static str, actual: &'static str },

    #[error("Block execution error in '{0}': {1}")]
    BlockExecution(String, String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, PlcError>;

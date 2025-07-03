// src/error.rs
use thiserror::Error;

#[cfg(feature = "metrics")]
use metrics::SetRecorderError;

#[cfg(feature = "metrics")]
impl<T> From<SetRecorderError<T>> for PlcError {
    fn from(err: SetRecorderError<T>) -> Self {
        PlcError::Config(format!("Failed to set metrics recorder: {}", err))
    }
}

// Add this if using serde_json
impl From<serde_json::Error> for PlcError {
    fn from(err: serde_json::Error) -> Self {
        PlcError::Config(format!("JSON error: {}", err))
    }
}

// Add this for socket address parsing
impl From<std::net::AddrParseError> for PlcError {
    fn from(err: std::net::AddrParseError) -> Self {
        PlcError::Config(format!("Invalid address: {}", err))
    }
}

#[derive(Error, Debug)]
pub enum PlcError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Signal not found
    #[error("Signal not found: {0}")]
    SignalNotFound(String),

    /// Generic not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Type mismatch error - Fixed to use struct format
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },
    
    /// Runtime error
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    /// IO error
    #[error("IO error")]
    Io(#[from] std::io::Error),

    /// Serde YAML error
    #[error("YAML error")]
    Yaml(#[from] serde_yaml::Error),

    /// Security error
    #[error("Security error: {0}")]
    Security(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Block error
    #[error("Block error: {0}")]
    Block(String),

    /// Signal error
    #[error("Signal error: {0}")]
    Signal(String),
    
    /// Metrics error (for Prometheus)
    #[cfg(feature = "metrics")]
    #[error("Metrics error")]
    Metrics(#[from] metrics_exporter_prometheus::BuildError),
    
    // Feature-specific errors
    #[cfg(feature = "s7-support")]
    #[error("S7 communication error: {0}")]
    S7(String),
    
    #[cfg(feature = "modbus-support")]
    #[error("Modbus error: {0}")]
    Modbus(#[from] tokio_modbus::Error),
    
    #[cfg(feature = "opcua-support")]
    #[error("OPC-UA error: {0}")]
    OpcUa(String),
    
    #[cfg(feature = "mqtt")]
    #[error("MQTT error: {0}")]
    Mqtt(String),
    
    #[cfg(feature = "advanced-storage")]
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[cfg(feature = "circuit-breaker")]
    #[error("Circuit breaker open")]
    CircuitOpen,
    
    #[cfg(feature = "security")]
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[cfg(feature = "security")]
    #[error("Authorization denied")]
    AuthorizationDenied,
    
    #[cfg(feature = "web")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Web error: {0}")]
    Web(String),
    
    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(String),

    #[cfg(feature = "quality-codes")]
    #[error("Signal quality error: {0}")]
    SignalQuality(String),
    
    // Enhanced error information
    #[cfg(feature = "enhanced-errors")]
    #[error("Detailed error: {message} at {location}")]
    Detailed {
        message: String,
        location: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: std::collections::HashMap<String, String>,
    },
}

pub type Result<T> = std::result::Result<T, PlcError>;

// Enhanced error creation helpers
#[cfg(feature = "enhanced-errors")]
impl PlcError {
    pub fn detailed(message: impl Into<String>) -> DetailedErrorBuilder {
        DetailedErrorBuilder {
            message: message.into(),
            location: String::new(),
            source: None,
            context: std::collections::HashMap::new(),
        }
    }
}

#[cfg(feature = "enhanced-errors")]
pub struct DetailedErrorBuilder {
    message: String,
    location: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    context: std::collections::HashMap<String, String>,
}

#[cfg(feature = "enhanced-errors")]
impl DetailedErrorBuilder {
    pub fn at(mut self, location: impl Into<String>) -> Self {
        self.location = location.into();
        self
    }
    
    pub fn caused_by(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }
    
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    pub fn build(self) -> PlcError {
        PlcError::Detailed {
            message: self.message,
            location: self.location,
            source: self.source,
            context: self.context,
        }
    }
}

// Error recovery suggestions
#[cfg(feature = "error-recovery")]
pub trait ErrorRecovery {
    fn recovery_suggestions(&self) -> Vec<String>;
    fn is_recoverable(&self) -> bool;
}

#[cfg(feature = "error-recovery")]
impl ErrorRecovery for PlcError {
    fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            PlcError::SignalNotFound(name) => vec![
                format!("Check if signal '{}' is defined in configuration", name),
                "Verify signal name spelling".to_string(),
                "Ensure signal is initialized before use".to_string(),
            ],
            PlcError::Config(msg) => vec![
                "Check YAML syntax".to_string(),
                "Validate configuration against schema".to_string(),
                format!("Error details: {}", msg),
            ],
            PlcError::TypeMismatch { expected, actual } => vec![
                format!("Convert {} to {} type", actual, expected),
                "Check signal type in configuration".to_string(),
                "Use appropriate conversion method".to_string(),
            ],
            #[cfg(feature = "circuit-breaker")]
            PlcError::CircuitOpen => vec![
                "Wait for circuit breaker to reset".to_string(),
                "Check system health".to_string(),
                "Review error logs for root cause".to_string(),
            ],
            _ => vec!["Review error message for details".to_string()],
        }
    }
    
    fn is_recoverable(&self) -> bool {
        !matches!(
            self,
            PlcError::Config(_) | PlcError::Io(_) | PlcError::Security(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PlcError::SignalNotFound("test_signal".to_string());
        assert_eq!(err.to_string(), "Signal not found: test_signal");
    }

    #[test]
    fn test_type_mismatch() {
        let err = PlcError::TypeMismatch {
            expected: "bool".to_string(),
            actual: "int".to_string(),
        };
        assert_eq!(err.to_string(), "Type mismatch: expected bool, got int");
    }

    #[cfg(feature = "error-recovery")]
    #[test]
    fn test_recovery_suggestions() {
        let err = PlcError::SignalNotFound("missing".to_string());
        let suggestions = err.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(err.is_recoverable());
    }
}

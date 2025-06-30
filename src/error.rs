// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlcError {
    // Core errors (always available)
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Signal not found: {0}")]
    SignalNotFound(String),
    
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
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
    #[error("Security error: {0}")]
    Security(String),
    
    #[cfg(feature = "security")]
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[cfg(feature = "security")]
    #[error("Authorization denied")]
    AuthorizationDenied,
    
    #[cfg(feature = "web")]
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(String),
    
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
                "Verify signal name spelling and case".to_string(),
            ],
            PlcError::Config(msg) => vec![
                "Validate configuration file syntax".to_string(),
                format!("Error details: {}", msg),
            ],
            #[cfg(feature = "s7-support")]
            PlcError::S7(msg) => vec![
                "Check PLC connection and network".to_string(),
                "Verify PLC is powered on and accessible".to_string(),
                format!("S7 error: {}", msg),
            ],
            _ => vec!["Check logs for more details".to_string()],
        }
    }
    
    fn is_recoverable(&self) -> bool {
        match self {
            PlcError::Config(_) => false,
            PlcError::SignalNotFound(_) => false,
            PlcError::TypeMismatch(_) => false,
            #[cfg(feature = "circuit-breaker")]
            PlcError::CircuitOpen => true,
            _ => true,
        }
    }
}

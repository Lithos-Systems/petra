// Enhance src/error.rs
#[derive(Error, Debug)]
pub enum PlcError {
    #[error("Security violation: {0}")]
    Security(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),
    
    #[error("Dependency failure: {service} - {reason}")]
    DependencyFailure { service: String, reason: String },
    
    // Add structured error codes for external systems
    #[error("E{code:04}: {message}")]
    Coded { code: u16, message: String },
}

// Add metrics and tracing
use tracing::{instrument, error_span};
use metrics::{counter, histogram, gauge};

#[instrument(skip(self), fields(node_count = %self.nodes.len()))]
pub async fn run(&mut self) -> Result<()> {
    let _span = error_span!("engine_run").entered();
    // ... existing code with enhanced telemetry
}

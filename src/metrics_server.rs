// src/metrics_server.rs
use crate::error::*;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use serde::{Serialize, Deserialize};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;

// MetricsConfig definition - always available regardless of feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
pub struct MetricsConfig {
    /// Address to bind the metrics server to
    pub bind_address: String,
    /// Enable/disable metrics collection
    pub enabled: bool,
    /// Custom metrics path (default: /metrics)
    pub path: Option<String>,
    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:9090".to_string(),
            enabled: cfg!(feature = "metrics"), // Only enabled if feature is available
            path: Some("/metrics".to_string()),
            timeout_secs: Some(30),
        }
    }
}

#[cfg(feature = "metrics")]
use {
    metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle},
    axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::get,
        Router,
        Server,
    },
};

pub struct MetricsServer {
    config: MetricsConfig,
    #[cfg(feature = "metrics")]
    handle: PrometheusHandle,
}

impl MetricsServer {
    pub fn new(config: MetricsConfig) -> Result<Self> {
        #[cfg(feature = "metrics")]
        {
            if !config.enabled {
                return Err(PlcError::Config("Metrics disabled in configuration".into()));
            }

            let builder = PrometheusBuilder::new();
            
            let handle = builder.install_recorder()
                .map_err(|e| PlcError::Config(format!("Failed to install metrics recorder: {}", e)))?;

            Ok(Self {
                config,
                handle,
            })
        }
        
        #[cfg(not(feature = "metrics"))]
        {
            if config.enabled {
                error!("Metrics enabled in config but 'metrics' feature not compiled");
                return Err(PlcError::Config("Metrics feature not enabled".into()));
            }
            Ok(Self { config })
        }
    }

    pub async fn run(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Metrics server disabled in configuration");
            return Ok(());
        }

        #[cfg(feature = "metrics")]
        {
            let metrics_path = self.config.path.as_deref().unwrap_or("/metrics");

            let app = Router::new()
                .route(metrics_path, get(metrics_handler))
                .with_state(self.handle.clone());

            let listener = TcpListener::bind(&self.config.bind_address).await
                .map_err(|e| PlcError::Config(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;
            let std_listener = listener.into_std()
                .map_err(|e| PlcError::WebServer(e.to_string()))?;

            info!("Metrics server listening on {} at {}", self.config.bind_address, metrics_path);

            Server::from_tcp(std_listener)
                .map_err(|e| PlcError::WebServer(e.to_string()))?
                .serve(app.into_make_service())
                .await
                .map_err(|e| PlcError::WebServer(e.to_string()))?;

            Ok(())
        }

        #[cfg(not(feature = "metrics"))]
        {
            error!("Metrics feature not enabled");
            Err(PlcError::Config("Metrics feature not enabled".into()))
        }
    }

    /// Get metrics server stats
    pub fn get_stats(&self) -> MetricsServerStats {
        MetricsServerStats {
            enabled: self.config.enabled,
            bind_address: self.config.bind_address.clone(),
            path: self.config.path.clone().unwrap_or_else(|| "/metrics".to_string()),
        }
    }

    /// Convenience constructor used by `main.rs`
    #[allow(dead_code)]
    pub fn new_with_addr(bind: &str, port: u16) -> Result<Self> {
        let cfg = MetricsConfig {
            bind_address: format!("{}:{}", bind, port),
            ..MetricsConfig::default()
        };
        Self::new(cfg)
    }

    #[allow(dead_code)]
    pub fn enable_runtime_metrics(&mut self) {}

    #[allow(dead_code)]
    pub async fn start(&self) -> Result<()> {
        self.run().await
    }

    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsServerStats {
    pub enabled: bool,
    pub bind_address: String,
    pub path: String,
}

#[cfg(feature = "metrics")]
async fn metrics_handler(
    State(handle): State<PrometheusHandle>,
) -> impl IntoResponse {
    // Fix: handle.render() returns a String, not a Result
    let metrics = handle.render();
    (StatusCode::OK, metrics)
}

// Helper function to validate metrics configuration
impl MetricsConfig {
    pub fn validate(&self) -> Result<()> {
        if self.enabled {
            // Validate bind address
            self.bind_address.parse::<SocketAddr>()
                .map_err(|e| PlcError::Config(format!("Invalid metrics bind address '{}': {}", self.bind_address, e)))?;
            
            // Validate path if provided
            if let Some(ref path) = self.path {
                if !path.starts_with('/') {
                    return Err(PlcError::Config(format!("Metrics path must start with '/': {}", path)));
                }
            }
            
            // Validate timeout
            if let Some(timeout) = self.timeout_secs {
                if timeout == 0 {
                    return Err(PlcError::Config("Metrics timeout cannot be 0".into()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert_eq!(config.bind_address, "0.0.0.0:9090");
        assert!(config.enabled);
        assert_eq!(config.path, Some("/metrics".to_string()));
        assert_eq!(config.timeout_secs, Some(30));
    }

    #[test]
    fn test_metrics_config_validation() {
        let mut config = MetricsConfig::default();
        assert!(config.validate().is_ok());

        config.bind_address = "invalid".to_string();
        assert!(config.validate().is_err());

        config.bind_address = "127.0.0.1:9090".to_string();
        config.path = Some("invalid".to_string());
        assert!(config.validate().is_err());

        config.path = Some("/valid".to_string());
        config.timeout_secs = Some(0);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_disabled_metrics() {
        let config = MetricsConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}

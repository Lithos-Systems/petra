//! Metrics server module for exposing Prometheus metrics

use axum::{routing::get, Router};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::error::Result;

/// Metrics server configuration
#[derive(Debug, Clone)]
pub struct MetricsServerConfig {
    /// Address to bind the metrics server to
    pub bind_address: SocketAddr,
    /// Optional custom buckets for histograms
    pub histogram_buckets: Option<Vec<f64>>,
}

impl Default for MetricsServerConfig {
    fn default() -> Self {
        Self {
            bind_address: ([127, 0, 0, 1], 9090).into(),
            histogram_buckets: None,
        }
    }
}

/// Metrics server that exposes Prometheus metrics
pub struct MetricsServer {
    config: MetricsServerConfig,
    handle: PrometheusHandle,
}

impl MetricsServer {
    /// Create a new metrics server
    pub fn new(config: MetricsServerConfig) -> Result<Self> {
        let builder = PrometheusBuilder::new();
        
        let builder = if let Some(buckets) = &config.histogram_buckets {
            builder.set_buckets_for_metric(
                Matcher::Full("petra_scan_duration_us".to_string()),
                buckets,
            )?
        } else {
            builder
        };
        
        let handle = builder.install_recorder()?;
        
        Ok(Self { config, handle })
    }
    
    /// Start the metrics server
    pub async fn start(self) -> Result<()> {
        let app = Router::new()
            .route("/metrics", get(move || async move {
                self.handle.render()
            }));
        
        let listener = TcpListener::bind(&self.config.bind_address).await?;
        info!("Metrics server listening on {}", self.config.bind_address);
        
        axum::serve(listener, app).await?;
        Ok(())
    }
}

/// Start a metrics server with default configuration
pub async fn start_metrics_server() -> Result<()> {
    let server = MetricsServer::new(MetricsServerConfig::default())?;
    server.start().await
}

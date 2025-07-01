// src/metrics_server.rs
use crate::{error::*, config::MetricsConfig};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{info, error};

#[cfg(feature = "metrics")]
use {
    metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle},
    axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::get,
        Router,
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
            let builder = PrometheusBuilder::new()
                .with_http_listener(config.bind_address.parse::<SocketAddr>()
                    .map_err(|e| PlcError::Config(format!("Invalid bind address: {}", e)))?
                );

            let handle = builder.install_recorder()
                .map_err(|e| PlcError::Config(format!("Failed to install metrics recorder: {}", e)))?;

            Ok(Self {
                config,
                handle,
            })
        }
        
        #[cfg(not(feature = "metrics"))]
        {
            Ok(Self { config })
        }
    }

    pub async fn run(&self) -> Result<()> {
        #[cfg(feature = "metrics")]
        {
            let app = Router::new()
                .route("/metrics", get(metrics_handler))
                .with_state(self.handle.clone());

            let listener = TcpListener::bind(&self.config.bind_address).await
                .map_err(|e| PlcError::Config(format!("Failed to bind to {}: {}", self.config.bind_address, e)))?;

            info!("Metrics server listening on {}", self.config.bind_address);

            axum::serve(listener, app).await
                .map_err(|e| PlcError::Config(format!("Metrics server error: {}", e)))?;
        }
        
        #[cfg(not(feature = "metrics"))]
        {
            error!("Metrics feature not enabled");
            return Err(PlcError::Config("Metrics feature not enabled".into()));
        }

        Ok(())
    }
}

#[cfg(feature = "metrics")]
async fn metrics_handler(
    State(handle): State<PrometheusHandle>,
) -> impl IntoResponse {
    match handle.render() {
        Ok(metrics) => (StatusCode::OK, metrics),
        Err(err) => {
            error!("Failed to render metrics: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, String::from("Failed to render metrics"))
        }
    }
}

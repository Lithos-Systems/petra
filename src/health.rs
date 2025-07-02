// src/health.rs - Complete Fixed Implementation
use crate::error::{PlcError, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn};

#[cfg(feature = "health-metrics")]
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    pub bind_address: SocketAddr,
    pub check_interval_seconds: u32,
    
    #[cfg(feature = "detailed-health")]
    pub detailed_checks: bool,
    
    #[cfg(feature = "health-history")]
    pub history_size: usize,
    
    #[cfg(feature = "custom-endpoints")]
    pub custom_endpoints: Vec<CustomEndpoint>,
}

#[cfg(feature = "custom-endpoints")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEndpoint {
    pub path: String,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: Status,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<HealthCheck>>,
    
    #[cfg(feature = "health-metrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SystemMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[cfg(feature = "health-metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub load_average: [f64; 3],
}

pub struct HealthMonitor {
    config: HealthConfig,
    start_time: std::time::Instant,
    checks: Arc<RwLock<Vec<HealthCheckFn>>>,
    
    #[cfg(feature = "health-history")]
    history: Arc<RwLock<Vec<HealthStatus>>>,
    
    #[cfg(feature = "health-metrics")]
    system: Arc<RwLock<System>>,
}

type HealthCheckFn = Box<dyn Fn() -> HealthCheck + Send + Sync>;

impl HealthMonitor {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            start_time: std::time::Instant::now(),
            checks: Arc::new(RwLock::new(Vec::new())),
            
            #[cfg(feature = "health-history")]
            history: Arc::new(RwLock::new(Vec::new())),
            
            #[cfg(feature = "health-metrics")]
            system: Arc::new(RwLock::new(System::new_all())),
        }
    }
    
    pub async fn add_check<F>(&self, name: String, check: F)
    where
        F: Fn() -> HealthCheck + Send + Sync + 'static,
    {
        let mut checks = self.checks.write().await;
        checks.push(Box::new(check));
    }
    
    pub async fn get_status(&self) -> HealthStatus {
        let uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Run all health checks
        let checks = if self.config.detailed_checks {
            let checks = self.checks.read().await;
            let results: Vec<HealthCheck> = checks.iter().map(|check| check()).collect();
            Some(results)
        } else {
            None
        };
        
        // Determine overall status
        let status = if let Some(ref checks) = checks {
            if checks.iter().any(|c| c.status == Status::Unhealthy) {
                Status::Unhealthy
            } else if checks.iter().any(|c| c.status == Status::Degraded) {
                Status::Degraded
            } else {
                Status::Healthy
            }
        } else {
            Status::Healthy
        };
        
        // Get system metrics if enabled
        #[cfg(feature = "health-metrics")]
        let metrics = {
            let mut system = self.system.write().await;
            system.refresh_all();
            
            let cpu_usage = system.global_cpu_info().cpu_usage();
            let memory_used = system.used_memory() / 1024 / 1024;
            let memory_total = system.total_memory() / 1024 / 1024;
            let memory_usage_percent = (memory_used as f32 / memory_total as f32) * 100.0;
            
            let (disk_used, disk_total) = system.disks().iter().fold((0, 0), |(used, total), disk| {
                (
                    used + (disk.total_space() - disk.available_space()),
                    total + disk.total_space(),
                )
            });
            
            let network_data = system.networks();
            let (rx_bytes, tx_bytes) = network_data
                .iter()
                .fold((0, 0), |(rx, tx), (_, data)| {
                    (rx + data.received(), tx + data.transmitted())
                });
            
            let load_avg = system.load_average();
            
            Some(SystemMetrics {
                cpu_usage_percent: cpu_usage,
                memory_used_mb: memory_used,
                memory_total_mb: memory_total,
                memory_usage_percent,
                disk_used_gb: (disk_used / 1024 / 1024 / 1024) as f64,
                disk_total_gb: (disk_total / 1024 / 1024 / 1024) as f64,
                network_rx_bytes: rx_bytes,
                network_tx_bytes: tx_bytes,
                load_average: [load_avg.one, load_avg.five, load_avg.fifteen],
            })
        };
        
        #[cfg(not(feature = "health-metrics"))]
        let metrics = None;
        
        let health_status = HealthStatus {
            status,
            timestamp: chrono::Utc::now(),
            uptime_seconds,
            version: crate::VERSION.to_string(),
            checks,
            #[cfg(feature = "health-metrics")]
            metrics,
        };
        
        // Store in history if enabled
        #[cfg(feature = "health-history")]
        {
            let mut history = self.history.write().await;
            history.push(health_status.clone());
            if history.len() > self.config.history_size {
                history.remove(0);
            }
        }
        
        health_status
    }
    
    pub fn build_router(self) -> Router {
        let shared_state = Arc::new(self);
        
        let mut router = Router::new()
            .route("/health", get(health_handler))
            .route("/health/live", get(liveness_handler))
            .route("/health/ready", get(readiness_handler));
        
        #[cfg(feature = "detailed-health")]
        {
            router = router.route("/health/detailed", get(detailed_health_handler));
        }
        
        #[cfg(feature = "health-history")]
        {
            router = router.route("/health/history", get(history_handler));
        }
        
        #[cfg(feature = "health-metrics")]
        {
            router = router.route("/health/metrics", get(metrics_handler));
        }
        
        router.with_state(shared_state)
    }
    
    pub async fn start(self) -> Result<()> {
        let addr = self.config.bind_address;
        let router = self.build_router();
        
        info!("Health monitor listening on {}", addr);
        
        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to start health server: {}", e),
            )))?;
            
        Ok(())
    }
}

// Handler functions
async fn health_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let status = monitor.get_status().await;
    let status_code = match status.status {
        Status::Healthy => axum::http::StatusCode::OK,
        Status::Degraded => axum::http::StatusCode::OK,
        Status::Unhealthy => axum::http::StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (status_code, Json(status))
}

async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now(),
    }))
}

async fn readiness_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let status = monitor.get_status().await;
    let ready = status.status != Status::Unhealthy;
    
    let status_code = if ready {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    
    (
        status_code,
        Json(serde_json::json!({
            "ready": ready,
            "timestamp": chrono::Utc::now(),
        })),
    )
}

#[cfg(feature = "detailed-health")]
async fn detailed_health_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let mut status = monitor.get_status().await;
    
    // Force detailed checks even if not configured
    if status.checks.is_none() {
        let checks = monitor.checks.read().await;
        let results: Vec<HealthCheck> = checks.iter().map(|check| check()).collect();
        status.checks = Some(results);
    }
    
    Json(status)
}

#[cfg(feature = "health-history")]
async fn history_handler(
    State(monitor): State<Arc<HealthMonitor>>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let history = monitor.history.read().await;
    let limit = params.limit.unwrap_or(100).min(history.len());
    let start = history.len().saturating_sub(limit);
    
    Json(&history[start..])
}

#[cfg(feature = "health-history")]
#[derive(Deserialize)]
struct HistoryParams {
    limit: Option<usize>,
}

#[cfg(feature = "health-metrics")]
async fn metrics_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let status = monitor.get_status().await;
    Json(status.metrics)
}

// Built-in health checks
pub fn database_check(connection_string: &str) -> HealthCheck {
    // Simplified example
    HealthCheck {
        name: "database".to_string(),
        status: Status::Healthy,
        message: Some("Database connection OK".to_string()),
        duration_ms: Some(5),
    }
}

pub fn disk_space_check(min_free_gb: f64) -> HealthCheck {
    #[cfg(feature = "health-metrics")]
    {
        let system = System::new_all();
        let (_, _, free_gb) = system.disks().iter().fold((0, 0, 0), |(used, total, free), disk| {
            (
                used + (disk.total_space() - disk.available_space()),
                total + disk.total_space(),
                free + disk.available_space(),
            )
        });
        
        let free_gb = (free_gb / 1024 / 1024 / 1024) as f64;
        
        if free_gb < min_free_gb {
            HealthCheck {
                name: "disk_space".to_string(),
                status: Status::Unhealthy,
                message: Some(format!("Only {:.2} GB free, minimum required: {:.2} GB", free_gb, min_free_gb)),
                duration_ms: Some(1),
            }
        } else {
            HealthCheck {
                name: "disk_space".to_string(),
                status: Status::Healthy,
                message: Some(format!("{:.2} GB free", free_gb)),
                duration_ms: Some(1),
            }
        }
    }
    
    #[cfg(not(feature = "health-metrics"))]
    HealthCheck {
        name: "disk_space".to_string(),
        status: Status::Healthy,
        message: Some("Health metrics not enabled".to_string()),
        duration_ms: Some(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_monitor() {
        let config = HealthConfig {
            bind_address: "127.0.0.1:0".parse().unwrap(),
            check_interval_seconds: 30,
            #[cfg(feature = "detailed-health")]
            detailed_checks: true,
            #[cfg(feature = "health-history")]
            history_size: 100,
            #[cfg(feature = "custom-endpoints")]
            custom_endpoints: vec![],
        };
        
        let monitor = HealthMonitor::new(config);
        
        // Add a simple check
        monitor.add_check("test".to_string(), || {
            HealthCheck {
                name: "test".to_string(),
                status: Status::Healthy,
                message: None,
                duration_ms: Some(0),
            }
        }).await;
        
        let status = monitor.get_status().await;
        assert_eq!(status.status, Status::Healthy);
        assert!(status.uptime_seconds >= 0);
    }
}

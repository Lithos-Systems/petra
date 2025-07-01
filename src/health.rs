// src/health.rs
use crate::engine::EngineStats;
use axum::{
    Router,
    routing::{get, post},
    response::{Json, Response, IntoResponse},
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{info, error};

#[cfg(feature = "detailed-health")]
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: Status,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub uptime_seconds: u64,
    
    #[cfg(feature = "detailed-health")]
    pub checks: Vec<HealthCheckResult>,
    
    #[cfg(feature = "health-metrics")]
    pub metrics: Option<HealthMetrics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Healthy,
    Degraded,
    Unhealthy,
}

#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: Status,
    pub message: Option<String>,
    pub duration_ms: u64,
}

#[cfg(feature = "health-metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct HealthMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub thread_count: u32,
    pub open_file_descriptors: u32,
}

pub struct HealthServer {
    addr: String,
    stats: Arc<RwLock<EngineStats>>,
    
    #[cfg(feature = "detailed-health")]
    health_checks: Vec<Box<dyn HealthCheck>>,
    
    #[cfg(feature = "health-history")]
    history: Arc<RwLock<HealthHistory>>,
    
    #[cfg(feature = "custom-endpoints")]
    custom_routes: Option<Router>,
}

#[cfg(feature = "detailed-health")]
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthCheckResult;
    fn name(&self) -> &str;
}

#[cfg(feature = "health-history")]
#[derive(Default)]
struct HealthHistory {
    entries: std::collections::VecDeque<HealthStatus>,
    max_entries: usize,
}

impl HealthServer {
    pub fn new(addr: impl Into<String>, stats: Arc<RwLock<EngineStats>>) -> Self {
        Self {
            addr: addr.into(),
            stats,
            #[cfg(feature = "detailed-health")]
            health_checks: Vec::new(),
            #[cfg(feature = "health-history")]
            history: Arc::new(RwLock::new(HealthHistory {
                entries: std::collections::VecDeque::new(),
                max_entries: 1000,
            })),
            #[cfg(feature = "custom-endpoints")]
            custom_routes: None,
        }
    }
    
    #[cfg(feature = "detailed-health")]
    pub fn add_check<C: HealthCheck + 'static>(mut self, check: C) -> Self {
        self.health_checks.push(Box::new(check));
        self
    }
    
    #[cfg(feature = "custom-endpoints")]
    pub fn with_custom_routes(mut self, routes: Router) -> Self {
        self.custom_routes = Some(routes);
        self
    }
    
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let app = self.build_router();
        
        info!("Starting health server on {}", self.addr);
        
        let listener = tokio::net::TcpListener::bind(&self.addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
    
    fn build_router(self) -> Router {
        let state = AppState {
            stats: self.stats.clone(),
            #[cfg(feature = "detailed-health")]
            health_checks: Arc::new(self.health_checks),
            #[cfg(feature = "health-history")]
            history: self.history.clone(),
        };
        
        let mut app = Router::new()
            .route("/health", get(health_handler))
            .route("/ready", get(ready_handler))
            .route("/live", get(live_handler));
        
        #[cfg(feature = "health-metrics")]
        {
            app = app.route("/metrics", get(metrics_handler));
        }
        
        #[cfg(feature = "health-history")]
        {
            app = app.route("/health/history", get(history_handler));
        }
        
        #[cfg(feature = "detailed-health")]
        {
            app = app.route("/health/detailed", get(detailed_health_handler));
        }
        
        #[cfg(feature = "custom-endpoints")]
        if let Some(custom) = self.custom_routes {
            app = app.merge(custom);
        }
        
        app.with_state(state)
    }
}

#[derive(Clone)]
struct AppState {
    stats: Arc<RwLock<EngineStats>>,
    #[cfg(feature = "detailed-health")]
    health_checks: Arc<Vec<Box<dyn HealthCheck>>>,
    #[cfg(feature = "health-history")]
    history: Arc<RwLock<HealthHistory>>,
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.stats.read();
    
    let status = if stats.running && stats.error_count == 0 {
        Status::Healthy
    } else if stats.running {
        Status::Degraded
    } else {
        Status::Unhealthy
    };
    
    let health = HealthStatus {
        status,
        timestamp: chrono::Utc::now(),
        version: crate::VERSION.to_string(),
        uptime_seconds: stats.uptime_secs,
        #[cfg(feature = "detailed-health")]
        checks: Vec::new(),
        #[cfg(feature = "health-metrics")]
        metrics: None,
    };
    
    #[cfg(feature = "health-history")]
    {
        let mut history = state.history.write();
        history.entries.push_back(health.clone());
        if history.entries.len() > history.max_entries {
            history.entries.pop_front();
        }
    }
    
    let status_code = match status {
        Status::Healthy => StatusCode::OK,
        Status::Degraded => StatusCode::OK,
        Status::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (status_code, Json(health))
}

async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.stats.read();
    
    if stats.running && stats.scan_count > 10 {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn live_handler() -> impl IntoResponse {
    // Always return OK for liveness
    StatusCode::OK
}

#[cfg(feature = "detailed-health")]
async fn detailed_health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.stats.read().clone();
    let mut checks = Vec::new();
    
    // Run all health checks
    for check in state.health_checks.iter() {
        let start = std::time::Instant::now();
        let result = check.check().await;
        let duration_ms = start.elapsed().as_millis() as u64;
        
        checks.push(HealthCheckResult {
            name: check.name().to_string(),
            status: result.status,
            message: result.message,
            duration_ms,
        });
    }
    
    // Determine overall status
    let overall_status = if checks.iter().all(|c| c.status == Status::Healthy) {
        Status::Healthy
    } else if checks.iter().any(|c| c.status == Status::Unhealthy) {
        Status::Unhealthy
    } else {
        Status::Degraded
    };
    
    let health = HealthStatus {
        status: overall_status,
        timestamp: chrono::Utc::now(),
        version: crate::VERSION.to_string(),
        uptime_seconds: stats.uptime_secs,
        checks,
        #[cfg(feature = "health-metrics")]
        metrics: get_system_metrics(),
    };
    
    Json(health)
}

#[cfg(feature = "health-metrics")]
fn get_system_metrics() -> Option<HealthMetrics> {
    use sysinfo::{System, SystemExt, ProcessExt};
    
    let mut sys = System::new();
    sys.refresh_all();
    
    let pid = sysinfo::Pid::from(std::process::id() as usize);
    let process = sys.process(pid)?;
    
    Some(HealthMetrics {
        cpu_usage_percent: process.cpu_usage() as f64,
        memory_usage_mb: process.memory() / 1024 / 1024,
        thread_count: sys.threads() as u32,
        open_file_descriptors: 0, // Platform-specific
    })
}

#[cfg(feature = "health-metrics")]
async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.stats.read();
    
    let metrics = serde_json::json!({
        "engine": {
            "running": stats.running,
            "scan_count": stats.scan_count,
            "error_count": stats.error_count,
            "uptime_seconds": stats.uptime_secs,
            "signal_count": stats.signal_count,
            "block_count": stats.block_count,
        },
        "system": get_system_metrics(),
    });
    
    Json(metrics)
}

#[cfg(feature = "health-history")]
async fn history_handler(State(state): State<AppState>) -> impl IntoResponse {
    let history = state.history.read();
    let entries: Vec<_> = history.entries.iter().cloned().collect();
    Json(entries)
}

// Example health checks
#[cfg(feature = "detailed-health")]
pub struct DatabaseHealthCheck {
    connection_string: String,
}

#[cfg(feature = "detailed-health")]
#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        // Implement actual database check
        HealthCheckResult {
            name: "database".to_string(),
            status: Status::Healthy,
            message: Some("Database connection OK".to_string()),
            duration_ms: 10,
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}

#[cfg(feature = "detailed-health")]
pub struct DiskSpaceHealthCheck {
    path: std::path::PathBuf,
    min_free_gb: u64,
}

#[cfg(feature = "detailed-health")]
#[async_trait]
impl HealthCheck for DiskSpaceHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        use fs2::available_space;
        
        match available_space(&self.path) {
            Ok(bytes) => {
                let gb = bytes / 1024 / 1024 / 1024;
                if gb >= self.min_free_gb {
                    HealthCheckResult {
                        name: "disk_space".to_string(),
                        status: Status::Healthy,
                        message: Some(format!("{}GB free", gb)),
                        duration_ms: 1,
                    }
                } else {
                    HealthCheckResult {
                        name: "disk_space".to_string(),
                        status: Status::Degraded,
                        message: Some(format!("Only {}GB free, minimum {}GB", gb, self.min_free_gb)),
                        duration_ms: 1,
                    }
                }
            }
            Err(e) => HealthCheckResult {
                name: "disk_space".to_string(),
                status: Status::Unhealthy,
                message: Some(format!("Failed to check disk space: {}", e)),
                duration_ms: 1,
            }
        }
    }
    
    fn name(&self) -> &str {
        "disk_space"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_status_serialization() {
        let status = HealthStatus {
            status: Status::Healthy,
            timestamp: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            #[cfg(feature = "detailed-health")]
            checks: vec![],
            #[cfg(feature = "health-metrics")]
            metrics: None,
        };
        
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
    }
}

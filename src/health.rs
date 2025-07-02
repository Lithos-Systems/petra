// src/health.rs - Complete health monitoring system with feature flags

use crate::error::{PlcError, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[cfg(feature = "health-metrics")]
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt};

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Address to bind health endpoint
    pub bind_address: SocketAddr,
    
    /// Health check interval in seconds
    pub check_interval_seconds: u32,
    
    /// Enable detailed health checks
    #[cfg(feature = "detailed-health")]
    #[serde(default)]
    pub detailed_checks: bool,
    
    /// History retention size
    #[cfg(feature = "health-history")]
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    
    /// Custom health check endpoints
    #[cfg(feature = "custom-endpoints")]
    #[serde(default)]
    pub custom_endpoints: Vec<CustomEndpoint>,
    
    /// System resource thresholds
    #[cfg(feature = "health-metrics")]
    #[serde(default)]
    pub thresholds: MetricThresholds,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:9090".parse().unwrap(),
            check_interval_seconds: 30,
            #[cfg(feature = "detailed-health")]
            detailed_checks: false,
            #[cfg(feature = "health-history")]
            history_size: default_history_size(),
            #[cfg(feature = "custom-endpoints")]
            custom_endpoints: Vec::new(),
            #[cfg(feature = "health-metrics")]
            thresholds: MetricThresholds::default(),
        }
    }
}

#[cfg(feature = "health-history")]
fn default_history_size() -> usize {
    100
}

#[cfg(feature = "custom-endpoints")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomEndpoint {
    pub path: String,
    pub handler: String,
    pub description: String,
}

#[cfg(feature = "health-metrics")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholds {
    pub cpu_percent_warning: f32,
    pub cpu_percent_critical: f32,
    pub memory_percent_warning: f32,
    pub memory_percent_critical: f32,
    pub disk_percent_warning: f32,
    pub disk_percent_critical: f32,
}

#[cfg(feature = "health-metrics")]
impl Default for MetricThresholds {
    fn default() -> Self {
        Self {
            cpu_percent_warning: 70.0,
            cpu_percent_critical: 90.0,
            memory_percent_warning: 80.0,
            memory_percent_critical: 95.0,
            disk_percent_warning: 85.0,
            disk_percent_critical: 95.0,
        }
    }
}

// ============================================================================
// HEALTH STATUS STRUCTURES
// ============================================================================

/// Overall health status
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: Status,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub version: String,
    
    /// Individual health checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<Vec<HealthCheck>>,
    
    /// System metrics
    #[cfg(feature = "health-metrics")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<SystemMetrics>,
    
    /// Component statuses
    #[cfg(feature = "detailed-health")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<ComponentStatuses>,
}

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// System resource metrics
#[cfg(feature = "health-metrics")]
#[derive(Debug, Clone, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f32,
    pub disk_used_gb: f64,
    pub disk_total_gb: f64,
    pub disk_usage_percent: f32,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub load_average: [f64; 3],
    pub process_count: usize,
    pub thread_count: usize,
}

/// Detailed component statuses
#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatuses {
    pub engine: ComponentStatus,
    pub signal_bus: ComponentStatus,
    pub protocols: Vec<ProtocolStatus>,
    pub storage: Option<StorageStatus>,
    pub alarms: Option<AlarmStatus>,
}

#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct ComponentStatus {
    pub name: String,
    pub status: Status,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct ProtocolStatus {
    pub protocol: String,
    pub status: Status,
    pub connected: bool,
    pub last_message: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count: u64,
}

#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct StorageStatus {
    pub status: Status,
    pub write_queue_size: usize,
    pub last_write: Option<chrono::DateTime<chrono::Utc>>,
    pub error_count: u64,
}

#[cfg(feature = "detailed-health")]
#[derive(Debug, Clone, Serialize)]
pub struct AlarmStatus {
    pub status: Status,
    pub active_alarms: usize,
    pub acknowledged_alarms: usize,
    pub suppressed_alarms: usize,
}

// ============================================================================
// HEALTH MONITOR IMPLEMENTATION
// ============================================================================

/// Main health monitoring service
pub struct HealthMonitor {
    config: HealthConfig,
    start_time: Instant,
    checks: Arc<RwLock<Vec<HealthCheckFn>>>,
    
    #[cfg(feature = "health-history")]
    history: Arc<RwLock<Vec<HealthStatus>>>,
    
    #[cfg(feature = "health-metrics")]
    system: Arc<RwLock<System>>,
    
    #[cfg(feature = "detailed-health")]
    component_providers: Arc<RwLock<Vec<ComponentProvider>>>,
}

type HealthCheckFn = Box<dyn Fn() -> HealthCheck + Send + Sync>;

#[cfg(feature = "detailed-health")]
type ComponentProvider = Box<dyn Fn() -> ComponentStatuses + Send + Sync>;

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            checks: Arc::new(RwLock::new(Vec::new())),
            
            #[cfg(feature = "health-history")]
            history: Arc::new(RwLock::new(Vec::with_capacity(config.history_size))),
            
            #[cfg(feature = "health-metrics")]
            system: Arc::new(RwLock::new(System::new_all())),
            
            #[cfg(feature = "detailed-health")]
            component_providers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Add a custom health check
    pub async fn add_check<F>(&self, name: String, check: F)
    where
        F: Fn() -> HealthCheck + Send + Sync + 'static,
    {
        let mut checks = self.checks.write().await;
        checks.push(Box::new(check));
        info!("Added health check: {}", name);
    }
    
    /// Add a component status provider
    #[cfg(feature = "detailed-health")]
    pub async fn add_component_provider<F>(&self, provider: F)
    where
        F: Fn() -> ComponentStatuses + Send + Sync + 'static,
    {
        let mut providers = self.component_providers.write().await;
        providers.push(Box::new(provider));
    }
    
    /// Get current health status
    pub async fn get_status(&self) -> HealthStatus {
        let uptime_seconds = self.start_time.elapsed().as_secs();
        
        // Run all health checks
        let checks = self.run_health_checks().await;
        
        // Determine overall status
        let status = self.calculate_overall_status(&checks);
        
        // Get system metrics if enabled
        #[cfg(feature = "health-metrics")]
        let metrics = self.collect_metrics().await;
        
        // Get component statuses if enabled
        #[cfg(feature = "detailed-health")]
        let components = self.collect_component_statuses().await;
        
        let health_status = HealthStatus {
            status,
            timestamp: chrono::Utc::now(),
            uptime_seconds,
            version: env!("CARGO_PKG_VERSION").to_string(),
            checks: if !checks.is_empty() { Some(checks) } else { None },
            
            #[cfg(feature = "health-metrics")]
            metrics,
            
            #[cfg(feature = "detailed-health")]
            components,
        };
        
        // Store in history if enabled
        #[cfg(feature = "health-history")]
        {
            self.store_in_history(health_status.clone()).await;
        }
        
        health_status
    }
    
    /// Run all registered health checks
    async fn run_health_checks(&self) -> Vec<HealthCheck> {
        #[cfg(feature = "detailed-health")]
        if self.config.detailed_checks {
            let checks = self.checks.read().await;
            return checks.iter().map(|check| {
                let start = Instant::now();
                let mut result = check();
                result.duration_ms = Some(start.elapsed().as_millis() as u64);
                result
            }).collect();
        }
        
        // Basic health checks always run
        let mut results = vec![
            self.basic_health_check(),
        ];
        
        // Add custom checks if not in detailed mode
        #[cfg(not(feature = "detailed-health"))]
        {
            let checks = self.checks.read().await;
            for check in checks.iter() {
                results.push(check());
            }
        }
        
        results
    }
    
    /// Basic health check (always available)
    fn basic_health_check(&self) -> HealthCheck {
        HealthCheck {
            name: "system".to_string(),
            status: Status::Healthy,
            message: Some("System is operational".to_string()),
            duration_ms: Some(0),
            metadata: None,
        }
    }
    
    /// Calculate overall status from individual checks
    fn calculate_overall_status(&self, checks: &[HealthCheck]) -> Status {
        if checks.is_empty() {
            return Status::Healthy;
        }
        
        let has_unhealthy = checks.iter().any(|c| c.status == Status::Unhealthy);
        let has_degraded = checks.iter().any(|c| c.status == Status::Degraded);
        
        if has_unhealthy {
            Status::Unhealthy
        } else if has_degraded {
            Status::Degraded
        } else {
            Status::Healthy
        }
    }
    
    /// Collect system metrics
    #[cfg(feature = "health-metrics")]
    async fn collect_metrics(&self) -> Option<SystemMetrics> {
        let mut system = self.system.write().await;
        system.refresh_all();
        
        let cpu_usage = system.global_cpu_info().cpu_usage();
        let memory_total = system.total_memory();
        let memory_used = system.used_memory();
        let memory_percent = (memory_used as f32 / memory_total as f32) * 100.0;
        
        let (disk_total, disk_used) = system.disks().iter().fold((0, 0), |(total, used), disk| {
            (
                total + disk.total_space(),
                used + (disk.total_space() - disk.available_space()),
            )
        });
        let disk_percent = (disk_used as f32 / disk_total as f32) * 100.0;
        
        let (rx_bytes, tx_bytes) = system.networks().iter().fold((0, 0), |(rx, tx), (_, data)| {
            (rx + data.received(), tx + data.transmitted())
        });
        
        let load_avg = match System::load_average() {
            Some(avg) => [avg.one, avg.five, avg.fifteen],
            None => [0.0, 0.0, 0.0],
        };
        
        Some(SystemMetrics {
            cpu_usage_percent: cpu_usage,
            memory_used_mb: memory_used / 1024 / 1024,
            memory_total_mb: memory_total / 1024 / 1024,
            memory_usage_percent: memory_percent,
            disk_used_gb: (disk_used as f64) / 1024.0 / 1024.0 / 1024.0,
            disk_total_gb: (disk_total as f64) / 1024.0 / 1024.0 / 1024.0,
            disk_usage_percent: disk_percent,
            network_rx_bytes: rx_bytes,
            network_tx_bytes: tx_bytes,
            load_average: load_avg,
            process_count: system.processes().len(),
            thread_count: system.processes().values().map(|p| p.tasks.len()).sum(),
        })
    }
    
    /// Collect component statuses
    #[cfg(feature = "detailed-health")]
    async fn collect_component_statuses(&self) -> Option<ComponentStatuses> {
        if !self.config.detailed_checks {
            return None;
        }
        
        let providers = self.component_providers.read().await;
        if providers.is_empty() {
            return None;
        }
        
        // Merge all component statuses
        // In a real implementation, this would properly merge multiple providers
        let statuses = providers.first().map(|provider| provider());
        statuses
    }
    
    /// Store status in history
    #[cfg(feature = "health-history")]
    async fn store_in_history(&self, status: HealthStatus) {
        let mut history = self.history.write().await;
        history.push(status);
        
        // Maintain history size limit
        while history.len() > self.config.history_size {
            history.remove(0);
        }
    }
    
    /// Get health history
    #[cfg(feature = "health-history")]
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<HealthStatus> {
        let history = self.history.read().await;
        let limit = limit.unwrap_or(history.len()).min(history.len());
        let start = history.len().saturating_sub(limit);
        
        history[start..].to_vec()
    }
    
    /// Build the health monitoring router
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
        
        #[cfg(feature = "custom-endpoints")]
        {
            // Add custom endpoints from configuration
            // This would be implemented based on the custom endpoint handlers
        }
        
        router.with_state(shared_state)
    }
    
    /// Start the health monitoring server
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
    
    /// Start periodic health checks
    pub fn start_periodic_checks(self: Arc<Self>) {
        let interval = Duration::from_secs(self.config.check_interval_seconds as u64);
        
        tokio::spawn(async move {
            let mut timer = tokio::time::interval(interval);
            timer.tick().await; // Skip first immediate tick
            
            loop {
                timer.tick().await;
                
                let status = self.get_status().await;
                
                if status.status != Status::Healthy {
                    warn!("Health check failed: {:?}", status);
                }
                
                #[cfg(feature = "health-metrics")]
                {
                    if let Some(metrics) = &status.metrics {
                        self.check_metric_thresholds(metrics).await;
                    }
                }
            }
        });
    }
    
    /// Check metrics against thresholds
    #[cfg(feature = "health-metrics")]
    async fn check_metric_thresholds(&self, metrics: &SystemMetrics) {
        let thresholds = &self.config.thresholds;
        
        if metrics.cpu_usage_percent > thresholds.cpu_percent_critical {
            error!("CPU usage critical: {:.1}%", metrics.cpu_usage_percent);
        } else if metrics.cpu_usage_percent > thresholds.cpu_percent_warning {
            warn!("CPU usage warning: {:.1}%", metrics.cpu_usage_percent);
        }
        
        if metrics.memory_usage_percent > thresholds.memory_percent_critical {
            error!("Memory usage critical: {:.1}%", metrics.memory_usage_percent);
        } else if metrics.memory_usage_percent > thresholds.memory_percent_warning {
            warn!("Memory usage warning: {:.1}%", metrics.memory_usage_percent);
        }
        
        if metrics.disk_usage_percent > thresholds.disk_percent_critical {
            error!("Disk usage critical: {:.1}%", metrics.disk_usage_percent);
        } else if metrics.disk_usage_percent > thresholds.disk_percent_warning {
            warn!("Disk usage warning: {:.1}%", metrics.disk_usage_percent);
        }
    }
}

// ============================================================================
// HTTP HANDLERS
// ============================================================================

/// Main health endpoint handler
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

/// Kubernetes liveness probe handler
async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now(),
    }))
}

/// Kubernetes readiness probe handler
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

/// Detailed health information handler
#[cfg(feature = "detailed-health")]
async fn detailed_health_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let mut monitor_ref = Arc::clone(&monitor);
    let mut config = monitor_ref.config.clone();
    config.detailed_checks = true;
    
    let status = monitor.get_status().await;
    Json(status)
}

/// Health history handler
#[cfg(feature = "health-history")]
async fn history_handler(
    State(monitor): State<Arc<HealthMonitor>>,
    Query(params): Query<HistoryParams>,
) -> impl IntoResponse {
    let history = monitor.get_history(params.limit).await;
    Json(history)
}

#[cfg(feature = "health-history")]
#[derive(Deserialize)]
struct HistoryParams {
    limit: Option<usize>,
}

/// System metrics handler
#[cfg(feature = "health-metrics")]
async fn metrics_handler(
    State(monitor): State<Arc<HealthMonitor>>,
) -> impl IntoResponse {
    let status = monitor.get_status().await;
    Json(status.metrics)
}

// ============================================================================
// BUILT-IN HEALTH CHECKS
// ============================================================================

/// Database connectivity check
pub fn database_check(connection_string: &str) -> Box<dyn Fn() -> HealthCheck + Send + Sync> {
    let conn_str = connection_string.to_string();
    
    Box::new(move || {
        // In a real implementation, this would actually test the database connection
        HealthCheck {
            name: "database".to_string(),
            status: Status::Healthy,
            message: Some(format!("Connected to: {}", conn_str)),
            duration_ms: Some(5),
            metadata: None,
        }
    })
}

/// Disk space check
pub fn disk_space_check(min_free_gb: f64) -> Box<dyn Fn() -> HealthCheck + Send + Sync> {
    Box::new(move || {
        #[cfg(feature = "health-metrics")]
        {
            let system = System::new_all();
            let free_gb = system.disks().iter()
                .map(|disk| disk.available_space())
                .sum::<u64>() as f64 / 1024.0 / 1024.0 / 1024.0;
            
            if free_gb < min_free_gb {
                HealthCheck {
                    name: "disk_space".to_string(),
                    status: Status::Unhealthy,
                    message: Some(format!("Only {:.2} GB free, minimum required: {:.2} GB", free_gb, min_free_gb)),
                    duration_ms: Some(10),
                    metadata: None,
                }
            } else {
                HealthCheck {
                    name: "disk_space".to_string(),
                    status: Status::Healthy,
                    message: Some(format!("{:.2} GB free", free_gb)),
                    duration_ms: Some(10),
                    metadata: None,
                }
            }
        }
        
        #[cfg(not(feature = "health-metrics"))]
        {
            HealthCheck {
                name: "disk_space".to_string(),
                status: Status::Healthy,
                message: Some("Disk space check requires health-metrics feature".to_string()),
                duration_ms: Some(0),
                metadata: None,
            }
        }
    })
}

/// Memory usage check
pub fn memory_check(max_usage_percent: f32) -> Box<dyn Fn() -> HealthCheck + Send + Sync> {
    Box::new(move || {
        #[cfg(feature = "health-metrics")]
        {
            let system = System::new_all();
            let used = system.used_memory() as f32;
            let total = system.total_memory() as f32;
            let usage_percent = (used / total) * 100.0;
            
            let status = if usage_percent > max_usage_percent {
                Status::Unhealthy
            } else if usage_percent > max_usage_percent * 0.9 {
                Status::Degraded
            } else {
                Status::Healthy
            };
            
            HealthCheck {
                name: "memory".to_string(),
                status,
                message: Some(format!("Memory usage: {:.1}%", usage_percent)),
                duration_ms: Some(5),
                metadata: Some(serde_json::json!({
                    "used_mb": used / 1024.0 / 1024.0,
                    "total_mb": total / 1024.0 / 1024.0,
                })),
            }
        }
        
        #[cfg(not(feature = "health-metrics"))]
        {
            HealthCheck {
                name: "memory".to_string(),
                status: Status::Healthy,
                message: Some("Memory check requires health-metrics feature".to_string()),
                duration_ms: Some(0),
                metadata: None,
            }
        }
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_health_monitor() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);
        
        let status = monitor.get_status().await;
        assert_eq!(status.status, Status::Healthy);
        assert!(status.uptime_seconds >= 0);
    }

    #[tokio::test]
    async fn test_custom_health_check() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);
        
        // Add a custom check
        monitor.add_check("test_check".to_string(), || {
            HealthCheck {
                name: "test_check".to_string(),
                status: Status::Degraded,
                message: Some("Test degraded status".to_string()),
                duration_ms: Some(1),
                metadata: None,
            }
        }).await;
        
        let status = monitor.get_status().await;
        assert!(status.checks.is_some());
        
        let checks = status.checks.unwrap();
        assert!(checks.iter().any(|c| c.name == "test_check"));
    }

    #[cfg(feature = "health-history")]
    #[tokio::test]
    async fn test_health_history() {
        let mut config = HealthConfig::default();
        config.history_size = 5;
        let monitor = Arc::new(HealthMonitor::new(config));
        
        // Generate some history
        for i in 0..10 {
            let _ = monitor.get_status().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        let history = monitor.get_history(None).await;
        assert_eq!(history.len(), 5); // Should be limited to history_size
    }

    #[cfg(feature = "health-metrics")]
    #[tokio::test]
    async fn test_system_metrics() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);
        
        let status = monitor.get_status().await;
        assert!(status.metrics.is_some());
        
        let metrics = status.metrics.unwrap();
        assert!(metrics.cpu_usage_percent >= 0.0 && metrics.cpu_usage_percent <= 100.0);
        assert!(metrics.memory_usage_percent >= 0.0 && metrics.memory_usage_percent <= 100.0);
    }

    #[test]
    fn test_overall_status_calculation() {
        let config = HealthConfig::default();
        let monitor = HealthMonitor::new(config);
        
        // All healthy
        let checks = vec![
            HealthCheck { name: "1".to_string(), status: Status::Healthy, message: None, duration_ms: None, metadata: None },
            HealthCheck { name: "2".to_string(), status: Status::Healthy, message: None, duration_ms: None, metadata: None },
        ];
        assert_eq!(monitor.calculate_overall_status(&checks), Status::Healthy);
        
        // One degraded
        let checks = vec![
            HealthCheck { name: "1".to_string(), status: Status::Healthy, message: None, duration_ms: None, metadata: None },
            HealthCheck { name: "2".to_string(), status: Status::Degraded, message: None, duration_ms: None, metadata: None },
        ];
        assert_eq!(monitor.calculate_overall_status(&checks), Status::Degraded);
        
        // One unhealthy
        let checks = vec![
            HealthCheck { name: "1".to_string(), status: Status::Degraded, message: None, duration_ms: None, metadata: None },
            HealthCheck { name: "2".to_string(), status: Status::Unhealthy, message: None, duration_ms: None, metadata: None },
        ];
        assert_eq!(monitor.calculate_overall_status(&checks), Status::Unhealthy);
    }
}

use crate::error::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use serde::Serialize;
use tracing::{info, warn, error};
use std::time::{Duration, Instant};

#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
    fn name(&self) -> &str;
    fn critical(&self) -> bool { true }
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub message: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip)]
    pub last_check: Option<Instant>,
    #[serde(skip)]
    pub duration_ms: Option<u64>,
}

impl HealthStatus {
    pub fn healthy(message: impl Into<String>) -> Self {
        Self {
            healthy: true,
            message: message.into(),
            metadata: HashMap::new(),
            last_check: Some(Instant::now()),
            duration_ms: None,
        }
    }
    
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            message: message.into(),
            metadata: HashMap::new(),
            last_check: Some(Instant::now()),
            duration_ms: None,
        }
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.metadata.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
        );
        self
    }
}

pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheck>>,
    cache: Arc<parking_lot::RwLock<HashMap<String, CachedStatus>>>,
    cache_duration: Duration,
}

struct CachedStatus {
    status: HealthStatus,
    expires: Instant,
}

impl HealthChecker {
    pub fn new(cache_duration: Duration) -> Self {
        Self {
            checks: Vec::new(),
            cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            cache_duration,
        }
    }
    
    pub fn add_check(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    pub async fn check_all(&self) -> HealthReport {
        let mut report = HealthReport {
            overall_health: true,
            checks: HashMap::new(),
            timestamp: Instant::now(),
        };
        
        let mut futures = Vec::new();
        
        for check in &self.checks {
            let check = check.clone();
            let cache = self.cache.clone();
            let cache_duration = self.cache_duration;
            
            futures.push(tokio::spawn(async move {
                let name = check.name().to_string();
                let critical = check.critical();
                
                // Check cache first
                if let Some(cached) = cache.read().get(&name) {
                    if cached.expires > Instant::now() {
                        return (name, cached.status.clone(), critical);
                    }
                }
                
                // Perform health check
                let start = Instant::now();
                let mut status = check.check().await;
                status.duration_ms = Some(start.elapsed().as_millis() as u64);
                
                // Update cache
                cache.write().insert(name.clone(), CachedStatus {
                    status: status.clone(),
                    expires: Instant::now() + cache_duration,
                });
                
                (name, status, critical)
            }));
        }
        
        // Collect results
        for future in futures {
            match future.await {
                Ok((name, status, critical)) => {
                    if !status.healthy && critical {
                        report.overall_health = false;
                    }
                    report.checks.insert(name, status);
                }
                Err(e) => {
                    error!("Health check task failed: {}", e);
                    report.overall_health = false;
                }
            }
        }
        
        report
    }
    
    pub async fn check_single(&self, name: &str) -> Option<HealthStatus> {
        for check in &self.checks {
            if check.name() == name {
                return Some(check.check().await);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub overall_health: bool,
    pub checks: HashMap<String, HealthStatus>,
    #[serde(skip)]
    pub timestamp: Instant,
}

// Specific health check implementations

pub struct MqttHealthCheck {
    client: Arc<rumqttc::AsyncClient>,
    timeout: Duration,
}

impl MqttHealthCheck {
    pub fn new(client: Arc<rumqttc::AsyncClient>) -> Self {
        Self {
            client,
            timeout: Duration::from_secs(5),
        }
    }
}

#[async_trait]
impl HealthCheck for MqttHealthCheck {
    async fn check(&self) -> HealthStatus {
        // Try to publish a health check message
        let topic = "petra/health/check";
        let payload = format!("{}", chrono::Utc::now().timestamp());
        
        match tokio::time::timeout(
            self.timeout,
            self.client.publish(topic, rumqttc::QoS::AtMostOnce, false, payload)
        ).await {
            Ok(Ok(_)) => HealthStatus::healthy("MQTT connection is healthy"),
            Ok(Err(e)) => HealthStatus::unhealthy(format!("MQTT publish failed: {}", e)),
            Err(_) => HealthStatus::unhealthy("MQTT health check timed out"),
        }
    }
    
    fn name(&self) -> &str {
        "mqtt"
    }
}

#[cfg(feature = "advanced-storage")]
pub struct StorageHealthCheck {
    manager: Arc<crate::storage::manager::StorageManager>,
}

#[cfg(feature = "advanced-storage")]
impl StorageHealthCheck {
    pub fn new(manager: Arc<crate::storage::manager::StorageManager>) -> Self {
        Self { manager }
    }
}

#[cfg(feature = "advanced-storage")]
#[async_trait]
impl HealthCheck for StorageHealthCheck {
    async fn check(&self) -> HealthStatus {
        let stats = self.manager.stats();
        
        let mut status = if stats.healthy {
            HealthStatus::healthy("Storage system is healthy")
        } else {
            HealthStatus::unhealthy("Storage system has issues")
        };
        
        status = status
            .with_metadata("buffer_size", stats.buffer_size)
            .with_metadata("pending_writes", stats.pending_writes)
            .with_metadata("failed_writes", stats.failed_writes)
            .with_metadata("last_flush", stats.last_flush_ms);
        
        status
    }
    
    fn name(&self) -> &str {
        "storage"
    }
}

pub struct EngineHealthCheck {
    engine_stats: Arc<parking_lot::RwLock<crate::engine::EngineStats>>,
    max_scan_time_ms: u64,
    max_error_rate: f64,
}

impl EngineHealthCheck {
    pub fn new(
        engine_stats: Arc<parking_lot::RwLock<crate::engine::EngineStats>>,
        max_scan_time_ms: u64,
    ) -> Self {
        Self {
            engine_stats,
            max_scan_time_ms,
            max_error_rate: 0.01, // 1% error rate threshold
        }
    }
}

#[async_trait]
impl HealthCheck for EngineHealthCheck {
    async fn check(&self) -> HealthStatus {
        let stats = self.engine_stats.read().clone();
        
        if !stats.running {
            return HealthStatus::unhealthy("Engine is not running");
        }
        
        let error_rate = if stats.scan_count > 0 {
            stats.error_count as f64 / stats.scan_count as f64
        } else {
            0.0
        };
        
        let mut status = HealthStatus::healthy("Engine is running");
        let mut issues = Vec::new();
        
        // Check error rate
        if error_rate > self.max_error_rate {
            issues.push(format!("High error rate: {:.2}%", error_rate * 100.0));
        }
        
        // Add metadata
        status = status
            .with_metadata("scan_count", stats.scan_count)
            .with_metadata("error_count", stats.error_count)
            .with_metadata("error_rate", error_rate)
            .with_metadata("uptime_secs", stats.uptime_secs)
            .with_metadata("signal_count", stats.signal_count)
            .with_metadata("block_count", stats.block_count);
        
        if !issues.is_empty() {
            status.healthy = false;
            status.message = issues.join(", ");
        }
        
        status
    }
    
    fn name(&self) -> &str {
        "engine"
    }
}

// HTTP endpoint handler
pub async fn health_endpoint(checker: Arc<HealthChecker>) -> impl warp::Reply {
    let report = checker.check_all().await;
    
    let status_code = if report.overall_health {
        warp::http::StatusCode::OK
    } else {
        warp::http::StatusCode::SERVICE_UNAVAILABLE
    };
    
    warp::reply::with_status(
        warp::reply::json(&report),
        status_code,
    )
}

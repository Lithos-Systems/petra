// Add to src/health.rs
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: HealthLevel,
    pub checks: HashMap<String, CheckResult>,
    pub timestamp: DateTime<Utc>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
}

pub async fn health_check(&self) -> HealthStatus {
    let mut checks = HashMap::new();
    
    // Check signal bus
    checks.insert("signal_bus".to_string(), 
        CheckResult::ok_with_metric("signals", self.bus.signal_count()));
    
    // Check storage
    if let Some(storage) = &self.storage {
        checks.insert("storage".to_string(), 
            storage.health_check().await.into());
    }
    
    // Check MQTT connectivity
    // Check S7 connectivity
    // Check memory usage
    
    HealthStatus {
        status: determine_overall_health(&checks),
        checks,
        timestamp: Utc::now(),
        uptime_seconds: self.start_time.elapsed().as_secs(),
    }
}

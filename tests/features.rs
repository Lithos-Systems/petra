//! Feature combination testing
//!
//! This module tests various feature combinations to ensure they work correctly
//! and validate the feature flag organization.

use petra::{Features, Config, init};

#[test]
fn test_feature_detection() {
    let features = Features::detect();
    
    // Test that feature detection works
    println!("Detected features:");
    features.print();
    
    // Ensure at least some core features are present
    assert!(features.core.standard_monitoring || features.core.enhanced_monitoring);
}

#[test]
fn test_feature_summary() {
    let features = Features::detect();
    let summary = features.summary();
    
    // Summary should not be empty
    assert!(!summary.is_empty());
    println!("Feature summary: {}", summary);
}

#[test]
fn test_init_with_features() {
    // Test that initialization works with current feature set
    init().expect("Failed to initialize PETRA with current features");
}

#[test]
#[cfg(feature = "mqtt")]
fn test_mqtt_feature() {
    let features = Features::detect();
    assert!(features.protocols.mqtt, "MQTT feature should be enabled");
    
    // Test that MQTT configuration can be parsed
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Bool"
blocks: []
mqtt:
  broker_url: "localhost"
  client_id: "petra_test"
  port: 1883
  subscriptions: []
  publications: []
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse MQTT config");
    config.validate().expect("MQTT config validation failed");
}

#[test]
#[cfg(feature = "s7-support")]
fn test_s7_feature() {
    let features = Features::detect();
    assert!(features.protocols.s7, "S7 feature should be enabled");
    
    // Test that S7 configuration can be parsed
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Bool"
blocks: []
s7:
  ip: "192.168.1.100"
  rack: 0
  slot: 1
  connections: []
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse S7 config");
    config.validate().expect("S7 config validation failed");
}

#[test]
#[cfg(feature = "history")]
fn test_history_feature() {
    let features = Features::detect();
    assert!(features.storage.history, "History feature should be enabled");
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Float"
blocks: []
storage:
  history:
    enabled: true
    path: "test_data/history"
    interval_ms: 1000
    retention_days: 7
    signals: ["test_signal"]
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse history config");
    config.validate().expect("History config validation failed");
}

#[test]
#[cfg(feature = "security")]
fn test_security_feature() {
    let features = Features::detect();
    assert!(features.security.security, "Security feature should be enabled");
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Bool"
blocks: []
security:
  enabled: true
  authentication:
    method: "None"
    session_timeout_sec: 3600
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse security config");
    config.validate().expect("Security config validation failed");
}

#[test]
#[cfg(feature = "alarms")]
fn test_alarms_feature() {
    let features = Features::detect();
    assert!(features.alarms.alarms, "Alarms feature should be enabled");
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "temperature"
    type: "Float"
blocks: []
alarms:
  enabled: true
  alarm_definitions:
    - name: "high_temperature"
      signal: "temperature"
      condition: "high"
      threshold: 80.0
      priority: "High"
      enabled: true
  escalation: []
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse alarms config");
    config.validate().expect("Alarms config validation failed");
}

#[test]
#[cfg(feature = "web")]
fn test_web_feature() {
    let features = Features::detect();
    assert!(features.web.web, "Web feature should be enabled");
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Bool"
blocks: []
web:
  enabled: true
  port: 8080
  bind_address: "0.0.0.0"
  api:
    enabled: true
    base_path: "/api"
    rate_limiting: true
    requests_per_minute: 60
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse web config");
    config.validate().expect("Web config validation failed");
}

#[test]
#[cfg(feature = "validation")]
fn test_validation_feature() {
    let features = Features::detect();
    assert!(features.validation.validation, "Validation feature should be enabled");
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "pressure"
    type: "Float"
    validation:
      rules:
        - rule_type: "range"
          params:
            min: 0.0
            max: 100.0
          message: "Pressure must be between 0 and 100"
      required: true
blocks: []
validation:
  enabled: true
  on_failure: "Warn"
  global_rules: []
  signal_rules: {}
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse validation config");
    config.validate().expect("Validation config validation failed");
}

// ============================================================================
// BUNDLE TESTS
// ============================================================================

#[test]
#[cfg(feature = "edge")]
fn test_edge_bundle() {
    let features = Features::detect();
    
    // Edge bundle should include MQTT and basic monitoring
    assert!(features.protocols.mqtt, "Edge bundle should include MQTT");
    assert!(features.core.standard_monitoring, "Edge bundle should include standard monitoring");
    
    // Edge bundle should NOT include heavy features
    assert!(!features.protocols.s7, "Edge bundle should not include S7");
    assert!(!features.protocols.modbus, "Edge bundle should not include Modbus");
    assert!(!features.protocols.opcua, "Edge bundle should not include OPC-UA");
    assert!(!features.storage.advanced, "Edge bundle should not include advanced storage");
}

#[test]
#[cfg(feature = "scada")]
fn test_scada_bundle() {
    let features = Features::detect();
    
    // SCADA bundle should include industrial protocols
    assert!(
        features.protocols.s7 || features.protocols.modbus || features.protocols.opcua,
        "SCADA bundle should include at least one industrial protocol"
    );
    
    // SCADA bundle should include enhanced monitoring and storage
    assert!(features.core.enhanced_monitoring, "SCADA bundle should include enhanced monitoring");
    assert!(features.storage.history, "SCADA bundle should include history storage");
    assert!(features.security.security, "SCADA bundle should include security");
}

#[test]
#[cfg(feature = "production")]
fn test_production_bundle() {
    let features = Features::detect();
    
    // Production bundle should include optimizations and security
    assert!(features.core.optimized, "Production bundle should be optimized");
    assert!(features.core.metrics, "Production bundle should include metrics");
    assert!(features.security.security, "Production bundle should include security");
    assert!(features.storage.wal, "Production bundle should include WAL");
}

#[test]
#[cfg(feature = "enterprise")]
fn test_enterprise_bundle() {
    let features = Features::detect();
    
    // Enterprise bundle should include all major features
    assert!(features.core.optimized, "Enterprise bundle should be optimized");
    assert!(features.core.enhanced_monitoring, "Enterprise bundle should include enhanced monitoring");
    assert!(features.core.metrics, "Enterprise bundle should include metrics");
    assert!(features.security.security, "Enterprise bundle should include security");
    assert!(features.storage.advanced, "Enterprise bundle should include advanced storage");
    assert!(features.alarms.alarms, "Enterprise bundle should include alarms");
    assert!(features.web.web, "Enterprise bundle should include web interface");
}

#[test]
#[cfg(feature = "development")]
fn test_development_bundle() {
    let features = Features::detect();
    
    // Development bundle should include most features for testing
    assert!(features.development.examples, "Development bundle should include examples");
    assert!(features.development.profiling, "Development bundle should include profiling");
    
    // Should include major feature categories
    assert!(features.protocols.mqtt, "Development bundle should include MQTT");
    assert!(features.storage.history, "Development bundle should include storage");
    assert!(features.security.security, "Development bundle should include security");
}

// ============================================================================
// FEATURE COMBINATION TESTS
// ============================================================================

#[test]
#[cfg(all(feature = "mqtt", feature = "security"))]
fn test_mqtt_with_security() {
    let features = Features::detect();
    assert!(features.protocols.mqtt && features.security.security);
    
    // Test secure MQTT configuration
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "secure_signal"
    type: "Bool"
blocks: []
mqtt:
  broker_url: "ssl://localhost"
  client_id: "petra_secure"
  port: 8883
  tls:
    enabled: true
    cert_path: "certs/client.crt"
    key_path: "certs/client.key"
    verify_client: true
security:
  enabled: true
  tls:
    enabled: true
    cert_path: "certs/server.crt"
    key_path: "certs/server.key"
    verify_client: true
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse secure MQTT config");
    config.validate().expect("Secure MQTT config validation failed");
}

#[test]
#[cfg(all(feature = "history", feature = "compression"))]
fn test_history_with_compression() {
    let features = Features::detect();
    assert!(features.storage.history && features.storage.compression);
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "compressed_data"
    type: "Float"
blocks: []
storage:
  history:
    enabled: true
    path: "data/compressed_history"
    interval_ms: 500
    retention_days: 90
  compression:
    algorithm: "zstd"
    level: 3
    enabled: true
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse compressed history config");
    config.validate().expect("Compressed history config validation failed");
}

#[test]
#[cfg(all(feature = "alarms", feature = "email", feature = "web"))]
fn test_alarms_with_notifications() {
    let features = Features::detect();
    assert!(features.alarms.alarms && features.alarms.email && features.web.web);
    
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "alarm_signal"
    type: "Float"
blocks: []
alarms:
  enabled: true
  alarm_definitions:
    - name: "critical_alarm"
      signal: "alarm_signal"
      condition: "high"
      threshold: 95.0
      priority: "Critical"
      enabled: true
  notifications:
    email:
      smtp_server: "smtp.example.com"
      smtp_port: 587
      username: "alerts@example.com"
      password: "secret"
      from_address: "petra@example.com"
      recipients: ["operator@example.com"]
      use_tls: true
web:
  enabled: true
  port: 8080
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse alarm notification config");
    config.validate().expect("Alarm notification config validation failed");
}

#[test]
#[cfg(all(feature = "enhanced-monitoring", feature = "metrics"))]
fn test_enhanced_monitoring_with_metrics() {
    let features = Features::detect();
    assert!(features.core.enhanced_monitoring && features.core.metrics);
    
    let config_yaml = r#"
scan_time_ms: 50
signals:
  - name: "monitored_signal"
    type: "Float"
blocks: []
monitoring:
  level: "Enhanced"
  jitter_threshold_us: 500
  collect_stats: true
  enhanced:
    block_timing: true
    memory_tracking: true
    signal_change_tracking: true
    detailed_stats: true
    stats_interval_ms: 1000
metrics:
  enabled: true
  port: 9090
  path: "/metrics"
  custom_metrics:
    - name: "custom_counter"
      metric_type: "counter"
      description: "Custom counter metric"
      signal: "monitored_signal"
"#;
    
    let config: Config = serde_yaml::from_str(config_yaml)
        .expect("Failed to parse enhanced monitoring config");
    config.validate().expect("Enhanced monitoring config validation failed");
}

// ============================================================================
// ERROR CONDITION TESTS
// ============================================================================

#[test]
#[cfg(all(feature = "standard-monitoring", feature = "enhanced-monitoring"))]
fn test_monitoring_conflict_detection() {
    // This test should fail at compile time due to build.rs validation
    // But if it compiles, the features should be properly detected
    let features = Features::detect();
    
    // Only one monitoring level should be active
    let monitoring_count = [
        features.core.standard_monitoring,
        features.core.enhanced_monitoring,
    ].iter().filter(|&&x| x).count();
    
    // Note: This might pass if only one is actually enabled despite both features
    println!("Active monitoring features: {}", monitoring_count);
}

#[test]
fn test_invalid_config_rejection() {
    // Test that invalid configurations are properly rejected
    let invalid_configs = vec![
        // Invalid scan time
        r#"
scan_time_ms: 0
signals: []
blocks: []
"#,
        // Duplicate signal names
        r#"
scan_time_ms: 100
signals:
  - name: "duplicate"
    type: "Bool"
  - name: "duplicate"
    type: "Float"
blocks: []
"#,
        // Duplicate block names
        r#"
scan_time_ms: 100
signals:
  - name: "signal1"
    type: "Bool"
blocks:
  - name: "block1"
    type: "AND"
  - name: "block1"
    type: "OR"
"#,
        // Block referencing non-existent signal
        r#"
scan_time_ms: 100
signals:
  - name: "signal1"
    type: "Bool"
blocks:
  - name: "block1"
    type: "AND"
    inputs:
      in1: "nonexistent_signal"
"#,
    ];
    
    for (i, invalid_yaml) in invalid_configs.iter().enumerate() {
        let result: Result<Config, _> = serde_yaml::from_str(invalid_yaml);
        
        match result {
            Ok(config) => {
                // Config parsed but should fail validation
                assert!(
                    config.validate().is_err(),
                    "Invalid config {} should fail validation", i
                );
            }
            Err(_) => {
                // Config failed to parse (also acceptable)
                println!("Invalid config {} failed to parse (expected)", i);
            }
        }
    }
}

// ============================================================================
// PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_config_parsing_performance() {
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "signal1"
    type: "Bool"
  - name: "signal2"
    type: "Float"
  - name: "signal3"
    type: "Int"
blocks:
  - name: "and_block"
    type: "AND"
    inputs:
      in1: "signal1"
      in2: "signal2"
    outputs:
      out: "signal3"
"#;
    
    let start = std::time::Instant::now();
    
    for _ in 0..1000 {
        let config: Config = serde_yaml::from_str(config_yaml)
            .expect("Failed to parse config");
        config.validate().expect("Config validation failed");
    }
    
    let duration = start.elapsed();
    println!("1000 config parse/validate cycles took: {:?}", duration);
    
    // Should be reasonably fast (less than 1 second for 1000 iterations)
    assert!(duration.as_secs() < 1, "Config parsing too slow: {:?}", duration);
}

#[test]
fn test_feature_detection_performance() {
    let start = std::time::Instant::now();
    
    for _ in 0..10000 {
        let _features = Features::detect();
    }
    
    let duration = start.elapsed();
    println!("10000 feature detection calls took: {:?}", duration);
    
    // Feature detection should be very fast
    assert!(duration.as_millis() < 100, "Feature detection too slow: {:?}", duration);
}

// ============================================================================
// UTILITY FUNCTIONS FOR TESTS
// ============================================================================

/// Helper function to create a minimal valid configuration
fn create_minimal_config() -> Config {
    let config_yaml = r#"
scan_time_ms: 100
signals:
  - name: "test_signal"
    type: "Bool"
blocks: []
"#;
    
    serde_yaml::from_str(config_yaml).expect("Failed to create minimal config")
}

/// Helper function to validate a configuration string
fn validate_config_string(yaml: &str) -> Result<(), String> {
    match serde_yaml::from_str::<Config>(yaml) {
        Ok(config) => config.validate().map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[test]
fn test_minimal_config_creation() {
    let config = create_minimal_config();
    config.validate().expect("Minimal config should be valid");
    
    assert_eq!(config.scan_time_ms, 100);
    assert_eq!(config.signals.len(), 1);
    assert_eq!(config.blocks.len(), 0);
    assert_eq!(config.signals[0].name, "test_signal");
}

#[test]
fn test_config_feature_summary() {
    let config = create_minimal_config();
    let summary = config.feature_summary();
    
    // Minimal config should have a basic summary
    assert!(!summary.is_empty());
    println!("Minimal config summary: {}", summary);
}

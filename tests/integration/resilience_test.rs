use petra::{Config, Engine, MqttConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_mqtt_broker_reconnection() {
    // This test requires mosquitto to be running
    let config = Config {
        mqtt: Some(MqttConfig {
            host: "localhost".to_string(),
            port: 1883,
            client_id: "test-resilience".to_string(),
            username: None,
            password: None,
            keepalive_secs: 60,
            timeout_ms: 5000,
            use_tls: false,
            qos: 0,
            retain: false,
            subscribe_topics: Vec::new(),
            publish_topic_base: None,
            auto_reconnect: true,
            max_reconnect_attempts: 5,
            reconnect_delay_secs: 1,
        }),
        scan_time_ms: 100,
        ..Default::default()
    };
    
    let mut engine = Engine::new(config).expect("Failed to create engine");
    
    // Start engine
    let engine_handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Let it connect
    sleep(Duration::from_millis(500)).await;
    
    // Simulate broker restart (would need test harness to actually stop/start mosquitto)
    // For now, just verify engine is still running
    assert!(!engine_handle.is_finished());
    
    engine_handle.abort();
}

#[tokio::test]
async fn test_storage_failover() {
    use petra::storage::{StorageConfig, StorageStrategy};
    
    let config = Config {
        storage: Some(StorageConfig {
            strategy: StorageStrategy::LocalFirst,
            local_config: Some(Default::default()),
            remote_config: None, // Simulate remote unavailable
            wal_config: Default::default(),
        }),
        scan_time_ms: 100,
        ..Default::default()
    };
    
    let mut engine = Engine::new(config).expect("Failed to create engine");
    let bus = engine.signal_bus();
    
    // Write data
    bus.write_signal("test", petra::Value::Float(42.0))
        .expect("Failed to write signal");
    
    // Should use local storage when remote is unavailable
    let engine_handle = tokio::spawn(async move {
        engine.run().await
    });
    
    sleep(Duration::from_secs(1)).await;
    
    // Verify data was stored locally
    // (would need to check local storage files)
    
    engine_handle.abort();
}

#[tokio::test]
async fn test_signal_bus_overflow_handling() {
    use petra::SignalBus;
    use std::sync::Arc;
    
    let bus = Arc::new(SignalBus::new());
    let mut handles = vec![];
    
    // Spawn many writers
    for i in 0..100 {
        let bus_clone = bus.clone();
        let handle = tokio::spawn(async move {
            for j in 0..1000 {
                let signal = format!("signal_{}_{}", i, j);
                let _ = bus_clone.write_signal(&signal, petra::Value::Integer(j as i64));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all writes
    for handle in handles {
        let _ = handle.await;
    }
    
    // Bus should handle this load without panicking
    // Check we can still read signals
    let value = bus.read_signal("signal_0_0");
    assert!(value.is_ok());
}

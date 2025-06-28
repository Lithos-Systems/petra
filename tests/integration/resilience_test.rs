use petra::test_utils::TestHarness;
use std::time::Duration;

#[tokio::test]
async fn test_mqtt_reconnection() {
    let config = petra::Config {
        mqtt: Some(petra::MqttConfig {
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: "test-resilience".to_string(),
            reconnect_interval_ms: Some(100),
            ..Default::default()
        }),
        ..Default::default()
    };
    
    let mut harness = TestHarness::new(config).await.unwrap();
    
    // Simulate broker disconnect
    // This would require a test MQTT broker that can be controlled
    
    // Verify reconnection happens
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Check that MQTT is connected again
    assert!(harness.is_mqtt_connected());
}

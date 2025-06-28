mod common;
use common::{TestHarness, test_config};
use petra::{Value, config::*};
use std::time::Duration;

#[tokio::test]
async fn test_simple_control_loop() {
    let mut config = test_config();
    
    // Add a simple control block
    config.blocks.push(BlockConfig {
        name: "gain_block".to_string(),
        block_type: "Math".to_string(),
        inputs: [
            ("a".to_string(), BlockInput::Signal("test_input".to_string())),
            ("b".to_string(), BlockInput::Value(ValueOrSignal::Value(Value::Float(2.0)))),
        ].into(),
        outputs: [("result".to_string(), "test_output".to_string())].into(),
        params: Some([("operation".to_string(), serde_json::json!("multiply"))].into()),
    });
    
    let mut harness = TestHarness::new(config).await.unwrap();
    
    // Set input
    harness.set_signal("test_input", Value::Float(5.0)).unwrap();
    
    // Run for a bit
    harness.run_for(Duration::from_millis(200)).await;
    
    // Check output
    let output = harness.get_signal("test_output").unwrap();
    match output {
        Value::Float(f) => assert!((f - 10.0).abs() < 0.001),
        _ => panic!("Wrong type"),
    }
}

#[tokio::test]
async fn test_alarm_triggering() {
    let mut config = test_config();
    
    // Add alarm
    config.alarms = Some(vec![
        AlarmConfig {
            name: "high_temp".to_string(),
            condition: "test_input > 50".to_string(),
            message: "Temperature too high".to_string(),
            severity: "critical".to_string(),
            enabled: true,
            contacts: vec![],
            acknowledge_required: true,
            auto_reset: false,
        }
    ]);
    
    let mut harness = TestHarness::new(config).await.unwrap();
    
    // Trigger alarm
    harness.set_signal("test_input", Value::Float(60.0)).unwrap();
    
    // Wait for alarm to trigger
    let triggered = harness.wait_for_condition(
        "alarm.high_temp.active",
        |v| matches!(v, Value::Bool(true)),
        Duration::from_secs(1)
    ).await;
    
    assert!(triggered, "Alarm should have triggered");
}

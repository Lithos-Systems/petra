use petra::{Config, Engine, SignalBus, Value};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_engine_start_stop() {
    let config = Config {
        signals: vec![
            petra::config::SignalConfig {
                name: "test_signal".to_string(),
                signal_type: petra::config::SignalType::Float,
                initial: Some(petra::config::InitialValue::Float(0.0)),
                ..Default::default()
            }
        ],
        blocks: vec![],
        scan_time_ms: 100,
        ..Default::default()
    };
    
    let mut engine = Engine::new(config).expect("Failed to create engine");
    
    // Start engine
    let handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Let it run for a bit
    sleep(Duration::from_millis(500)).await;
    
    // Stop gracefully
    handle.abort();
    
    // Verify it stopped
    assert!(handle.is_finished());
}

#[tokio::test]
async fn test_signal_persistence_across_restart() {
    let bus = SignalBus::new();
    
    // Write a signal
    bus.write_signal("persistent_signal", Value::Float(42.0))
        .expect("Failed to write signal");
    
    // Create engine with the same bus
    let config = Config {
        signals: vec![
            petra::config::SignalConfig {
                name: "persistent_signal".to_string(),
                signal_type: petra::config::SignalType::Float,
                initial: Some(petra::config::InitialValue::Float(0.0)),
                ..Default::default()
            }
        ],
        blocks: vec![],
        scan_time_ms: 100,
        ..Default::default()
    };
    
    let engine = Engine::new_with_bus(config, bus.clone())
        .expect("Failed to create engine");
    
    // Verify signal value persisted
    let value = bus.read_signal("persistent_signal")
        .expect("Failed to read signal");
    
    match value {
        Value::Float(f) => assert_eq!(f, 42.0),
        _ => panic!("Wrong value type"),
    }
}

#[tokio::test]
async fn test_scan_timing_accuracy() {
    let config = Config {
        signals: vec![
            petra::config::SignalConfig {
                name: "scan_counter".to_string(),
                signal_type: petra::config::SignalType::Int,
                initial: Some(petra::config::InitialValue::Int(0)),
                ..Default::default()
            }
        ],
        blocks: vec![
            // Counter block that increments each scan
            petra::config::BlockConfig {
                name: "scan_counter_block".to_string(),
                block_type: "Counter".to_string(),
                inputs: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("enable".to_string(), 
                        petra::config::BlockInput::Value(
                            petra::config::ValueOrSignal::Value(
                                petra::Value::Bool(true)
                            )
                        )
                    );
                    map
                },
                outputs: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("count".to_string(), "scan_counter".to_string());
                    map
                },
                params: Some({
                    let mut map = std::collections::HashMap::new();
                    map.insert("increment".to_string(), serde_json::Value::from(1));
                    map
                }),
            }
        ],
        scan_time_ms: 50,  // 50ms = 20 scans/second
        ..Default::default()
    };
    
    let mut engine = Engine::new(config).expect("Failed to create engine");
    let bus = engine.signal_bus().clone();
    
    // Run engine in background
    let handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Wait for 1 second
    sleep(Duration::from_secs(1)).await;
    
    // Stop engine
    handle.abort();
    
    // Check scan count (should be ~20)
    let count = bus.read_signal("scan_counter")
        .expect("Failed to read counter");
    
    match count {
        Value::Int(n) => {
            assert!(n >= 18 && n <= 22, 
                "Scan count {} not in expected range 18-22", n);
        }
        _ => panic!("Wrong value type"),
    }
}

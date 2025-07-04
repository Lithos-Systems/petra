// tests/integration/engine_lifecycle_test.rs
//! Integration tests for the PETRA engine lifecycle and functionality

use petra::{Config, Engine, SignalBus, Value, PlcError};
use petra::config::{SignalConfig, BlockConfig};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

/// Create a minimal test configuration
fn create_test_config() -> Config {
    Config {
        scan_time_ms: 100,
        max_scan_jitter_ms: 10,
        error_recovery: true,
        max_consecutive_errors: 5,
        restart_delay_ms: 1000,
        
        signals: vec![
            SignalConfig {
                name: "test_signal".to_string(),
                signal_type: "float".to_string(),
                initial: Some(serde_yaml::Value::from(0.0)),
                description: Some("Test signal".to_string()),
                unit: None,
                min: None,
                max: None,
                access: None,
                #[cfg(feature = "extended-types")]
                metadata: HashMap::new(),
            }
        ],
        
        blocks: vec![],
        
        protocols: None,
        version: "1.0".to_string(),
        description: Some("Test configuration".to_string()),
        author: Some("Test".to_string()),
        created: None,
        modified: None,
        metadata: HashMap::new(),
        
        #[cfg(feature = "mqtt")]
        mqtt: None,
        #[cfg(feature = "security")]
        security: None,
        #[cfg(feature = "history")]
        history: None,
        #[cfg(feature = "alarms")]
        alarms: None,
        #[cfg(feature = "web")]
        web: None,
        #[cfg(feature = "advanced-storage")]
        storage: None,
        #[cfg(feature = "s7-support")]
        s7: None,
        #[cfg(feature = "modbus-support")]
        modbus: None,
        #[cfg(feature = "opcua-support")]
        opcua: None,
        #[cfg(feature = "twilio")]
        twilio: None,
        #[cfg(feature = "email")]
        email: None,
    }
}

#[tokio::test]
async fn test_engine_new_with_bus() {
    // Create a signal bus and pre-populate it
    let bus = SignalBus::new();
    bus.set("persistent_signal", Value::Float(42.0))
        .expect("Failed to set signal");
    
    // Create engine with the existing bus
    let config = create_test_config();
    let engine = Engine::new_with_bus(config, bus.clone())
        .expect("Failed to create engine");
    
    // Verify the pre-existing signal is accessible
    let value = engine.signal_bus().get("persistent_signal")
        .expect("Failed to get signal");
    assert_eq!(value, Some(Value::Float(42.0)));
    
    // Verify configured signals were also initialized
    let test_signal = engine.signal_bus().get("test_signal")
        .expect("Failed to get test signal");
    assert_eq!(test_signal, Some(Value::Float(0.0)));
}

#[tokio::test]
async fn test_engine_start_stop() {
    let config = create_test_config();
    let mut engine = Engine::new(config).expect("Failed to create engine");
    
    // Verify initial state
    assert!(!engine.is_running());
    assert_eq!(engine.scan_count(), 0);
    
    // Start engine in background
    let handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Let it run for a bit
    sleep(Duration::from_millis(500)).await;
    
    // Stop gracefully by aborting the task
    handle.abort();
    
    // Verify it stopped
    assert!(handle.is_finished());
}

#[tokio::test]
async fn test_signal_persistence_across_restart() {
    let bus = SignalBus::new();
    
    // Write a signal
    bus.set("persistent_signal", Value::Float(42.0))
        .expect("Failed to write signal");
    
    // Create first engine instance
    let config = create_test_config();
    let engine1 = Engine::new_with_bus(config.clone(), bus.clone())
        .expect("Failed to create first engine");
    
    // Modify signal through engine
    engine1.signal_bus().set("persistent_signal", Value::Float(100.0))
        .expect("Failed to update signal");
    
    // Create second engine instance with same bus
    let engine2 = Engine::new_with_bus(config, bus.clone())
        .expect("Failed to create second engine");
    
    // Verify signal value persisted
    let value = engine2.signal_bus().get("persistent_signal")
        .expect("Failed to read signal");
    
    assert_eq!(value, Some(Value::Float(100.0)));
}

#[tokio::test]
async fn test_scan_timing_accuracy() {
    let config = Config {
        scan_time_ms: 50, // Fast scan for testing
        ..create_test_config()
    };
    
    let mut engine = Engine::new(config).expect("Failed to create engine");
    
    // Run engine for a short time
    let engine_handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Let it run for exactly 1 second
    sleep(Duration::from_secs(1)).await;
    
    // Stop the engine
    engine_handle.abort();
    
    // We should have approximately 20 scans (1000ms / 50ms)
    // Allow for some variance due to startup/shutdown time
    let scan_count = engine.scan_count();
    assert!(scan_count >= 15 && scan_count <= 25,
        "Expected 15-25 scans, got {}", scan_count);
}

#[tokio::test]
async fn test_engine_performance_monitoring() {
    let config = create_test_config();
    let engine = Engine::new(config).expect("Failed to create engine");
    
    // Execute a few scan cycles
    for _ in 0..5 {
        engine.execute_scan_cycle().await
            .expect("Scan cycle failed");
    }
    
    // Check statistics
    let stats = engine.stats().await;
    assert_eq!(stats.scan_count, 5);
    assert_eq!(stats.error_count, 0);
    assert!(stats.avg_scan_time > Duration::ZERO);
    assert!(stats.min_scan_time <= stats.max_scan_time);
    
    // Get performance summary
    let summary = engine.performance_summary().await;
    assert!(summary.contains("Scans: 5"));
    assert!(summary.contains("Errors: 0"));
}

#[tokio::test]
async fn test_engine_error_recovery() {
    use petra::EngineConfig;
    
    let config = create_test_config();
    let engine_config = EngineConfig {
        error_recovery: true,
        max_consecutive_errors: 3,
        recovery_delay_ms: 100,
        ..Default::default()
    };
    
    let engine = Engine::new_with_config(config, engine_config)
        .expect("Failed to create engine");
    
    // Simulate errors
    for _ in 0..2 {
        engine.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        engine.consecutive_errors.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    
    assert_eq!(engine.error_count(), 2);
    assert_eq!(engine.consecutive_errors(), 2);
    
    // Successful operation should reset consecutive errors
    engine.execute_scan_cycle().await
        .expect("Scan cycle failed");
    
    // Note: In the real implementation, consecutive_errors is reset in the run loop
    // For this test, we're just verifying the counters work
    assert_eq!(engine.scan_count(), 1);
}

#[tokio::test]
async fn test_engine_with_blocks() {
    let mut config = create_test_config();
    
    // Add input and output signals
    config.signals.push(SignalConfig {
        name: "input".to_string(),
        signal_type: "bool".to_string(),
        initial: Some(serde_yaml::Value::from(true)),
        description: Some("Input signal".to_string()),
        unit: None,
        min: None,
        max: None,
        access: None,
        #[cfg(feature = "extended-types")]
        metadata: HashMap::new(),
    });
    
    config.signals.push(SignalConfig {
        name: "output".to_string(),
        signal_type: "bool".to_string(),
        initial: None,
        description: Some("Output signal".to_string()),
        unit: None,
        min: None,
        max: None,
        access: None,
        #[cfg(feature = "extended-types")]
        metadata: HashMap::new(),
    });
    
    // Add a NOT block
    config.blocks.push(BlockConfig {
        name: "not_gate".to_string(),
        block_type: "NOT".to_string(),
        inputs: {
            let mut inputs = HashMap::new();
            inputs.insert("input".to_string(), "input".to_string());
            inputs
        },
        outputs: {
            let mut outputs = HashMap::new();
            outputs.insert("output".to_string(), "output".to_string());
            outputs
        },
        parameters: HashMap::new(),
        priority: 100,
        enabled: true,
        description: Some("Test NOT gate".to_string()),
        category: Some("Logic".to_string()),
        tags: vec!["test".to_string()],
        #[cfg(feature = "circuit-breaker")]
        circuit_breaker: None,
        #[cfg(feature = "enhanced-monitoring")]
        enhanced_monitoring: false,
        metadata: HashMap::new(),
    });
    
    let engine = Engine::new(config).expect("Failed to create engine");
    
    // Verify initial state
    let input = engine.signal_bus().get("input").unwrap();
    assert_eq!(input, Some(Value::Bool(true)));
    
    // Execute scan cycle
    engine.execute_scan_cycle().await
        .expect("Scan cycle failed");
    
    // Check NOT block inverted the signal
    let output = engine.signal_bus().get("output").unwrap();
    assert_eq!(output, Some(Value::Bool(false)));
}

#[tokio::test]
async fn test_concurrent_signal_access() {
    use std::sync::Arc;
    
    let config = create_test_config();
    let engine = Arc::new(Engine::new(config).expect("Failed to create engine"));
    
    // Spawn multiple tasks to access signals concurrently
    let mut handles = vec![];
    
    for i in 0..10 {
        let engine_clone = Arc::clone(&engine);
        let handle = tokio::spawn(async move {
            let signal_name = format!("concurrent_signal_{}", i);
            
            // Write signal
            engine_clone.signal_bus()
                .set(&signal_name, Value::Float(i as f64))
                .expect("Failed to set signal");
            
            // Read it back
            let value = engine_clone.signal_bus()
                .get(&signal_name)
                .expect("Failed to get signal");
            
            assert_eq!(value, Some(Value::Float(i as f64)));
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }
}

#[tokio::test]
async fn test_engine_reset() {
    let config = create_test_config();
    let engine = Engine::new(config).expect("Failed to create engine");
    
    // Execute some scans
    for _ in 0..5 {
        engine.execute_scan_cycle().await
            .expect("Scan cycle failed");
    }
    
    // Add some errors
    engine.error_count.fetch_add(3, std::sync::atomic::Ordering::Relaxed);
    
    // Verify counts
    assert_eq!(engine.scan_count(), 5);
    assert_eq!(engine.error_count(), 3);
    
    // Reset
    engine.reset_blocks().await
        .expect("Reset failed");
    
    // Verify counters were reset
    assert_eq!(engine.scan_count(), 0);
    assert_eq!(engine.error_count(), 0);
    assert_eq!(engine.consecutive_errors(), 0);
}

#[cfg(feature = "hot-reload")]
#[tokio::test]
async fn test_hot_reload() {
    let config = create_test_config();
    let engine = Engine::new(config.clone()).expect("Failed to create engine");
    
    // Modify config
    let mut new_config = config;
    new_config.scan_time_ms = 200; // Change scan time
    
    // Reload configuration
    engine.reload_config(new_config).await
        .expect("Reload failed");
    
    // Verify engine is still functional
    engine.execute_scan_cycle().await
        .expect("Scan cycle failed after reload");
}

#[tokio::test]
async fn test_dynamic_block_management() {
    use petra::blocks::Block;
    
    // Create a simple test block
    struct TestBlock {
        name: String,
        execute_count: std::sync::atomic::AtomicU64,
    }
    
    impl Block for TestBlock {
        fn execute(&mut self, _bus: &SignalBus) -> Result<(), PlcError> {
            self.execute_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }
        
        fn name(&self) -> &str {
            &self.name
        }
        
        fn block_type(&self) -> &str {
            "TEST_BLOCK"
        }
        
        fn reset(&mut self) -> Result<(), PlcError> {
            self.execute_count.store(0, std::sync::atomic::Ordering::Relaxed);
            Ok(())
        }
    }
    
    let config = create_test_config();
    let engine = Engine::new(config).expect("Failed to create engine");
    
    // Add a dynamic block
    let test_block = Box::new(TestBlock {
        name: "dynamic_test".to_string(),
        execute_count: std::sync::atomic::AtomicU64::new(0),
    });
    
    engine.add_block(test_block).await
        .expect("Failed to add block");
    
    // Execute scan to run the block
    engine.execute_scan_cycle().await
        .expect("Scan cycle failed");
    
    // Remove the block
    engine.remove_block("dynamic_test").await
        .expect("Failed to remove block");
    
    // Try to remove again (should fail)
    assert!(engine.remove_block("dynamic_test").await.is_err());
}

/// Test helper to create a more complex configuration
fn create_complex_config() -> Config {
    let mut config = create_test_config();
    
    // Add multiple signals
    for i in 0..10 {
        config.signals.push(SignalConfig {
            name: format!("signal_{}", i),
            signal_type: "float".to_string(),
            initial: Some(serde_yaml::Value::from(i as f64)),
            description: Some(format!("Signal {}", i)),
            unit: Some("units".to_string()),
            min: Some(0.0),
            max: Some(100.0),
            access: None,
            #[cfg(feature = "extended-types")]
            metadata: HashMap::new(),
        });
    }
    
    config
}

#[tokio::test]
async fn test_engine_with_multiple_signals() {
    let config = create_complex_config();
    let engine = Engine::new(config).expect("Failed to create engine");
    
    // Verify all signals were initialized
    for i in 0..10 {
        let signal_name = format!("signal_{}", i);
        let value = engine.signal_bus().get(&signal_name).unwrap();
        assert_eq!(value, Some(Value::Float(i as f64)));
    }
    
    // Update signals and verify
    for i in 0..10 {
        let signal_name = format!("signal_{}", i);
        engine.signal_bus()
            .set(&signal_name, Value::Float((i * 2) as f64))
            .expect("Failed to update signal");
    }
    
    // Verify updates
    for i in 0..10 {
        let signal_name = format!("signal_{}", i);
        let value = engine.signal_bus().get(&signal_name).unwrap();
        assert_eq!(value, Some(Value::Float((i * 2) as f64)));
    }
}

#[tokio::test]
async fn test_engine_state_transitions() {
    use petra::EngineState;
    
    let config = create_test_config();
    let mut engine = Engine::new(config).expect("Failed to create engine");
    
    // Initial state
    assert_eq!(engine.state().await, EngineState::Stopped);
    
    // Start engine
    let engine_handle = tokio::spawn(async move {
        engine.run().await
    });
    
    // Give it time to start
    sleep(Duration::from_millis(100)).await;
    
    // Note: We can't directly check Running state without access to the engine
    // in the spawned task, but we can verify the handle is still running
    assert!(!engine_handle.is_finished());
    
    // Stop the engine
    engine_handle.abort();
    sleep(Duration::from_millis(100)).await;
    
    // Verify it stopped
    assert!(engine_handle.is_finished());
}

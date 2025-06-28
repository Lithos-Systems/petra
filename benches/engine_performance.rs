use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use petra::{Config, Engine, SignalBus, Value};
use petra::config::{SignalConfig, BlockConfig, SignalType, InitialValue};
use petra::block::{Block, BlockFactory};
use std::collections::HashMap;
use std::time::Duration;

fn create_benchmark_config(num_signals: usize, num_blocks: usize) -> Config {
    let mut signals = Vec::new();
    let mut blocks = Vec::new();
    
    // Create signals
    for i in 0..num_signals {
        signals.push(SignalConfig {
            name: format!("signal_{}", i),
            signal_type: "float".to_string(),
            initial: Some(InitialValue::Float(0.0)),
            description: None,
            unit: None,
            min: None,
            max: None,
            s7_config: None,
            modbus_config: None,
            persistence: None,
            history: None,
            validation: None,
            max_updates_per_second: None,
        });
    }
    
    // Create simple logic blocks (AND gates with 2 inputs)
    for i in 0..num_blocks {
        let mut inputs = HashMap::new();
        inputs.insert("in1".to_string(), petra::config::BlockInput::Signal(format!("signal_{}", i % num_signals)));
        inputs.insert("in2".to_string(), petra::config::BlockInput::Signal(format!("signal_{}", (i + 1) % num_signals)));
        
        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), format!("signal_{}", (i + num_signals) % num_signals));
        
        blocks.push(BlockConfig {
            name: format!("block_{}", i),
            block_type: "AND".to_string(),
            inputs,
            outputs,
            params: None,
        });
    }
    
    Config {
        signals,
        blocks,
        scan_time_ms: 50,
        mqtt: None,
        twilio: None,
        s7_connections: None,
        modbus_connections: None,
        alarms: None,
        history: None,
        storage: None,
        security: None,
        opcua: None,
        max_signals: None,
        signal_ttl_secs: None,
    }
}

fn benchmark_scan_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_performance");
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(3));
    
    // Test different scales
    for (num_signals, num_blocks) in &[(100, 10), (1000, 100), (10000, 1000)] {
        let config = create_benchmark_config(*num_signals, *num_blocks);
        let mut engine = Engine::new(config).expect("Failed to create engine");
        
        group.throughput(Throughput::Elements(*num_blocks as u64));
        group.bench_with_input(
            BenchmarkId::new("signals_and_blocks", format!("{}/{}", num_signals, num_blocks)),
            &(num_signals, num_blocks),
            |b, _| {
                b.iter(|| {
                    engine.execute_scan_cycle();
                });
            },
        );
    }
    
    group.finish();
}

fn benchmark_signal_bus_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_bus");
    group.measurement_time(Duration::from_secs(5));
    
    let bus = SignalBus::new();
    
    // Pre-populate signals
    for i in 0..1000 {
        bus.write_signal(&format!("signal_{}", i), Value::Float(0.0))
            .expect("Failed to write signal");
    }
    
    group.bench_function("write_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            let _ = bus.write_signal(
                &format!("signal_{}", counter % 1000), 
                Value::Float(black_box(counter as f64))
            );
            counter += 1;
        });
    });
    
    group.bench_function("read_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            let _ = bus.read_signal(&format!("signal_{}", counter % 1000));
            counter += 1;
        });
    });
    
    group.bench_function("batch_write_10", |b| {
        b.iter(|| {
            let updates: Vec<(&str, Value)> = (0..10)
                .map(|i| {
                    let signal = format!("signal_{}", i);
                    // Leak the string to get a &'static str for the benchmark
                    let leaked = Box::leak(signal.into_boxed_str());
                    (leaked as &str, Value::Float(i as f64))
                })
                .collect();
            let _ = bus.write_batch(updates);
        });
    });
    
    group.bench_function("concurrent_read_write", |b| {
        use std::sync::Arc;
        use std::thread;
        
        let bus = Arc::new(SignalBus::new());
        
        // Pre-populate
        for i in 0..100 {
            bus.write_signal(&format!("sig_{}", i), Value::Float(0.0))
                .expect("Failed to write");
        }
        
        b.iter(|| {
            let mut handles = vec![];
            
            // Spawn readers
            for i in 0..4 {
                let bus_clone = bus.clone();
                let handle = thread::spawn(move || {
                    for j in 0..25 {
                        let _ = bus_clone.read_signal(&format!("sig_{}", (i * 25 + j) % 100));
                    }
                });
                handles.push(handle);
            }
            
            // Spawn writers
            for i in 0..2 {
                let bus_clone = bus.clone();
                let handle = thread::spawn(move || {
                    for j in 0..50 {
                        let _ = bus_clone.write_signal(
                            &format!("sig_{}", (i * 50 + j) % 100),
                            Value::Float(j as f64)
                        );
                    }
                });
                handles.push(handle);
            }
            
            // Wait for all threads
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_block_execution(c: &mut Criterion) {
    use petra::block::*;
    
    let mut group = c.benchmark_group("block_execution");
    let bus = SignalBus::new();
    
    // Initialize signals
    bus.write_signal("input1", Value::Bool(true)).unwrap();
    bus.write_signal("input2", Value::Bool(false)).unwrap();
    bus.write_signal("output", Value::Bool(false)).unwrap();
    bus.write_signal("float_in", Value::Float(50.0)).unwrap();
    bus.write_signal("float_out", Value::Float(0.0)).unwrap();
    
    // Create blocks using the factory
    let factory = BlockFactory::new();
    
    // AND block
    let mut and_config = BlockConfig {
        name: "test_and".to_string(),
        block_type: "AND".to_string(),
        inputs: HashMap::from([
            ("in1".to_string(), petra::config::BlockInput::Signal("input1".to_string())),
            ("in2".to_string(), petra::config::BlockInput::Signal("input2".to_string())),
        ]),
        outputs: HashMap::from([
            ("out".to_string(), "output".to_string()),
        ]),
        params: None,
    };
    
    let mut and_block = factory.create_block(&and_config)
        .expect("Failed to create AND block");
    
    group.bench_function("and_block", |b| {
        b.iter(|| {
            and_block.execute(&bus).unwrap();
        });
    });
    
    // Math block (more complex)
    let mut math_config = BlockConfig {
        name: "test_math".to_string(),
        block_type: "Math".to_string(),
        inputs: HashMap::from([
            ("a".to_string(), petra::config::BlockInput::Signal("float_in".to_string())),
            ("b".to_string(), petra::config::BlockInput::Value(
                petra::config::ValueOrSignal::Value(Value::Float(2.0))
            )),
        ]),
        outputs: HashMap::from([
            ("result".to_string(), "float_out".to_string()),
        ]),
        params: Some(HashMap::from([
            ("operation".to_string(), serde_json::json!("multiply")),
        ])),
    };
    
    let mut math_block = factory.create_block(&math_config)
        .expect("Failed to create Math block");
    
    group.bench_function("math_block", |b| {
        b.iter(|| {
            math_block.execute(&bus).unwrap();
        });
    });
    
    group.finish();
}

fn benchmark_history_write(c: &mut Criterion) {
    use petra::history::{HistoryManager, HistoryConfig};
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    
    let mut group = c.benchmark_group("history");
    group.measurement_time(Duration::from_secs(5));
    
    let rt = Runtime::new().unwrap();
    let bus = Arc::new(SignalBus::new());
    
    // Pre-populate signals
    for i in 0..100 {
        bus.write_signal(&format!("hist_signal_{}", i), Value::Float(i as f64))
            .unwrap();
    }
    
    let config = HistoryConfig {
        enabled: true,
        directory: "/tmp/petra_bench_history".into(),
        file_size_mb: 10,
        compression: "zstd".to_string(),
        retention_days: 7,
        signals: Some(vec!["hist_signal_*".to_string()]),
    };
    
    group.bench_function("history_batch_write", |b| {
        let manager = rt.block_on(async {
            HistoryManager::new(config.clone(), bus.clone())
                .await
                .expect("Failed to create history manager")
        });
        
        b.iter(|| {
            rt.block_on(async {
                // Simulate a batch of signal changes
                for i in 0..10 {
                    let signal = format!("hist_signal_{}", i);
                    let value = Value::Float((i * 10) as f64);
                    manager.record_change(&signal, &value).await.unwrap();
                }
            });
        });
    });
    
    group.finish();
    
    // Cleanup
    let _ = std::fs::remove_dir_all("/tmp/petra_bench_history");
}

#[cfg(feature = "enhanced")]
fn benchmark_enhanced_features(c: &mut Criterion) {
    use petra::{EnhancedSignalBus, SignalBusConfig};
    use std::time::Duration;
    
    let mut group = c.benchmark_group("enhanced_features");
    
    let config = SignalBusConfig {
        max_signals: 10000,
        signal_ttl: Duration::from_secs(3600),
        cleanup_interval: Duration::from_secs(60),
        hot_signal_threshold: 100,
    };
    
    let bus = EnhancedSignalBus::new(config);
    
    // Pre-populate
    for i in 0..1000 {
        bus.write_signal(&format!("signal_{}", i), Value::Float(0.0)).unwrap();
    }
    
    // Make a signal "hot"
    for _ in 0..200 {
        let _ = bus.read_signal("hot_signal");
    }
    
    group.bench_function("enhanced_write_hot_signal", |b| {
        b.iter(|| {
            bus.write_signal("hot_signal", Value::Float(black_box(42.0))).unwrap();
        });
    });
    
    group.bench_function("enhanced_read_hot_signal", |b| {
        b.iter(|| {
            let _ = bus.read_signal("hot_signal").unwrap();
        });
    });
    
    group.bench_function("enhanced_cold_signal", |b| {
        let mut counter = 0;
        b.iter(|| {
            let signal = format!("cold_signal_{}", counter % 100);
            let _ = bus.read_signal(&signal);
            counter += 1;
        });
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_scan_performance,
    benchmark_signal_bus_operations,
    benchmark_block_execution,
    benchmark_history_write,
);

#[cfg(feature = "enhanced")]
criterion_group!(
    enhanced_benches,
    benchmark_enhanced_features
);

#[cfg(not(feature = "enhanced"))]
criterion_main!(benches);

#[cfg(feature = "enhanced")]
criterion_main!(benches, enhanced_benches);

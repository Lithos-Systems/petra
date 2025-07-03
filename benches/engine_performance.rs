// benches/engine_performance.rs - Fixed version

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use petra::config::{BlockConfig, SignalConfig, Config};
use petra::{Engine, SignalBus, Value};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

fn create_benchmark_config(num_signals: usize, num_blocks: usize) -> Config {
    let mut signals = Vec::new();
    let mut blocks = Vec::new();

    // Create signals with all required fields
    for i in 0..num_signals {
        signals.push(SignalConfig {
            name: format!("signal_{}", i),
            signal_type: "float".to_string(),
            description: Some(format!("Test signal {}", i)),
            initial: Some(serde_yaml::Value::from(0.0f64)),
            metadata: HashMap::new(),
            tags: vec![],
        });
    }

    // Create simple logic blocks (AND gates with 2 inputs)
    for i in 0..num_blocks {
        let mut inputs = HashMap::new();
        inputs.insert("in1".to_string(), format!("signal_{}", i % num_signals));
        inputs.insert(
            "in2".to_string(),
            format!("signal_{}", (i + 1) % num_signals),
        );

        let mut outputs = HashMap::new();
        outputs.insert(
            "out".to_string(),
            format!("signal_{}", (i + num_signals) % num_signals),
        );

        blocks.push(BlockConfig {
            name: format!("block_{}", i),
            block_type: "AND".to_string(),
            description: Some(format!("Test AND block {}", i)),
            inputs,
            outputs,
            params: HashMap::new(),
            tags: vec![],
        });
    }

    Config {
        signals,
        blocks,
        scan_time_ms: 50,
        max_scan_jitter_ms: 50,
        error_recovery: true,
        // Feature-gated fields
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
        protocols: None,
    }
}

fn benchmark_scan_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_performance");
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(3));

    // Create runtime for async operations
    let rt = Runtime::new().unwrap();

    // Test different scales
    for (num_signals, num_blocks) in &[(100, 10), (1000, 100), (10000, 1000)] {
        let config = create_benchmark_config(*num_signals, *num_blocks);
        let engine = Engine::new(config).expect("Failed to create engine");

        group.throughput(Throughput::Elements(*num_blocks as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "signals_and_blocks",
                format!("{}_signals_{}_blocks", num_signals, num_blocks),
            ),
            &(*num_signals, *num_blocks),
            |b, _| {
                b.iter(|| {
                    // Use block_on since execute_scan_cycle is async
                    rt.block_on(async {
                        black_box(engine.execute_scan_cycle().await);
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_signal_bus_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_bus");

    // Benchmark signal writes
    group.bench_function("write_1000_signals", |b| {
        let bus = SignalBus::new();

        b.iter(|| {
            for i in 0..1000 {
                bus.set(&format!("signal_{}", i), Value::Float(black_box(i as f64)))
                    .unwrap();
            }
        });
    });

    // Benchmark signal reads
    group.bench_function("read_1000_signals", |b| {
        let bus = SignalBus::new();

        // Pre-populate signals
        for i in 0..1000 {
            bus.set(&format!("signal_{}", i), Value::Float(i as f64))
                .unwrap();
        }

        b.iter(|| {
            for i in 0..1000 {
                black_box(bus.get(&format!("signal_{}", i)).unwrap());
            }
        });
    });

    // Benchmark atomic updates
    group.bench_function("atomic_update_100_signals", |b| {
        let bus = SignalBus::new();

        // Pre-populate
        for i in 0..100 {
            bus.set(&format!("counter_{}", i), Value::Int(0))
                .unwrap();
        }

        b.iter(|| {
            for i in 0..100 {
                bus.update(&format!("counter_{}", i), |old| {
                    match old {
                        Some(Value::Int(n)) => Value::Int(n + 1),
                        _ => Value::Int(1),
                    }
                })
                .unwrap();
            }
        });
    });

    // Benchmark concurrent access
    group.bench_function("concurrent_access", |b| {
        use std::sync::Arc;
        use std::thread;

        let bus = Arc::new(SignalBus::new());

        // Pre-populate
        for i in 0..100 {
            bus.set(&format!("sig_{}", i), Value::Float(0.0)).unwrap();
        }

        b.iter(|| {
            let mut handles = vec![];

            // Spawn readers
            for i in 0..4 {
                let bus_clone = bus.clone();
                let handle = thread::spawn(move || {
                    for j in 0..25 {
                        let _ = bus_clone.get(&format!("sig_{}", (i * 25 + j) % 100));
                    }
                });
                handles.push(handle);
            }

            // Spawn writers
            for i in 0..2 {
                let bus_clone = bus.clone();
                let handle = thread::spawn(move || {
                    for j in 0..50 {
                        let _ = bus_clone.set(
                            &format!("sig_{}", (i * 50 + j) % 100),
                            Value::Float(j as f64),
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
    use petra::blocks::create_block;

    let mut group = c.benchmark_group("block_execution");
    let bus = SignalBus::new();

    // Initialize signals
    bus.set("input1", Value::Bool(true)).unwrap();
    bus.set("input2", Value::Bool(false)).unwrap();
    bus.set("output", Value::Bool(false)).unwrap();
    bus.set("float_in", Value::Float(50.0)).unwrap();
    bus.set("float_out", Value::Float(0.0)).unwrap();

    // AND block
    let and_config = BlockConfig {
        name: "test_and".to_string(),
        block_type: "AND".to_string(),
        description: Some("Test AND block".to_string()),
        inputs: HashMap::from([
            ("in1".to_string(), "input1".to_string()),
            ("in2".to_string(), "input2".to_string()),
        ]),
        outputs: HashMap::from([("out".to_string(), "output".to_string())]),
        params: HashMap::new(),
        tags: vec![],
    };

    let mut and_block = create_block(&and_config).expect("Failed to create AND block");

    group.bench_function("and_block", |b| {
        b.iter(|| {
            and_block.execute(&bus).unwrap();
        });
    });

    // Math block (more complex)
    let math_config = BlockConfig {
        name: "test_math".to_string(),
        block_type: "MUL".to_string(),  // Use MUL instead of Math
        description: Some("Test multiplication block".to_string()),
        inputs: HashMap::from([
            ("in1".to_string(), "float_in".to_string()),
            ("in2".to_string(), "float_in".to_string()),
        ]),
        outputs: HashMap::from([("out".to_string(), "float_out".to_string())]),
        params: HashMap::new(),
        tags: vec![],
    };

    let mut math_block = create_block(&math_config).expect("Failed to create Math block");

    group.bench_function("math_block", |b| {
        b.iter(|| {
            math_block.execute(&bus).unwrap();
        });
    });

    group.finish();
}

#[cfg(feature = "history")]
fn benchmark_history_write(c: &mut Criterion) {
    use petra::history::{HistoryConfig, HistoryManager};
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    let mut group = c.benchmark_group("history");
    group.measurement_time(Duration::from_secs(5));

    let rt = Runtime::new().unwrap();
    let bus = Arc::new(SignalBus::new());

    // Pre-populate signals
    for i in 0..100 {
        bus.set(&format!("hist_signal_{}", i), Value::Float(i as f64))
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

// Only define the benches once based on features
#[cfg(feature = "history")]
criterion_group!(
    benches,
    benchmark_scan_performance,
    benchmark_signal_bus_operations,
    benchmark_block_execution,
    benchmark_history_write,
);

#[cfg(not(feature = "history"))]
criterion_group!(
    benches,
    benchmark_scan_performance,
    benchmark_signal_bus_operations,
    benchmark_block_execution,
);

criterion_main!(benches);

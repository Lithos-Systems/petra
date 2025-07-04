//! Engine performance benchmarks

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
                        // FIX: Handle the Result by using .unwrap() or .expect()
                        // This silences the warning by explicitly acknowledging we're discarding the error
                        let _ = black_box(engine.execute_scan_cycle().await);
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
                let _ = bus.set(&format!("signal_{}", i), Value::Float(black_box(i as f64)));
            }
        });
    });

    // Benchmark signal reads
    group.bench_function("read_1000_signals", |b| {
        let bus = SignalBus::new();

        // Pre-populate signals
        for i in 0..1000 {
            let _ = bus.set(&format!("signal_{}", i), Value::Float(i as f64));
        }

        b.iter(|| {
            for i in 0..1000 {
                let _ = black_box(bus.get(&format!("signal_{}", i)));
            }
        });
    });

    // Benchmark atomic updates
    group.bench_function("atomic_update_100_signals", |b| {
        let bus = SignalBus::new();

        // Pre-populate
        for i in 0..100 {
            let _ = bus.set(&format!("counter_{}", i), Value::Integer(0));
        }

        b.iter(|| {
            for i in 0..100 {
                let _ = bus.update(&format!("counter_{}", i), |old| {
                    match old {
                        Some(Value::Integer(n)) => Value::Integer(n + 1),
                        _ => Value::Integer(1),
                    }
                });
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
            let _ = bus.set(&format!("sig_{}", i), Value::Float(0.0));
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
                            Value::Float(black_box((i * 50 + j) as f64)),
                        );
                    }
                });
                handles.push(handle);
            }

            // Wait for all threads
            for handle in handles {
                let _ = handle.join();
            }
        });
    });

    group.finish();
}

fn benchmark_block_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_execution");

    // Benchmark individual block types
    group.bench_function("and_block", |b| {
        let bus = SignalBus::new();
        let _ = bus.set("input1", Value::Bool(true));
        let _ = bus.set("input2", Value::Bool(false));

        let mut inputs = HashMap::new();
        inputs.insert("in1".to_string(), "input1".to_string());
        inputs.insert("in2".to_string(), "input2".to_string());

        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), "and_output".to_string());

        let config = BlockConfig {
            name: "test_and".to_string(),
            block_type: "AND".to_string(),
            description: Some("Test AND block".to_string()),
            inputs,
            outputs,
            params: HashMap::new(),
            tags: vec![],
        };

        let mut block = petra::blocks::create_block(&config)
            .expect("Failed to create block");

        b.iter(|| {
            let _ = black_box(block.execute(&bus));
        });
    });

    // Benchmark PID controller if available
    #[cfg(feature = "pid-control")]
    group.bench_function("pid_block", |b| {
        let bus = SignalBus::new();
        let _ = bus.set("process_value", Value::Float(95.0));

        let mut params = HashMap::new();
        params.insert("kp".to_string(), serde_json::json!(1.0));
        params.insert("ki".to_string(), serde_json::json!(0.1));
        params.insert("kd".to_string(), serde_json::json!(0.01));
        params.insert("setpoint".to_string(), serde_json::json!(100.0));

        let mut inputs = HashMap::new();
        inputs.insert("pv".to_string(), "process_value".to_string());

        let mut outputs = HashMap::new();
        outputs.insert("cv".to_string(), "control_output".to_string());

        let config = BlockConfig {
            name: "test_pid".to_string(),
            block_type: "PID".to_string(),
            description: Some("Test PID block".to_string()),
            inputs,
            outputs,
            params,
            tags: vec![],
        };

        let mut block = petra::blocks::create_block(&config)
            .expect("Failed to create block");

        b.iter(|| {
            let _ = black_box(block.execute(&bus));
        });
    });

    group.finish();
}

fn benchmark_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_operations");

    group.bench_function("value_creation", |b| {
        b.iter(|| {
            black_box(Value::Float(42.0));
            black_box(Value::Integer(42));
            black_box(Value::Bool(true));
        });
    });

    group.bench_function("value_conversion", |b| {
        let float_val = Value::Float(42.5);
        let int_val = Value::Integer(42);
        let bool_val = Value::Bool(true);

        b.iter(|| {
            let _ = black_box(float_val.as_f64());
            let _ = black_box(int_val.as_i64());
            let _ = black_box(bool_val.as_bool());
        });
    });

    #[cfg(feature = "value-arithmetic")]
    group.bench_function("value_arithmetic", |b| {
        let val1 = Value::Float(10.0);
        let val2 = Value::Float(20.0);

        b.iter(|| {
            let _ = black_box(val1.add(&val2));
            let _ = black_box(val1.multiply(&val2));
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5));
    targets = benchmark_scan_performance, benchmark_signal_bus_operations, benchmark_block_execution, benchmark_value_operations
}

criterion_main!(benches);

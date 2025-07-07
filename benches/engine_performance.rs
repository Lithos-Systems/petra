//! Enhanced Engine performance benchmarks with configurable signal counts

use criterion::measurement::WallTime;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use petra::config::{BlockConfig, Config, SignalConfig};
use petra::{Engine, SignalBus, Value};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, SystemTime};
use tokio::runtime::Runtime;

/// Parse signal/block counts from environment variables or use defaults
fn parse_counts(env_var: &str, default: &str) -> Vec<usize> {
    env::var(env_var)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

/// Get benchmark configuration from environment or defaults
fn get_benchmark_config() -> (Vec<usize>, Vec<usize>) {
    let signals = parse_counts("PETRA_BENCH_SIGNALS", "100,1000,10000");
    let blocks = parse_counts("PETRA_BENCH_BLOCKS", "10,100,1000");

    // Ensure we have matching counts or use defaults
    if signals.is_empty() || blocks.is_empty() || signals.len() != blocks.len() {
        eprintln!("Warning: Invalid signal/block configuration, using defaults");
        return (vec![100, 1000, 10000], vec![10, 100, 1000]);
    }

    (signals, blocks)
}

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
            category: Some("Benchmark".to_string()),
            source: Some("Generator".to_string()),
            update_frequency_ms: Some(100),
            metadata: HashMap::new(),
            tags: vec![],
            #[cfg(feature = "engineering-types")]
            units: None,
            #[cfg(feature = "engineering-types")]
            min_value: None,
            #[cfg(feature = "engineering-types")]
            max_value: None,
            #[cfg(feature = "quality-codes")]
            quality_enabled: false,
            #[cfg(feature = "history")]
            log_to_history: false,
            #[cfg(feature = "history")]
            log_interval_ms: 0,
            #[cfg(feature = "alarms")]
            enable_alarms: false,
            #[cfg(feature = "validation")]
            validation: None,
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
            priority: 0,
            enabled: true,
            tags: vec![],
            category: Some("Logic".to_string()),
            metadata: HashMap::new(),
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
            #[cfg(feature = "enhanced-monitoring")]
            enhanced_monitoring: false,
        });
    }

    Config {
        // Core engine settings
        scan_time_ms: 50,
        // Use a conservative jitter value to avoid validation edge cases
        max_scan_jitter_ms: 20,
        error_recovery: true,
        max_consecutive_errors: 10,
        restart_delay_ms: 5000,

        // Signal and block definitions
        signals,
        blocks,

        // Metadata fields
        version: "1.0.0".to_string(),
        description: Some("Benchmark configuration".to_string()),
        author: Some("Benchmark Generator".to_string()),
        created_at: Some(SystemTime::now()),
        modified_at: Some(SystemTime::now()),
        tags: vec!["benchmark".to_string()],
        metadata: HashMap::new(),

        // Protocol configuration
        protocols: None,

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
        #[cfg(feature = "metrics")]
        metrics: None,
        #[cfg(feature = "realtime")]
        realtime: None,
    }
}

fn benchmark_scan_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("scan_performance");
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(3));

    // Get configurable signal/block counts
    let (signal_counts, block_counts) = get_benchmark_config();

    // Print configuration for debugging
    eprintln!("Benchmark configuration:");
    for (signals, blocks) in signal_counts.iter().zip(block_counts.iter()) {
        eprintln!("  {} signals, {} blocks", signals, blocks);
    }

    // Create runtime for async operations
    let rt = Runtime::new().unwrap();

    // Test different scales using configured values
    for (num_signals, num_blocks) in signal_counts.iter().zip(block_counts.iter()) {
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

    // Get largest signal count for stress testing
    let (signal_counts, _) = get_benchmark_config();
    let max_signals = signal_counts.iter().max().unwrap_or(&1000);

    // Scale signal operations based on configuration
    let test_signals = (*max_signals).min(10000); // Cap at 10k for reasonable test times

    // Benchmark signal writes
    group.bench_function(&format!("write_{}_signals", test_signals), |b| {
        let bus = SignalBus::new();

        b.iter(|| {
            for i in 0..test_signals {
                let _ = bus.set(&format!("signal_{}", i), Value::Float(black_box(i as f64)));
            }
        });
    });

    // Benchmark signal reads
    group.bench_function(&format!("read_{}_signals", test_signals), |b| {
        let bus = SignalBus::new();

        // Pre-populate signals
        for i in 0..test_signals {
            let _ = bus.set(&format!("signal_{}", i), Value::Float(i as f64));
        }

        b.iter(|| {
            for i in 0..test_signals {
                let _ = black_box(bus.get(&format!("signal_{}", i)));
            }
        });
    });

    // Benchmark atomic updates (scaled down for performance)
    let update_signals = (test_signals / 10).max(100);
    group.bench_function(&format!("atomic_update_{}_signals", update_signals), |b| {
        let bus = SignalBus::new();

        // Pre-populate
        for i in 0..update_signals {
            let _ = bus.set(&format!("counter_{}", i), Value::Integer(0));
        }

        b.iter(|| {
            for i in 0..update_signals {
                let _ = bus.update(&format!("counter_{}", i), |old| match old {
                    Some(Value::Integer(n)) => Value::Integer(n + 1),
                    _ => Value::Integer(1),
                });
            }
        });
    });

    group.finish();
}

fn benchmark_block_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_execution");

    // Test different block counts
    let (_, block_counts) = get_benchmark_config();

    for &num_blocks in &block_counts {
        if num_blocks > 10000 {
            continue; // Skip very large block counts for individual block tests
        }

        group.bench_function(&format!("execute_{}_blocks", num_blocks), |b| {
            let config = create_benchmark_config(num_blocks * 2, num_blocks);
            let engine = Engine::new(config).expect("Failed to create engine");
            let rt = Runtime::new().unwrap();

            b.iter(|| {
                rt.block_on(async {
                    let _ = black_box(engine.execute_scan_cycle().await);
                });
            });
        });
    }

    group.finish();
}

fn benchmark_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_operations");

    // These don't scale with signal count, so keep them consistent
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
            let _ = black_box(float_val.as_float());
            let _ = black_box(int_val.as_integer());
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

// Add a simple test benchmark for quick validation
fn benchmark_simple_test(c: &mut Criterion) {
    c.bench_function("simple_test", |b| {
        b.iter_with_setup(
            || {
                let bus = SignalBus::new();
                let _ = bus.set("test_signal", Value::Float(42.0));
                bus
            },
            |bus| {
                let _ = black_box(bus.get("test_signal"));
            },
        );
    });
}

// Add a feature diagnostic benchmark
fn benchmark_feature_diagnostic(c: &mut Criterion) {
    c.bench_function("feature_diagnostic", |b| {
        b.iter(|| {
            let mut result = 0u32;

            #[cfg(feature = "enhanced-monitoring")]
            {
                result += black_box(1);
            }

            #[cfg(feature = "optimized")]
            {
                result += black_box(2);
            }

            #[cfg(feature = "extended-types")]
            {
                result += black_box(4);
            }

            let value = black_box(Value::Float(result as f64 + 1.0));
            std::hint::black_box(value);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5))
        .with_output_color(true)
        .with_measurement(WallTime);
    targets =
        benchmark_simple_test,
        benchmark_feature_diagnostic,
        benchmark_scan_performance,
        benchmark_signal_bus_operations,
        benchmark_block_execution,
        benchmark_value_operations
}

criterion_main!(benches);

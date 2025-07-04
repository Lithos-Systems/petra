use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use petra::*;

fn benchmark_parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_execution");

    for block_count in [10, 50, 100, 500, 1000].iter() {
        // Sequential execution
        group.bench_with_input(
            BenchmarkId::new("sequential", block_count),
            block_count,
            |b, &count| {
                let config = create_benchmark_config_sequential(count);
                let engine = Engine::new(config).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        let _ = black_box(engine.execute_scan_cycle().await);
                    });
                });
            },
        );

        // Parallel execution
        group.bench_with_input(
            BenchmarkId::new("parallel", block_count),
            block_count,
            |b, &count| {
                let config = create_benchmark_config_parallel(count);
                let engine = Engine::new(config).unwrap();
                let rt = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    rt.block_on(async {
                        let _ = black_box(engine.execute_scan_cycle().await);
                    });
                });
            },
        );
    }

    group.finish();
}

fn benchmark_simd_math(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_math");

    for array_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("standard_add", array_size),
            array_size,
            |b, &size| {
                let arrays = create_float_arrays(size);
                b.iter(|| standard_array_add(&arrays.0, &arrays.1));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("simd_add", array_size),
            array_size,
            |b, &size| {
                let arrays = create_float_arrays(size);
                b.iter(|| simd_array_add(&arrays.0, &arrays.1));
            },
        );
    }

    group.finish();
}
criterion_group!(
    benches,
    benchmark_parallel_vs_sequential,
    benchmark_simd_math
);
criterion_main!(benches);

fn create_benchmark_config_sequential(num_blocks: usize) -> petra::config::Config {
    use std::collections::HashMap;

    let num_signals = num_blocks * 3;
    let mut signals = Vec::new();
    let mut blocks = Vec::new();

    for i in 0..num_signals {
        signals.push(petra::config::SignalConfig {
            name: format!("seq_signal_{}", i),
            signal_type: "float".to_string(),
            description: Some(format!("Sequential signal {}", i)),
            initial: Some(serde_yaml::Value::from(0.0f64)),
            category: Some("Sequential".to_string()),
            source: Some("Benchmark".to_string()),
            update_frequency_ms: Some(50),
            tags: vec!["sequential".to_string()],
            metadata: HashMap::new(),
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

    for i in 0..num_blocks {
        let mut inputs = HashMap::new();
        inputs.insert("in1".to_string(), format!("seq_signal_{}", i * 2));
        inputs.insert("in2".to_string(), format!("seq_signal_{}", i * 2 + 1));

        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), format!("seq_signal_{}", num_signals + i));

        blocks.push(petra::config::BlockConfig {
            name: format!("and_block_{}", i),
            block_type: "AND".to_string(),
            inputs,
            outputs,
            params: HashMap::new(),
            priority: 0,
            enabled: true,
            description: Some(format!("Sequential block {}", i)),
            category: None,
            tags: vec![],
            metadata: HashMap::new(),
            #[cfg(feature = "enhanced-errors")]
            error_handling: None,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
            #[cfg(feature = "enhanced-monitoring")]
            enhanced_monitoring: false,
        });
    }

    petra::config::Config {
        signals,
        blocks,
        scan_time_ms: 50,
        max_scan_jitter_ms: 25,
        error_recovery: true,
        max_consecutive_errors: 10,
        restart_delay_ms: 1000,
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
        #[cfg(feature = "validation")]
        validation: None,
        #[cfg(feature = "metrics")]
        metrics: None,
        #[cfg(feature = "realtime")]
        realtime: None,
        version: "1.0".to_string(),
        description: None,
        author: None,
        created_at: None,
        modified_at: None,
        tags: Vec::new(),
        metadata: HashMap::new(),
    }
}

fn create_benchmark_config_parallel(num_blocks: usize) -> petra::config::Config {
    // For simplicity, reuse the sequential configuration
    create_benchmark_config_sequential(num_blocks)
}

fn create_float_arrays(size: usize) -> (Vec<f32>, Vec<f32>) {
    let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..size).map(|i| i as f32).collect();
    (a, b)
}

fn standard_array_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

fn simd_array_add(a: &[f32], b: &[f32]) -> Vec<f32> {
    // Placeholder SIMD implementation; simple loop for now
    a.iter().zip(b.iter()).map(|(x, y)| x + y).collect()
}

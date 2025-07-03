//! Engine performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use petra::{Config, Engine, SignalBus, Value};
use std::time::Duration;

fn create_test_config(num_blocks: usize) -> Config {
    let mut config = Config {
        scan_time_ms: 10,
        max_scan_jitter_ms: 50,
        error_recovery: true,
        signals: Vec::new(),
        blocks: Vec::new(),
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
    };
    
    // Add signals
    for i in 0..num_blocks * 2 {
        config.signals.push(petra::config::SignalConfig {
            name: format!("signal_{}", i),
            signal_type: "bool".to_string(),
            initial: Some(serde_yaml::Value::Bool(false)),
            description: Some(format!("Test signal {}", i)),
            tags: Vec::new(),
            #[cfg(feature = "engineering-types")]
            units: None,
            #[cfg(feature = "quality-codes")]
            quality_enabled: false,
            #[cfg(feature = "validation")]
            validation: None,
            metadata: Default::default(),
        });
    }
    
    // Add AND blocks
    for i in 0..num_blocks {
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("in1".to_string(), format!("signal_{}", i * 2));
        inputs.insert("in2".to_string(), format!("signal_{}", i * 2 + 1));

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("out".to_string(), format!("output_{}", i));

        config.blocks.push(petra::config::BlockConfig {
            name: format!("and_block_{}", i),
            block_type: "AND".to_string(),
            inputs,
            outputs,
            params: std::collections::HashMap::new(),
            description: Some(format!("Test AND block {}", i)),
            tags: Vec::new(),
            #[cfg(feature = "enhanced-errors")]
            error_handling: None,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
        });
    }
    
    config
}

fn benchmark_engine_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_scan");
    
    for block_count in [10, 50, 100, 500].iter() {
        group.bench_function(format!("{}_blocks", block_count), |b| {
            let config = create_test_config(*block_count);
            let engine = Engine::new(config).expect("Failed to create engine");
            
            b.iter(|| {
                black_box(engine.scan_once());
            });
        });
    }
    
    group.finish();
}

fn benchmark_signal_bus(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_bus");
    
    group.bench_function("write_1000_signals", |b| {
        let bus = SignalBus::new();
        
        b.iter(|| {
            for i in 0..1000 {
                bus.write(&format!("signal_{}", i), Value::Float(i as f64));
            }
        });
    });
    
    group.bench_function("read_1000_signals", |b| {
        let bus = SignalBus::new();
        
        // Pre-populate signals
        for i in 0..1000 {
            bus.write(&format!("signal_{}", i), Value::Float(i as f64));
        }
        
        b.iter(|| {
            for i in 0..1000 {
                black_box(bus.read(&format!("signal_{}", i)));
            }
        });
    });
    
    group.bench_function("concurrent_access", |b| {
        use std::sync::Arc;
        use std::thread;
        
        let bus = Arc::new(SignalBus::new());
        
        b.iter(|| {
            let mut handles = vec![];
            
            for thread_id in 0..4 {
                let bus_clone = Arc::clone(&bus);
                let handle = thread::spawn(move || {
                    for i in 0..250 {
                        let signal_id = thread_id * 250 + i;
                        bus_clone.write(
                            &format!("signal_{}", signal_id),
                            Value::Float(signal_id as f64),
                        );
                    }
                });
                handles.push(handle);
            }
            
            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
    
    group.finish();
}

fn benchmark_block_execution(c: &mut Criterion) {
    use petra::blocks::Block;
    
    let mut group = c.benchmark_group("block_execution");
    
    group.bench_function("and_block", |b| {
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("in1".to_string(), "in1".to_string());
        inputs.insert("in2".to_string(), "in2".to_string());

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("out".to_string(), "out".to_string());

        let config = petra::config::BlockConfig {
            name: "test_and".to_string(),
            block_type: "AND".to_string(),
            inputs,
            outputs,
            params: std::collections::HashMap::new(),
            description: Some(String::new()),
            tags: Vec::new(),
            #[cfg(feature = "enhanced-errors")]
            error_handling: None,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
        };
        
        let mut block = petra::blocks::create_block(&config)
            .expect("Failed to create block");
        let bus = SignalBus::new();
        
        bus.write("in1", Value::Bool(true));
        bus.write("in2", Value::Bool(true));
        
        b.iter(|| {
            black_box(block.execute(&bus));
        });
    });
    
    #[cfg(feature = "pid-control")]
    group.bench_function("pid_block", |b| {
        let mut params = std::collections::HashMap::new();
        params.insert("kp".to_string(), serde_json::json!(1.0));
        params.insert("ki".to_string(), serde_json::json!(0.1));
        params.insert("kd".to_string(), serde_json::json!(0.01));
        params.insert("setpoint".to_string(), serde_json::json!(100.0));
        
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("pv".to_string(), "process_value".to_string());

        let mut outputs = std::collections::HashMap::new();
        outputs.insert("cv".to_string(), "control_output".to_string());

        let config = petra::config::BlockConfig {
            name: "test_pid".to_string(),
            block_type: "PID".to_string(),
            inputs,
            outputs,
            params,
            description: Some(String::new()),
            tags: Vec::new(),
            #[cfg(feature = "enhanced-errors")]
            error_handling: None,
            #[cfg(feature = "circuit-breaker")]
            circuit_breaker: None,
        };
        
        let mut block = petra::blocks::create_block(&config)
            .expect("Failed to create block");
        let bus = SignalBus::new();
        
        bus.write("process_value", Value::Float(95.0));
        
        b.iter(|| {
            black_box(block.execute(&bus));
        });
    });
    
    group.finish();
}

fn benchmark_value_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_operations");
    
    group.bench_function("value_creation", |b| {
        b.iter(|| {
            black_box(Value::Float(42.0));
            black_box(Value::Int(42));
            black_box(Value::Bool(true));
        });
    });
    
    group.bench_function("value_conversion", |b| {
        let float_val = Value::Float(42.5);
        let int_val = Value::Int(42);
        let bool_val = Value::Bool(true);
        
        b.iter(|| {
            black_box(float_val.as_float());
            black_box(int_val.as_int());
            black_box(bool_val.as_bool());
        });
    });
    
    #[cfg(feature = "value-arithmetic")]
    group.bench_function("value_arithmetic", |b| {
        let val1 = Value::Float(10.0);
        let val2 = Value::Float(20.0);
        
        b.iter(|| {
            black_box(val1.add(&val2));
            black_box(val1.multiply(&val2));
        });
    });
    
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(5));
    targets = benchmark_engine_scan, benchmark_signal_bus, benchmark_block_execution, benchmark_value_operations
}

criterion_main!(benches);

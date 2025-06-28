use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use petra::{Config, Engine, SignalBus, Value};
use petra::config::{SignalConfig, BlockConfig, SignalType, InitialValue};
use std::collections::HashMap;

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
    
    // Test different scales
    for (num_signals, num_blocks) in &[(100, 10), (1000, 100), (10000, 1000)] {
        let config = create_benchmark_config(*num_signals, *num_blocks);
        let mut engine = Engine::new(config).expect("Failed to create engine");
        
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
    let bus = SignalBus::new();
    
    // Pre-populate signals
    for i in 0..1000 {
        bus.write_signal(&format!("signal_{}", i), Value::Float(0.0)).unwrap();
    }
    
    group.bench_function("write_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            bus.write_signal(
                &format!("signal_{}", counter % 1000), 
                Value::Float(black_box(counter as f64))
            ).unwrap();
            counter += 1;
        });
    });
    
    group.bench_function("read_single", |b| {
        let mut counter = 0;
        b.iter(|| {
            let _ = bus.read_signal(&format!("signal_{}", counter % 1000)).unwrap();
            counter += 1;
        });
    });
    
    group.bench_function("batch_write_10", |b| {
        b.iter(|| {
            let batch: Vec<(&str, Value)> = (0..10)
                .map(|i| {
                    let signal = format!("signal_{}", i);
                    (signal.as_str(), Value::Float(i as f64))
                })
                .collect();
            bus.write_batch(batch).unwrap();
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
    
    // AND block
    let mut and_block = AndBlock::new(
        "test_and".to_string(),
        vec!["input1".to_string(), "input2".to_string()],
        "output".to_string(),
    );
    
    group.bench_function("and_block", |b| {
        b.iter(|| {
            and_block.execute(&bus).unwrap();
        });
    });
    
    // PID block with more complex calculations
    bus.write_signal("setpoint", Value::Float(100.0)).unwrap();
    bus.write_signal("process_value", Value::Float(95.0)).unwrap();
    bus.write_signal("pid_output", Value::Float(0.0)).unwrap();
    
    let mut pid_block = PidBlock {
        name: "test_pid".to_string(),
        setpoint: "setpoint".to_string(),
        process_value: "process_value".to_string(),
        output: "pid_output".to_string(),
        kp: 1.0,
        ki: 0.1,
        kd: 0.01,
        integral: 0.0,
        last_error: 0.0,
        output_min: -100.0,
        output_max: 100.0,
    };
    
    group.bench_function("pid_block", |b| {
        b.iter(|| {
            pid_block.execute(&bus).unwrap();
        });
    });
    
    group.finish();
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
    
    group.bench_function("enhanced_write_hot_signal", |b| {
        // This signal should become "hot" and cached
        b.iter(|| {
            bus.write_signal("hot_signal", Value::Float(black_box(42.0))).unwrap();
        });
    });
    
    group.bench_function("enhanced_read_hot_signal", |b| {
        // Pre-heat the signal
        for _ in 0..200 {
            let _ = bus.read_signal("hot_signal");
        }
        
        b.iter(|| {
            let _ = bus.read_signal("hot_signal").unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(
    benches, 
    benchmark_scan_performance,
    benchmark_signal_bus_operations,
    benchmark_block_execution,
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

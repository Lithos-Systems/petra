use criterion::{black_box, criterion_group, criterion_main, Criterion};
use petra::{SignalBus, Value};
use std::time::Duration;

fn benchmark_signal_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signal_operations");
    let bus = SignalBus::new();

    // Pre-populate some signals
    for i in 0..100 {
        let _ = bus.set(&format!("test_signal_{}", i), Value::Float(i as f64));
    }

    group.bench_function("signal_read", |b| {
        b.iter(|| {
            let _ = black_box(bus.get("test_signal_1"));
        });
    });

    group.bench_function("signal_write", |b| {
        b.iter(|| {
            let _ = black_box(bus.set("benchmark_signal", Value::Float(42.0)));
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_signal_operations);
criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Once;

mod features;

static PRINT_FEATURES: Once = Once::new();

fn print_features_once() {
    PRINT_FEATURES.call_once(features::print_enabled_features);
}

// Basic benchmark that always works
fn simple_test(c: &mut Criterion) {
    print_features_once();
    c.bench_function("simple_test", |b| {
        b.iter(|| {
            let x = 2 + 2;
            black_box(x);
        });
    });
}

// PETRA-specific benchmarks
fn petra_value_benchmark(c: &mut Criterion) {
    print_features_once();
    use petra::Value;

    c.bench_function("value_creation", |b| {
        b.iter(|| {
            let val = Value::Float(42.0);
            black_box(val);
        });
    });
}

fn signal_bus_benchmark(c: &mut Criterion) {
    print_features_once();
    use petra::{SignalBus, Value};

    c.bench_function("signal_operations", |b| {
        let bus = SignalBus::new();
        b.iter(|| {
            let _ = bus.set("test", Value::Float(1.0));
            let val = bus.get("test").unwrap_or(Value::Float(0.0));
            black_box(val);
        });
    });
}

// Diagnostic benchmark for feature visibility
fn diagnostic_benchmark(c: &mut Criterion) {
    print_features_once();
    c.bench_function("feature_diagnostic", |b| {
        b.iter(|| {
            #[cfg(feature = "extended-types")]
            let extended = true;
            #[cfg(not(feature = "extended-types"))]
            let extended = false;

            black_box(extended);
        });
    });
}

// Configure benchmark groups depending on available features
criterion_group!(
    petra_benches,
    simple_test,
    petra_value_benchmark,
    signal_bus_benchmark,
    diagnostic_benchmark
);

criterion_main!(petra_benches);

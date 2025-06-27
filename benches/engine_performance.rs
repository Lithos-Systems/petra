// benches/engine_performance.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_scan_performance(c: &mut Criterion) {
    c.bench_function("scan_1000_signals", |b| {
        let config = create_benchmark_config(1000, 100);
        let mut engine = Engine::new(config).unwrap();
        
        b.iter(|| {
            // Measure single scan cycle
            engine.execute_scan_cycle()
        });
    });
}

criterion_group!(benches, benchmark_scan_performance);
criterion_main!(benches);

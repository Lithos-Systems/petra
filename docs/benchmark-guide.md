The repository contains a small Criterion-based benchmark that targets the engine’s scan cycle. The benchmark is defined in benches/engine_performance.rs:

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

This benchmark sets up an engine with a configurable number of signals and repeatedly calls execute_scan_cycle() to measure performance.

The README documents how to run all benchmarks using Cargo’s built-in tooling:

### Performance & Monitoring
```bash
# Run benchmarks
cargo bench

# Enable detailed metrics
RUST_LOG=petra=debug cargo run --release -- config.yaml

# Memory profiling (with pprof feature)
cargo run --release --features pprof -- config.yaml

# Monitor with custom scan times
cargo run --release -- config.yaml --scan-time 50

These instructions show the standard way to run Criterion benches (cargo bench), collect runtime metrics, and optionally perform memory profiling with the optional pprof feature.
How Benchmarks Relate to Engine Metrics

Inside the engine loop, metrics are recorded for scan duration, jitter, and overruns:

gauge!("petra_scan_jitter_avg_us").set(avg_jitter.as_micros() as f64);
gauge!("petra_scan_jitter_max_us").set(max_jitter.as_micros() as f64);
gauge!("petra_scan_variance_us").set(jitter.as_micros() as f64);
histogram!("petra_scan_duration_seconds").record(scan_duration.as_secs_f64());
gauge!("petra_scan_duration_ms").set(scan_duration.as_millis() as f64);
counter!("petra_scan_overruns_total").increment(1);

These metrics integrate with Prometheus (via metrics-exporter-prometheus) so you can observe performance trends while running the benchmarks or the application itself.
Running Benchmarks

    Ensure dependencies – cargo bench will fetch Criterion and other crates as needed. Compilation may take a while on the first run because of the graphical and network-related dependencies listed in Cargo.toml.

    Execute the benchmark – From the repository root, run:

cargo bench

This will build the benchmark target and execute the function defined in benches/engine_performance.rs. Criterion produces detailed output and saves reports (HTML by default) in the target/criterion directory.

Examine reports – Open the generated HTML report (e.g., target/criterion/scan_1000_signals/report/index.html) in a browser to view graphs and statistics for scan cycle timing.

Tune metrics – If you want more verbose logging or to collect additional performance data, use the environment variable shown in the README:

RUST_LOG=petra=debug cargo run --release -- config.yaml

This can be combined with the benchmark to inspect scanning behavior under different configurations.

Optional profiling – For memory profiling or CPU flame graphs, enable the pprof feature during a normal run:

    cargo run --release --features pprof -- config.yaml

    The pprof output can be viewed to understand allocation hotspots or CPU usage during benchmark runs.

Notes on Current Benchmark State

    The benchmark references a helper function create_benchmark_config and an Engine::execute_scan_cycle() method. These may not be implemented in the repository; ensure they exist or provide stub implementations before running cargo bench.

    Some dependencies require network access (e.g., for crates.io). During this analysis, fetching crates triggered numerous downloads, illustrating the compilation cost of optional features.

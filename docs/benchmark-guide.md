# Benchmarks & Performance Guide

The repository contains a small **Criterion-based benchmark** that measures the engine’s scan-cycle performance.  
Benchmark definition: **`benches/engine_performance.rs`**

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_scan_performance(c: &mut Criterion) {
    c.bench_function("scan_1000_signals", |b| {
        let config = create_benchmark_config(1_000, 100);
        let mut engine = Engine::new(config).unwrap();

        b.iter(|| {
            // Measure a single scan-cycle
            engine.execute_scan_cycle();
        });
    });
}

criterion_group!(benches, benchmark_scan_performance);
criterion_main!(benches);
````

The benchmark creates an engine with a **configurable number of signals** and repeatedly calls
`execute_scan_cycle()` to gather timing statistics.

---

## Running the Benchmarks

1. **Compile & execute**

   ```bash
   cargo bench
   ```

2. **Open the report**

   Criterion stores HTML reports under
   `target/criterion/<benchmark-name>/report/index.html`.

3. **(Optional) verbose logging**

   ```bash
   RUST_LOG=petra=debug cargo run --release -- config.yaml
   ```

4. **(Optional) memory / CPU profiling**

   ```bash
   cargo run --release --features pprof -- config.yaml
   ```

---

## Performance & Monitoring Commands

```bash
# Standard Criterion run
cargo bench

# Run Petra with detailed runtime metrics
RUST_LOG=petra=debug cargo run --release -- config.yaml

# Enable pprof feature for flame-graphs / heap profiles
cargo run --release --features pprof -- config.yaml

# Override configured scan-time (example: 50 ms)
cargo run --release -- config.yaml --scan-time 50
```

---

## How Benchmarks Relate to Engine Metrics

Inside the engine loop, several Prometheus-compatible metrics are recorded:

```rust
gauge!("petra_scan_jitter_avg_us").set(avg_jitter.as_micros() as f64);
gauge!("petra_scan_jitter_max_us").set(max_jitter.as_micros() as f64);
gauge!("petra_scan_variance_us").set(jitter.as_micros() as f64);
histogram!("petra_scan_duration_seconds").record(scan_duration.as_secs_f64());
gauge!("petra_scan_duration_ms").set(scan_duration.as_millis() as f64);
counter!("petra_scan_overruns_total").increment(1);
```

With the bundled **`metrics-exporter-prometheus`** backend you can observe scan duration, jitter, and overrun counts in real time while running benchmarks or production code.

---

## Notes & Caveats

* The benchmark depends on `create_benchmark_config` and `Engine::execute_scan_cycle()`.
  Ensure these exist (or stub them) before running `cargo bench`.

* The first run may be slow because Criterion pulls and compiles many crates.

* Some optional crates require network access at build time―plan for extra compile time in CI.

---

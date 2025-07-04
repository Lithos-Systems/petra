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

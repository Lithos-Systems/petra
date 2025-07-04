use criterion::{criterion_group, criterion_main, Criterion};

fn minimal_benchmark(c: &mut Criterion) {
    c.bench_function("simple_test", |b| {
        b.iter(|| {
            let x = 2 + 2;
            criterion::black_box(x);
        });
    });
}

criterion_group!(benches, minimal_benchmark);
criterion_main!(benches);

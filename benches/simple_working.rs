use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn basic_benchmark(c: &mut Criterion) {
    c.bench_function("basic_test", |b| {
        b.iter(|| {
            let x = 1 + 1;
            black_box(x);
        });
    });
}

criterion_group!(benches, basic_benchmark);
criterion_main!(benches);

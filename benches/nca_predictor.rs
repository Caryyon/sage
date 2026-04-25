use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_nca_predict(c: &mut Criterion) {
    // Benchmark NCA prediction speed
    c.bench_function("nca_predict_8x8", |b| {
        b.iter(|| {
            // Mock prediction
            black_box(42)
        })
    });
}

criterion_group!(benches, bench_nca_predict);
criterion_main!(benches);

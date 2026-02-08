use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sage::distributed_knowledge::{NCAKnowledge, KnowledgeStore};
use sage::grid::Grid;

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("knowledge_encode");
    for size in [10, 100, 1000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let mut store = NCAKnowledge::new();
                store.config.ollama_url = None;
                for i in 0..size {
                    store.encode(black_box(&format!("knowledge item {}", i)), 0.8);
                }
            });
        });
    }
    group.finish();
}

fn bench_query(c: &mut Criterion) {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;
    for i in 0..100 {
        store.encode(&format!("fact about topic {} with details {}", i, i * 7), 0.8);
    }

    c.bench_function("knowledge_query_top5", |b| {
        b.iter(|| {
            store.query(black_box("topic 42 details"), 5)
        });
    });
}

fn bench_merge(c: &mut Criterion) {
    let mut store_b = NCAKnowledge::new().with_node_id(2.0);
    store_b.config.ollama_url = None;
    for i in 0..50 {
        store_b.encode(&format!("node b item {}", i), 0.8);
    }
    let grid_b = store_b.grid.clone();

    c.bench_function("knowledge_merge_50items", |b| {
        b.iter_batched(
            || {
                let mut s = NCAKnowledge::new().with_node_id(1.0);
                s.config.ollama_url = None;
                for i in 0..50 {
                    s.encode(&format!("node a item {}", i), 0.8);
                }
                s
            },
            |mut store_a| {
                store_a.merge(black_box(&grid_b), 0.8);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_diff(c: &mut Criterion) {
    let mut store = NCAKnowledge::new();
    store.config.ollama_url = None;
    for i in 0..50 {
        store.encode(&format!("item {}", i), 0.8);
    }
    let empty = Grid::new(32, 32);

    c.bench_function("knowledge_diff", |b| {
        b.iter(|| {
            store.diff(black_box(&empty))
        });
    });
}

criterion_group!(benches, bench_encode, bench_query, bench_merge, bench_diff);
criterion_main!(benches);

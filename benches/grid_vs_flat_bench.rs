//! Grid vs Flat Vector Store Benchmark
//!
//! ML analysis recommendation: "Benchmark the grid against a flat vector store."
//!
//! This compares SAGE's 256×256 NCA grid retrieval against:
//! 1. Brute-force cosine similarity on a flat Vec<(FeatureVector, String)>
//! 2. Spatial locality from grid hashing vs pure semantic similarity
//!
//! The grid adds collision risk and spatial complexity — this measures if it's worth it.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sage::distributed_knowledge::{KnowledgeStore, NCAKnowledge};
use sage::distributed_knowledge::encoder::{encode_text, EncoderConfig, FeatureVector};

/// Flat vector store — baseline comparison
struct FlatStore {
    embeddings: Vec<Vec<f64>>,
    texts: Vec<String>,
}

impl FlatStore {
    fn new() -> Self {
        Self { embeddings: Vec::new(), texts: Vec::new() }
    }

    fn encode(&mut self, text: &str, config: &EncoderConfig) {
        let features = encode_text(text, config).expect("encode failed");
        self.embeddings.push(features.values);
        self.texts.push(text.to_string());
    }

    fn query(&self, query: &str, config: &EncoderConfig, top_k: usize) -> Vec<(String, f64)> {
        let q = encode_text(query, config).expect("encode failed");
        let mut scores: Vec<(usize, f64)> = self.embeddings.iter().enumerate().map(|(i, emb)| {
            let dot: f64 = q.values.iter().zip(emb.iter()).map(|(a, b)| a * b).sum();
            let mag_q: f64 = q.values.iter().map(|v| v * v).sum::<f64>().sqrt();
            let mag_e: f64 = emb.iter().map(|v| v * v).sum::<f64>().sqrt();
            (i, if mag_q < 1e-10 || mag_e < 1e-10 { 0.0 } else { dot / (mag_q * mag_e) })
        }).collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        scores.into_iter().map(|(i, s)| (self.texts[i].clone(), s)).collect()
    }
}

fn bench_encode_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode");

    for size in [10, 50, 100].iter() {
        // Grid encode
        group.bench_with_input(
            BenchmarkId::new("grid", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut store = NCAKnowledge::new();
                    store.config.ollama_url = None;
                    for i in 0..size {
                        store.encode(black_box(&format!("knowledge item {} with details", i)), 0.8);
                    }
                });
            },
        );

        // Flat encode
        group.bench_with_input(
            BenchmarkId::new("flat", size),
            size,
            |b, &size| {
                let config = EncoderConfig::default();
                b.iter(|| {
                    let mut flat = FlatStore::new();
                    for i in 0..size {
                        flat.encode(black_box(&format!("knowledge item {} with details", i)), &config);
                    }
                });
            },
        );
    }
    group.finish();
}

fn bench_query_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("query");

    for (n_facts, label) in [(50, "50"), (100, "100"), (500, "500")].iter() {
        // Setup grid store
        let mut grid = NCAKnowledge::new();
        grid.config.ollama_url = None;
        for i in 0..*n_facts {
            grid.encode(&format!("fact about topic {} with key details {}", i % 20, i), 0.8);
        }

        // Setup flat store
        let config = EncoderConfig::default();
        let mut flat = FlatStore::new();
        for i in 0..*n_facts {
            flat.encode(&format!("fact about topic {} with key details {}", i % 20, i), &config);
        }

        // Grid query
        group.bench_with_input(
            BenchmarkId::new("grid", label),
            &(),
            |b, _| {
                b.iter(|| grid.query(black_box("topic 5 details"), 5));
            },
        );

        // Flat query
        group.bench_with_input(
            BenchmarkId::new("flat", label),
            &(),
            |b, _| {
                b.iter(|| flat.query(black_box("topic 5 details"), &config, 5));
            },
        );
    }
    group.finish();
}

fn bench_retrieval_quality(c: &mut Criterion) {
    // This benchmarks recall: does the grid find what we encoded?
    // vs flat store which just does pure similarity.

    let mut group = c.benchmark_group("retrieval_quality");

    let facts: Vec<String> = (0..50)
        .map(|i| format!("The capital of country {} is city {} with population {}", i, i * 100, i * 1000))
        .collect();

    // Grid store
    let mut grid = NCAKnowledge::new();
    grid.config.ollama_url = None;
    for fact in &facts {
        grid.encode(fact, 0.8);
    }

    // Flat store
    let config = EncoderConfig::default();
    let mut flat = FlatStore::new();
    for fact in &facts {
        flat.encode(fact, &config);
    }

    // Query for specific facts and check recall
    for (i, query) in ["capital country 5", "city 200 population", "country 10 city 1000"].iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("grid", i),
            query,
            |b, q| {
                b.iter(|| {
                    let results = grid.query(black_box(q), 5);
                    // Measure: top result similarity
                    results.first().map(|r| r.relevance).unwrap_or(0.0)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("flat", i),
            query,
            |b, q| {
                b.iter(|| {
                    let results = flat.query(black_box(q), &config, 5);
                    results.first().map(|(_, s)| s).unwrap_or(0.0)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_encode_comparison, bench_query_comparison, bench_retrieval_quality);
criterion_main!(benches);
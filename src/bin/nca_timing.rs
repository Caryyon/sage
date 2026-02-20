//! Quick NCA inference timing benchmark
use sage::inference::nca_predictor::*;
use std::time::Instant;

fn main() {
    let w = NcaWeights::random();
    eprintln!(
        "NCA params: {} ({:.1} KB)",
        w.param_count(),
        w.param_count() as f64 * 8.0 / 1024.0
    );

    for grid_size in [16, 32, 48, 64] {
        let corpus = "the quick brown fox jumps over the lazy dog and the cat sat on the mat the quick brown fox";
        let tok = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
        let w2 = w.clone();
        // 1 NCA step per run_and_read call
        let mut pred = NcaPredictor::with_grid_size(tok, w2, 1, grid_size);
        let input = pred.tokenizer.encode("the quick brown");

        // Warmup
        pred.run_and_read(&input);

        let n = 20;
        let start = Instant::now();
        for _ in 0..n {
            pred.run_and_read(&input);
        }
        let per_step_ms = start.elapsed().as_secs_f64() * 1000.0 / n as f64;

        eprintln!(
            "{}×{}: {:.2} ms/step | 5 steps={:.0}ms | Pi4≈{:.0}ms (5 steps)",
            grid_size,
            grid_size,
            per_step_ms,
            per_step_ms * 5.0,
            per_step_ms * 5.0 * 10.0
        );
    }
}

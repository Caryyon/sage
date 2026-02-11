//! Criticality Measurement and Regularization for NCA Dynamics
//!
//! Based on: Pontes-Filho et al. (2025), "Reservoir Computing with Critical NCA"
//!
//! NCA grids operating at the "edge of chaos" — the boundary between ordered
//! and chaotic dynamics — have maximum computational capacity. This module
//! provides tools to measure and optimize for criticality.
//!
//! Key metrics:
//! - **Branching ratio**: avg cells affected by one cell's change. Critical = 1.0
//! - **Lyapunov exponent**: divergence rate of nearby trajectories. Critical ≈ 0.0
//! - **Power law exponent**: avalanche size distribution slope. Critical τ ≈ 1.5
//! - **Criticality score**: composite 0–1 where 1.0 = perfectly critical

use crate::inference::nca_predictor::{NcaPredictor, NcaWeights, SimpleTokenizer, NCA_CHANNELS};
use rand::Rng;
use std::error::Error;

// ---------------------------------------------------------------------------
// Avalanche Statistics
// ---------------------------------------------------------------------------

/// Distribution of avalanche sizes from perturbation experiments.
#[derive(Clone, Debug)]
pub struct AvalancheStats {
    /// Raw avalanche sizes (number of cells that changed significantly)
    pub sizes: Vec<usize>,
    /// Histogram: bin index → count (bin i = avalanche size i)
    pub histogram: Vec<usize>,
    /// Maximum avalanche size observed
    pub max_size: usize,
}

impl AvalancheStats {
    /// Render a simple ASCII histogram of avalanche sizes.
    pub fn ascii_histogram(&self, max_width: usize) -> String {
        if self.histogram.is_empty() {
            return String::from("  (no data)");
        }
        let _max_count = *self.histogram.iter().max().unwrap_or(&1);
        let mut lines = Vec::new();
        // Bin into ~20 buckets for readability
        let n_buckets = 20.min(self.max_size + 1);
        if n_buckets == 0 {
            return String::from("  (no avalanches)");
        }
        let bucket_size = ((self.max_size + 1) as f64 / n_buckets as f64).ceil() as usize;
        let mut buckets = vec![0usize; n_buckets];
        for (size, &count) in self.histogram.iter().enumerate() {
            let b = (size / bucket_size.max(1)).min(n_buckets - 1);
            buckets[b] += count;
        }
        let bmax = *buckets.iter().max().unwrap_or(&1).max(&1);
        for (i, &count) in buckets.iter().enumerate() {
            let lo = i * bucket_size;
            let hi = ((i + 1) * bucket_size).min(self.max_size + 1) - 1;
            let bar_len = (count as f64 / bmax as f64 * max_width as f64).round() as usize;
            let bar: String = "█".repeat(bar_len);
            lines.push(format!("  {:>4}-{:<4} │{} ({})", lo, hi, bar, count));
        }
        lines.join("\n")
    }
}

/// Measure avalanche size distribution by perturbing settled NCA grids.
///
/// 1. Activate some tokens, run NCA to settle
/// 2. Clone the grid state
/// 3. Perturb a single random cell by `perturbation_size`
/// 4. Run both grids for `propagation_steps` more steps
/// 5. Count cells where |perturbed - original| > threshold → avalanche size
/// 6. Repeat `n_samples` times
pub fn measure_avalanche_distribution(
    predictor: &mut NcaPredictor,
    input_tokens: &[usize],
    n_samples: usize,
    perturbation_size: f64,
    propagation_steps: usize,
    threshold: f64,
) -> AvalancheStats {
    let mut rng = rand::thread_rng();
    let gs = predictor.grid_size();
    let mut sizes = Vec::with_capacity(n_samples);

    for _ in 0..n_samples {
        // Run NCA to get settled state
        let settled = predictor.run_and_get_state(input_tokens);

        // Clone and perturb one random cell
        let mut perturbed_grid = settled.clone();
        let pr = rng.gen_range(0..gs);
        let pc = rng.gen_range(0..gs);
        let ch = rng.gen_range(0..NCA_CHANNELS);
        perturbed_grid[pr][pc][ch] += perturbation_size;

        // Run both forward with a temporary predictor approach:
        // We'll manually step both grids using the predictor's weights
        let weights = predictor.weights().clone();
        let mut orig = settled;
        let mut pert = perturbed_grid;

        for _ in 0..propagation_steps {
            orig = nca_step_grid(&orig, &weights, gs);
            pert = nca_step_grid(&pert, &weights, gs);
        }

        // Count cells with significant difference
        let mut avalanche_size = 0;
        for r in 0..gs {
            for c in 0..gs {
                let mut diff = 0.0;
                for ch in 0..NCA_CHANNELS {
                    diff += (orig[r][c][ch] - pert[r][c][ch]).abs();
                }
                if diff > threshold {
                    avalanche_size += 1;
                }
            }
        }
        sizes.push(avalanche_size);
    }

    let max_size = *sizes.iter().max().unwrap_or(&0);
    let mut histogram = vec![0usize; max_size + 1];
    for &s in &sizes {
        histogram[s] += 1;
    }

    AvalancheStats { sizes, histogram, max_size }
}

/// Run one NCA step on an external grid using given weights.
/// This duplicates the NCA update logic to work on detached grids.
fn nca_step_grid(
    grid: &[Vec<[f64; NCA_CHANNELS]>],
    weights: &NcaWeights,
    gs: usize,
) -> Vec<Vec<[f64; NCA_CHANNELS]>> {
    let mut new_grid = grid.to_vec();
    let perception_size = 9 * NCA_CHANNELS;

    for r in 0..gs {
        for c in 0..gs {
            // Perceive 3x3 neighborhood
            let mut input = vec![0.0; perception_size];
            let mut idx = 0;
            for dr in [-1i32, 0, 1] {
                for dc in [-1i32, 0, 1] {
                    let nr = (r as i32 + dr).rem_euclid(gs as i32) as usize;
                    let nc = (c as i32 + dc).rem_euclid(gs as i32) as usize;
                    for ch in 0..NCA_CHANNELS {
                        input[idx] = grid[nr][nc][ch];
                        idx += 1;
                    }
                }
            }

            // 3-layer MLP forward: input → h1 (ReLU) → h2 (ReLU) → output (tanh) × 0.1
            let h1_size = weights.b1.len();
            let h2_size = weights.b2.len();

            let mut h1 = vec![0.0; h1_size];
            for h in 0..h1_size {
                let mut sum = weights.b1[h];
                for i in 0..perception_size {
                    sum += weights.w1[h][i] * input[i];
                }
                h1[h] = sum.max(0.0);
            }
            let mut h2 = vec![0.0; h2_size];
            for h in 0..h2_size {
                let mut sum = weights.b2[h];
                for i in 0..h1_size {
                    sum += weights.w2[h][i] * h1[i];
                }
                h2[h] = sum.max(0.0);
            }
            for ch in 0..NCA_CHANNELS {
                let mut sum = weights.b3[ch];
                for h in 0..h2_size {
                    sum += weights.w3[ch][h] * h2[h];
                }
                let delta = sum.tanh() * 0.1;
                new_grid[r][c][ch] = (grid[r][c][ch] + delta).clamp(-5.0, 5.0);
            }
        }
    }
    new_grid
}

// ---------------------------------------------------------------------------
// Criticality Metrics
// ---------------------------------------------------------------------------

/// Complete criticality measurement results.
#[derive(Clone, Debug)]
pub struct CriticalityMetrics {
    /// Average cells affected by one cell's perturbation (critical = 1.0)
    pub branching_ratio: f64,
    /// Lyapunov exponent estimate (critical ≈ 0.0; positive = chaotic, negative = ordered)
    pub lyapunov_estimate: f64,
    /// Power law exponent τ from avalanche distribution (critical ≈ 1.5)
    pub power_law_exponent: f64,
    /// Composite score 0–1 where 1.0 = perfectly critical
    pub criticality_score: f64,
    /// Full avalanche statistics
    pub avalanche_stats: AvalancheStats,
}

/// Measure the branching ratio: perturb one cell, count how many cells change
/// significantly after 1 NCA step. Average over many trials.
pub fn measure_branching_ratio(
    predictor: &mut NcaPredictor,
    input_tokens: &[usize],
    n_samples: usize,
    perturbation_size: f64,
    threshold: f64,
) -> f64 {
    let mut rng = rand::thread_rng();
    let gs = predictor.grid_size();
    let weights = predictor.weights().clone();
    let mut total = 0.0;

    for _ in 0..n_samples {
        let settled = predictor.run_and_get_state(input_tokens);

        // Perturb one cell
        let mut perturbed = settled.clone();
        let pr = rng.gen_range(0..gs);
        let pc = rng.gen_range(0..gs);
        let ch = rng.gen_range(0..NCA_CHANNELS);
        perturbed[pr][pc][ch] += perturbation_size;

        // One NCA step for both
        let orig_next = nca_step_grid(&settled, &weights, gs);
        let pert_next = nca_step_grid(&perturbed, &weights, gs);

        // Count affected cells (excluding the perturbed cell itself)
        let mut affected = 0;
        for r in 0..gs {
            for c in 0..gs {
                if r == pr && c == pc { continue; }
                let mut diff = 0.0;
                for ch in 0..NCA_CHANNELS {
                    diff += (orig_next[r][c][ch] - pert_next[r][c][ch]).abs();
                }
                if diff > threshold {
                    affected += 1;
                }
            }
        }
        total += affected as f64;
    }

    total / n_samples as f64
}

/// Estimate the Lyapunov exponent: run two copies of the grid with tiny initial
/// difference, measure divergence over time.
///
/// λ ≈ ln(divergence / epsilon) / steps
/// Positive = chaotic, zero = critical, negative = ordered.
pub fn measure_lyapunov(
    predictor: &mut NcaPredictor,
    input_tokens: &[usize],
    n_samples: usize,
    epsilon: f64,
    steps: usize,
) -> f64 {
    let mut rng = rand::thread_rng();
    let gs = predictor.grid_size();
    let weights = predictor.weights().clone();
    let mut total_lyapunov = 0.0;
    let mut valid_samples = 0;

    for _ in 0..n_samples {
        let settled = predictor.run_and_get_state(input_tokens);

        // Create perturbed copy with epsilon difference everywhere
        let mut perturbed = settled.clone();
        let pr = rng.gen_range(0..gs);
        let pc = rng.gen_range(0..gs);
        for ch in 0..NCA_CHANNELS {
            perturbed[pr][pc][ch] += epsilon;
        }

        // Run both forward
        let mut orig = settled;
        let mut pert = perturbed;
        for _ in 0..steps {
            orig = nca_step_grid(&orig, &weights, gs);
            pert = nca_step_grid(&pert, &weights, gs);
        }

        // Measure total divergence
        let mut divergence = 0.0;
        for r in 0..gs {
            for c in 0..gs {
                for ch in 0..NCA_CHANNELS {
                    divergence += (orig[r][c][ch] - pert[r][c][ch]).powi(2);
                }
            }
        }
        divergence = divergence.sqrt();

        let initial_dist = (epsilon * epsilon * NCA_CHANNELS as f64).sqrt();
        if divergence > 1e-15 && initial_dist > 1e-15 {
            total_lyapunov += (divergence / initial_dist).ln() / steps as f64;
            valid_samples += 1;
        }
    }

    if valid_samples > 0 {
        total_lyapunov / valid_samples as f64
    } else {
        f64::NEG_INFINITY // completely ordered, no divergence
    }
}

/// Fit a power law to the avalanche size distribution using log-log linear regression.
/// P(s) ∝ s^(-τ) → log P(s) = -τ log(s) + const
/// Returns the exponent τ (positive value; critical ≈ 1.5).
pub fn fit_power_law_exponent(stats: &AvalancheStats) -> f64 {
    // Use sizes > 0 only (can't take log of 0)
    let mut log_s = Vec::new();
    let mut log_p = Vec::new();
    let total: f64 = stats.sizes.len() as f64;

    for (size, &count) in stats.histogram.iter().enumerate() {
        if size > 0 && count > 0 {
            log_s.push((size as f64).ln());
            log_p.push((count as f64 / total).ln());
        }
    }

    if log_s.len() < 2 {
        return 0.0; // Not enough data
    }

    // Linear regression: log_p = slope * log_s + intercept
    // slope = -τ
    let n = log_s.len() as f64;
    let sum_x: f64 = log_s.iter().sum();
    let sum_y: f64 = log_p.iter().sum();
    let sum_xy: f64 = log_s.iter().zip(log_p.iter()).map(|(x, y)| x * y).sum();
    let sum_xx: f64 = log_s.iter().map(|x| x * x).sum();

    let denom = n * sum_xx - sum_x * sum_x;
    if denom.abs() < 1e-15 {
        return 0.0;
    }

    let slope = (n * sum_xy - sum_x * sum_y) / denom;
    -slope // τ is the negative of the slope
}

/// Compute composite criticality score from individual metrics.
/// Score is 0–1 where 1.0 = perfectly critical.
pub fn compute_criticality_score(branching_ratio: f64, lyapunov: f64, power_law_exp: f64) -> f64 {
    // Branching ratio: critical = 1.0, score based on distance from 1.0
    let br_score = (-((branching_ratio - 1.0).powi(2)) / 0.5).exp();

    // Lyapunov: critical = 0.0, score based on distance from 0
    let ly_score = (-(lyapunov.powi(2)) / 0.5).exp();

    // Power law exponent: critical ≈ 1.5, score based on distance from 1.5
    let pl_score = if power_law_exp > 0.0 {
        (-((power_law_exp - 1.5).powi(2)) / 1.0).exp()
    } else {
        0.0
    };

    // Weighted combination
    0.4 * br_score + 0.3 * ly_score + 0.3 * pl_score
}

/// Measure all criticality metrics for a predictor.
pub fn measure_criticality(
    predictor: &mut NcaPredictor,
    input_tokens: &[usize],
    n_samples: usize,
    perturbation_size: f64,
) -> CriticalityMetrics {
    let threshold = perturbation_size * 0.5;

    let branching_ratio = measure_branching_ratio(
        predictor, input_tokens, n_samples, perturbation_size, threshold,
    );

    let lyapunov_estimate = measure_lyapunov(
        predictor, input_tokens, n_samples.min(50), perturbation_size * 0.1, 15,
    );

    let avalanche_stats = measure_avalanche_distribution(
        predictor, input_tokens, n_samples, perturbation_size, 10, threshold,
    );

    let power_law_exponent = fit_power_law_exponent(&avalanche_stats);

    let criticality_score = compute_criticality_score(
        branching_ratio, lyapunov_estimate, power_law_exponent,
    );

    CriticalityMetrics {
        branching_ratio,
        lyapunov_estimate,
        power_law_exponent,
        criticality_score,
        avalanche_stats,
    }
}

// ---------------------------------------------------------------------------
// Criticality Regularizer (for training)
// ---------------------------------------------------------------------------

/// Adds a criticality penalty to training loss.
///
/// Use during ES training or reservoir readout training to push
/// the NCA toward the edge of chaos.
pub struct CriticalityRegularizer {
    /// Weight of the criticality penalty (0.0 = no regularization)
    pub weight: f64,
    /// Number of perturbation samples (fewer = faster but noisier)
    pub n_samples: usize,
    /// Perturbation magnitude
    pub perturbation_size: f64,
    /// Input tokens to use for measurement (should activate the grid)
    pub probe_tokens: Vec<usize>,
}

impl CriticalityRegularizer {
    pub fn new(weight: f64, probe_tokens: Vec<usize>) -> Self {
        Self {
            weight,
            n_samples: 10, // Keep low for training speed; use more for final measurement
            perturbation_size: 0.05,
            probe_tokens,
        }
    }

    /// Compute the criticality penalty for given NCA weights.
    /// Returns a value where 0.0 = perfectly critical, higher = further from critical.
    pub fn penalty(
        &self,
        tokenizer: &SimpleTokenizer,
        weights: &NcaWeights,
        grid_size: usize,
        nca_steps: usize,
    ) -> f64 {
        if self.weight == 0.0 {
            return 0.0;
        }

        let mut predictor = NcaPredictor::with_grid_size(
            tokenizer.clone(), weights.clone(), nca_steps, grid_size,
        );

        let metrics = measure_criticality(
            &mut predictor,
            &self.probe_tokens,
            self.n_samples,
            self.perturbation_size,
        );

        // Penalty = 1.0 - criticality_score (so maximizing fitness minimizes penalty)
        self.weight * (1.0 - metrics.criticality_score)
    }

    /// Compute criticality score (for use as a fitness bonus).
    /// Uses a fast approximation (branching ratio only) during training.
    pub fn score(
        &self,
        tokenizer: &SimpleTokenizer,
        weights: &NcaWeights,
        grid_size: usize,
        nca_steps: usize,
    ) -> f64 {
        if self.weight == 0.0 {
            return 0.0;
        }

        let mut predictor = NcaPredictor::with_grid_size(
            tokenizer.clone(), weights.clone(), nca_steps, grid_size,
        );

        // Fast approximation: just branching ratio (most informative, cheapest)
        let threshold = self.perturbation_size * 0.5;
        let br = measure_branching_ratio(
            &mut predictor,
            &self.probe_tokens,
            self.n_samples,
            self.perturbation_size,
            threshold,
        );

        // Score: Gaussian centered at BR=1.0
        (-((br - 1.0).powi(2)) / 0.5).exp()
    }
}

// ---------------------------------------------------------------------------
// Training with Criticality
// ---------------------------------------------------------------------------

/// Train NCA with criticality regularization using evolution strategy.
/// Fitness = accuracy_weight * accuracy + criticality_weight * criticality_score
///
/// Returns (predictor, final_accuracy, criticality_score).
pub fn train_nca_critical(
    corpus: &str,
    config: &super::nca_predictor::TrainingConfig,
    accuracy_weight: f64,
    criticality_weight: f64,
    verbose: bool,
) -> Result<(NcaPredictor, f64, f64), Box<dyn Error>> {
    let grid_size = config.grid_size;
    let tokenizer = SimpleTokenizer::from_corpus(corpus, grid_size * grid_size);
    let tokens = tokenizer.encode(corpus);
    let vocab_size = tokenizer.vocab_size();

    if tokens.len() < config.context_window + 1 {
        return Err("Corpus too small for training".into());
    }

    let random_accuracy = 1.0 / vocab_size as f64;

    if verbose {
        eprintln!("🔥 Criticality-driven NCA training");
        eprintln!("   Accuracy weight: {:.1}, Criticality weight: {:.1}", accuracy_weight, criticality_weight);
        eprintln!("   Vocab: {}, Grid: {}×{}, Random baseline: {:.4}%",
                  vocab_size, grid_size, grid_size, random_accuracy * 100.0);
    }

    // Build training examples
    let max_examples = config.max_examples.min(tokens.len() - config.context_window);
    let step = ((tokens.len() - config.context_window) / max_examples).max(1);
    let examples: Vec<(Vec<usize>, usize)> = (0..tokens.len() - config.context_window)
        .step_by(step)
        .take(max_examples)
        .map(|i| {
            let ctx = tokens[i..i + config.context_window].to_vec();
            let target = tokens[i + config.context_window];
            (ctx, target)
        })
        .collect();

    // Probe tokens for criticality measurement (first few examples' contexts)
    let probe_tokens: Vec<usize> = if !examples.is_empty() {
        examples[0].0.clone()
    } else {
        vec![1, 2, 3]
    };

    let regularizer = CriticalityRegularizer::new(criticality_weight, probe_tokens);

    let mut best_weights = NcaWeights::random();
    let mut best_fitness = f64::NEG_INFINITY;
    let mut best_accuracy = 0.0;
    let mut best_crit_score = 0.0;
    let mut rng = rand::thread_rng();

    for epoch in 0..config.epochs {
        let base_params = best_weights.to_vec();
        let n_params = base_params.len();

        let mut noise_vecs: Vec<Vec<f64>> = Vec::with_capacity(config.population_size);
        let mut fitnesses: Vec<f64> = Vec::with_capacity(config.population_size);

        for _ in 0..config.population_size {
            let noise: Vec<f64> = (0..n_params).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let perturbed: Vec<f64> = base_params.iter()
                .zip(&noise)
                .map(|(p, n)| p + config.sigma * n)
                .collect();

            let w = NcaWeights::from_vec(&perturbed);

            // Accuracy component
            let accuracy = evaluate_fitness_internal(&tokenizer, &w, &examples, grid_size, config.nca_steps);

            // Criticality component
            let crit_score = regularizer.score(&tokenizer, &w, grid_size, config.nca_steps);

            let fitness = accuracy_weight * accuracy + criticality_weight * crit_score;
            noise_vecs.push(noise);
            fitnesses.push(fitness);
        }

        // Normalize and update (same ES update as original)
        let mean_f: f64 = fitnesses.iter().sum::<f64>() / fitnesses.len() as f64;
        let std_f: f64 = {
            let var = fitnesses.iter().map(|f| (f - mean_f).powi(2)).sum::<f64>() / fitnesses.len() as f64;
            var.sqrt().max(1e-8)
        };
        let norm_fitnesses: Vec<f64> = fitnesses.iter().map(|f| (f - mean_f) / std_f).collect();

        let mut new_params = base_params.clone();
        for i in 0..n_params {
            let mut grad = 0.0;
            for j in 0..config.population_size {
                grad += norm_fitnesses[j] * noise_vecs[j][i];
            }
            grad /= (config.population_size as f64) * config.sigma;
            new_params[i] += config.learning_rate * grad;
        }

        let new_weights = NcaWeights::from_vec(&new_params);
        let new_acc = evaluate_fitness_internal(&tokenizer, &new_weights, &examples, grid_size, config.nca_steps);
        let new_crit = regularizer.score(&tokenizer, &new_weights, grid_size, config.nca_steps);
        let new_fitness = accuracy_weight * new_acc + criticality_weight * new_crit;

        if new_fitness > best_fitness {
            best_fitness = new_fitness;
            best_weights = new_weights;
            best_accuracy = new_acc;
            best_crit_score = new_crit;
        }

        if verbose {
            eprintln!("  Epoch {}/{}: acc={:.2}%, crit={:.3}, fitness={:.4}",
                      epoch + 1, config.epochs, best_accuracy * 100.0, best_crit_score, best_fitness);
        }
    }

    let predictor = NcaPredictor::with_grid_size(tokenizer, best_weights, config.nca_steps, grid_size);
    Ok((predictor, best_accuracy, best_crit_score))
}

/// Internal accuracy evaluation (top-5), same as nca_predictor's logic.
fn evaluate_fitness_internal(
    tokenizer: &SimpleTokenizer,
    weights: &NcaWeights,
    examples: &[(Vec<usize>, usize)],
    grid_size: usize,
    nca_steps: usize,
) -> f64 {
    let mut predictor = NcaPredictor::with_grid_size(tokenizer.clone(), weights.clone(), nca_steps, grid_size);
    let mut correct = 0;
    for (ctx, target) in examples {
        let activations = predictor.run_and_read(ctx);
        let mut indexed: Vec<(usize, f64)> = activations.iter().enumerate().map(|(i, &v)| (i, v)).collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if indexed.iter().take(5).any(|(id, _)| id == target) {
            correct += 1;
        }
    }
    correct as f64 / examples.len() as f64
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_predictor() -> (NcaPredictor, Vec<usize>) {
        let corpus = "the cat sat on the mat the dog sat on the log";
        let tokenizer = SimpleTokenizer::from_corpus(corpus, 64);
        let tokens = tokenizer.encode("the cat sat");
        let weights = NcaWeights::random();
        let predictor = NcaPredictor::with_grid_size(tokenizer, weights, 3, 8);
        (predictor, tokens)
    }

    #[test]
    fn test_avalanche_distribution() {
        let (mut predictor, tokens) = make_test_predictor();
        let stats = measure_avalanche_distribution(&mut predictor, &tokens, 20, 0.05, 5, 0.01);
        assert_eq!(stats.sizes.len(), 20);
        assert!(!stats.histogram.is_empty());
    }

    #[test]
    fn test_branching_ratio() {
        let (mut predictor, tokens) = make_test_predictor();
        let br = measure_branching_ratio(&mut predictor, &tokens, 20, 0.05, 0.01);
        assert!(br >= 0.0, "branching ratio should be non-negative");
    }

    #[test]
    fn test_lyapunov_estimate() {
        let (mut predictor, tokens) = make_test_predictor();
        let ly = measure_lyapunov(&mut predictor, &tokens, 10, 0.001, 10);
        // Should be finite
        assert!(ly.is_finite(), "lyapunov should be finite, got {}", ly);
    }

    #[test]
    fn test_power_law_fit() {
        // Create a synthetic power-law-ish distribution
        let sizes = vec![1, 1, 1, 1, 2, 2, 2, 3, 3, 5, 8, 15];
        let max_size = 15;
        let mut histogram = vec![0; 16];
        for &s in &sizes { histogram[s] += 1; }
        let stats = AvalancheStats { sizes, histogram, max_size };
        let tau = fit_power_law_exponent(&stats);
        assert!(tau > 0.0, "exponent should be positive, got {}", tau);
    }

    #[test]
    fn test_criticality_score_bounds() {
        // Perfect criticality
        let score = compute_criticality_score(1.0, 0.0, 1.5);
        assert!((score - 1.0).abs() < 0.01, "perfect criticality should score ~1.0, got {}", score);

        // Far from critical
        let score2 = compute_criticality_score(5.0, 3.0, 0.0);
        assert!(score2 < 0.3, "far from critical should score low, got {}", score2);
    }

    #[test]
    fn test_full_criticality_measurement() {
        let (mut predictor, tokens) = make_test_predictor();
        let metrics = measure_criticality(&mut predictor, &tokens, 20, 0.05);
        assert!(metrics.criticality_score >= 0.0 && metrics.criticality_score <= 1.0);
        assert!(metrics.branching_ratio >= 0.0);
    }

    #[test]
    fn test_ascii_histogram() {
        let stats = AvalancheStats {
            sizes: vec![0, 1, 1, 2, 3, 5],
            histogram: vec![1, 2, 1, 1, 0, 1],
            max_size: 5,
        };
        let hist = stats.ascii_histogram(20);
        assert!(!hist.is_empty());
    }

    #[test]
    fn test_regularizer_penalty() {
        let corpus = "the cat sat on the mat";
        let tokenizer = SimpleTokenizer::from_corpus(corpus, 64);
        let weights = NcaWeights::random();
        let reg = CriticalityRegularizer::new(0.3, vec![1, 2, 3]);
        let penalty = reg.penalty(&tokenizer, &weights, 8, 3);
        assert!(penalty >= 0.0 && penalty <= 0.3);
    }

    #[test]
    fn test_nca_step_grid_matches() {
        // Verify our standalone nca_step_grid produces valid output
        let (mut predictor, tokens) = make_test_predictor();
        let state = predictor.run_and_get_state(&tokens);
        let weights = predictor.weights().clone();
        let gs = predictor.grid_size();
        let next = nca_step_grid(&state, &weights, gs);
        assert_eq!(next.len(), gs);
        assert_eq!(next[0].len(), gs);
        // Values should be clamped
        for r in 0..gs {
            for c in 0..gs {
                for ch in 0..NCA_CHANNELS {
                    assert!(next[r][c][ch] >= -5.0 && next[r][c][ch] <= 5.0);
                }
            }
        }
    }
}

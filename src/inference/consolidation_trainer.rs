//! Consolidation Parameter Training via Evolution Strategy
//!
//! Trains the 5 consolidation parameters (decay_rate, strengthen_rate, spread_rate,
//! confidence_boost, activation_threshold) on retrieval quality.
//!
//! The idea:
//! 1. Start with random parameters
//! 2. Encode facts into the NCA grid
//! 3. Run consolidation steps
//! 4. Measure retrieval accuracy
//! 5. Evolve parameters toward better retrieval

use crate::distributed_knowledge::{NCAKnowledge, KnowledgeStore};
use crate::grid::ConsolidationParams;
use rand::Rng;
use std::error::Error;

/// Training configuration for consolidation parameter ES
#[derive(Clone, Debug)]
pub struct ConsolidationTrainingConfig {
    /// Number of ES epochs
    pub epochs: usize,
    /// Population size per epoch
    pub population_size: usize,
    /// Initial standard deviation for parameter perturbation
    pub sigma: f64,
    /// Number of consolidation steps per evaluation
    pub consolidation_steps: usize,
    /// Number of facts to encode for retrieval test
    pub num_facts: usize,
    /// Number of retrieval queries per evaluation
    pub num_queries: usize,
    /// Learning rate for parameter update
    pub learning_rate: f64,
    /// Number of top performers to keep (elitism)
    pub elite_count: usize,
}

impl Default for ConsolidationTrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 30,
            population_size: 20,
            sigma: 0.1,
            consolidation_steps: 2,
            num_facts: 50,
            num_queries: 20,
            learning_rate: 0.1,
            elite_count: 5,
        }
    }
}

/// Train consolidation parameters via evolution strategy.
///
/// Returns trained parameters and final retrieval accuracy.
///
/// # Arguments
/// * `config` - Training configuration
/// * `verbose` - Print progress
///
/// # Example
/// ```ignore
/// use sage::inference::consolidation_trainer::{train_consolidation, ConsolidationTrainingConfig};
///
/// let config = ConsolidationTrainingConfig::default();
/// let (params, accuracy) = train_consolidation(&config, true).unwrap();
/// println!("Trained params: decay={}, strengthen={}", params.decay_rate, params.strengthen_rate);
/// println!("Retrieval accuracy: {:.1}%", accuracy * 100.0);
/// ```
pub fn train_consolidation(
    config: &ConsolidationTrainingConfig,
    verbose: bool,
) -> Result<(ConsolidationParams, f64), Box<dyn Error>> {
    let mut rng = rand::thread_rng();

    // Start with default params
    let mut best_params = ConsolidationParams::default();
    let mut best_fitness = evaluate_params(&best_params, config)?;

    if verbose {
        eprintln!(
            "🎯 Starting consolidation training ({} epochs, {} population)",
            config.epochs, config.population_size
        );
        eprintln!("   Initial accuracy: {:.1}%", best_fitness * 100.0);
    }

    for epoch in 0..config.epochs {
        let base_vec = best_params.to_vec();
        let n_params = base_vec.len();

        // Generate perturbations and evaluate
        let mut population: Vec<(ConsolidationParams, f64)> = Vec::with_capacity(config.population_size);

        for _ in 0..config.population_size {
            let noise: Vec<f64> = (0..n_params).map(|_| rng.gen::<f64>() * 2.0 - 1.0).collect();
            let perturbed: Vec<f64> = base_vec
                .iter()
                .zip(&noise)
                .map(|(&p, &n)| p + config.sigma * n)
                .collect();
            let params = ConsolidationParams::from_vec(&perturbed);

            let fitness = evaluate_params(&params, config)?;
            population.push((params, fitness));
        }

        // Add current best
        population.push((best_params.clone(), best_fitness));

        // Sort by fitness (descending)
        population.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Keep elites
        let elites: Vec<&(ConsolidationParams, f64)> = population.iter().take(config.elite_count).collect();

        // Compute weighted average of elite parameters
        let mut new_vec = vec![0.0; n_params];
        let total_weight: f64 = (1..=config.elite_count).sum::<usize>() as f64;

        for (i, (params, _fitness)) in elites.iter().enumerate() {
            let weight = (config.elite_count - i) as f64 / total_weight;
            let params_vec = params.to_vec();
            for j in 0..n_params {
                new_vec[j] += weight * params_vec[j];
            }
        }

        let new_params = ConsolidationParams::from_vec(&new_vec);
        let new_fitness = evaluate_params(&new_params, config)?;

        // Update if improved
        if new_fitness > best_fitness {
            best_params = new_params;
            best_fitness = new_fitness;

            if verbose && (epoch + 1) % 5 == 0 {
                eprintln!(
                    "   Epoch {}: accuracy={:.1}% (decay={:.3}, strengthen={:.3}, spread={:.3})",
                    epoch + 1,
                    best_fitness * 100.0,
                    best_params.decay_rate,
                    best_params.strengthen_rate,
                    best_params.spread_rate
                );
            }
        }
    }

    if verbose {
        eprintln!(
            "✅ Final: accuracy={:.1}% with decay={:.3}, strengthen={:.3}, spread={:.3}, conf={:.3}, thresh={:.3}",
            best_fitness * 100.0,
            best_params.decay_rate,
            best_params.strengthen_rate,
            best_params.spread_rate,
            best_params.confidence_boost,
            best_params.activation_threshold
        );
    }

    Ok((best_params, best_fitness))
}

/// Evaluate consolidation parameters on retrieval accuracy.
///
/// Creates a grid, encodes facts, runs consolidation, then tests retrieval.
fn evaluate_params(params: &ConsolidationParams, config: &ConsolidationTrainingConfig) -> Result<f64, Box<dyn Error>> {
    let mut knowledge = NCAKnowledge::new();

    // Generate random facts
    let facts: Vec<String> = (0..config.num_facts)
        .map(|i| format!("fact_{}: This is test fact number {}", i, i))
        .collect();

    // Encode facts
    for fact in &facts {
        knowledge.encode(fact, 0.8);
    }

    // Run consolidation
    knowledge.grid.consolidate_knowledge_with_params(config.consolidation_steps, params);

    // Test retrieval
    let mut correct = 0;
    let query_indices: Vec<usize> = (0..config.num_queries)
        .map(|_| rand::thread_rng().gen_range(0..config.num_facts))
        .collect();

    for &idx in &query_indices {
        let query = &facts[idx];
        let results = knowledge.query(query, 5);

        // Check if the correct fact is in top 5
        if results.iter().any(|r| r.text.as_deref() == Some(query.as_str())) {
            correct += 1;
        }
    }

    Ok(correct as f64 / config.num_queries as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consolidation_params_roundtrip() {
        let params = ConsolidationParams {
            decay_rate: 0.05,
            strengthen_rate: 0.08,
            spread_rate: 0.04,
            confidence_boost: 0.03,
            activation_threshold: 0.4,
        };

        let vec = params.to_vec();
        assert_eq!(vec.len(), 5);

        let restored = ConsolidationParams::from_vec(&vec);
        assert!((restored.decay_rate - params.decay_rate).abs() < 1e-10);
        assert!((restored.strengthen_rate - params.strengthen_rate).abs() < 1e-10);
        assert!((restored.spread_rate - params.spread_rate).abs() < 1e-10);
        assert!((restored.confidence_boost - params.confidence_boost).abs() < 1e-10);
        assert!((restored.activation_threshold - params.activation_threshold).abs() < 1e-10);
    }

    #[test]
    fn test_consolidation_params_clamping() {
        let params = ConsolidationParams::from_vec(&[2.0, -0.5, 0.5, 0.2, 1.5]);
        assert!(params.decay_rate <= 0.2);
        assert!(params.strengthen_rate >= 0.0);
        assert!(params.spread_rate <= 0.2);
        assert!(params.confidence_boost <= 0.1);
        assert!(params.activation_threshold >= 0.05 && params.activation_threshold <= 0.8);
    }

    #[test]
    fn test_consolidation_params_perturb() {
        let params = ConsolidationParams::default();
        let noise = [0.5, -0.5, 0.3, -0.3, 0.1];
        let perturbed = params.perturb(&noise, 0.1);

        // Perturbed params should be close to original
        assert!((perturbed.decay_rate - params.decay_rate).abs() < 0.1);
        assert!((perturbed.strengthen_rate - params.strengthen_rate).abs() < 0.1);
    }
}
//! Population Based Training (PBT)
//!
//! PBT (DeepMind, 2017) discovers optimal hyperparameter *schedules* through evolution.
//!
//! Algorithm:
//! ```text
//! 1. Initialize population of N networks with random hyperparameters
//! 2. Train all networks in parallel
//! 3. Periodically:
//!    - Evaluate all networks
//!    - EXPLOIT: Bottom 20% copy weights from top 20%
//!    - EXPLORE: Mutate hyperparameters of copied networks
//! 4. Continue training
//! ```
//!
//! Key insight: Discovers *schedules*, not just fixed values.
//! Learning rate might need to be high early, low later - PBT finds this.

use crate::nca::{NCA, WeightSnapshot};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// HYPERPARAMETERS
// ============================================================================

/// Hyperparameters that PBT can evolve
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hyperparameters {
    /// Learning rate for weight updates
    pub learning_rate: f64,
    /// Number of NCA evolution steps before computing loss
    pub evolution_steps: usize,
    /// Batch size (number of seeds per training step)
    pub batch_size: usize,
    /// Probability of sampling from pool vs fresh seed
    pub pool_sample_rate: f64,
    /// Stochastic cell update rate (0.0-1.0)
    pub stochastic_update_rate: f64,
    /// Gradient clip norm
    pub grad_clip_norm: f64,
}

impl Default for Hyperparameters {
    fn default() -> Self {
        Self {
            learning_rate: 0.0001,
            evolution_steps: 64,
            batch_size: 5,
            pool_sample_rate: 0.3,
            stochastic_update_rate: 0.5,
            grad_clip_norm: 3.0,
        }
    }
}

impl Hyperparameters {
    /// Create with random perturbations from default
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let default = Self::default();

        Self {
            learning_rate: default.learning_rate * 2.0_f64.powf(rng.gen_range(-2.0..2.0)),
            evolution_steps: (default.evolution_steps as f64 * rng.gen_range(0.5..2.0)) as usize,
            batch_size: rng.gen_range(1..=10),
            pool_sample_rate: rng.gen_range(0.0..0.5),
            stochastic_update_rate: rng.gen_range(0.3..0.7),
            grad_clip_norm: rng.gen_range(1.0..5.0),
        }
    }

    /// Mutate hyperparameters (explore step)
    pub fn mutate(&mut self) {
        let mut rng = rand::thread_rng();

        // Each hyperparameter has 30% chance of being mutated
        if rng.gen::<f64>() < 0.3 {
            // Multiply by random factor in [0.8, 1.2]
            self.learning_rate *= rng.gen_range(0.8..1.2);
            self.learning_rate = self.learning_rate.clamp(1e-6, 0.01);
        }

        if rng.gen::<f64>() < 0.3 {
            let delta: i32 = rng.gen_range(-20..=20);
            self.evolution_steps = (self.evolution_steps as i32 + delta).clamp(20, 200) as usize;
        }

        if rng.gen::<f64>() < 0.3 {
            let delta: i32 = rng.gen_range(-2..=2);
            self.batch_size = (self.batch_size as i32 + delta).clamp(1, 10) as usize;
        }

        if rng.gen::<f64>() < 0.3 {
            self.pool_sample_rate += rng.gen_range(-0.1..0.1);
            self.pool_sample_rate = self.pool_sample_rate.clamp(0.0, 0.5);
        }

        if rng.gen::<f64>() < 0.3 {
            self.stochastic_update_rate += rng.gen_range(-0.1..0.1);
            self.stochastic_update_rate = self.stochastic_update_rate.clamp(0.3, 0.7);
        }

        if rng.gen::<f64>() < 0.3 {
            self.grad_clip_norm *= rng.gen_range(0.8..1.2);
            self.grad_clip_norm = self.grad_clip_norm.clamp(1.0, 10.0);
        }
    }

    /// Resample hyperparameters completely (stronger exploration)
    pub fn resample(&mut self) {
        *self = Self::random();
    }
}

// ============================================================================
// POPULATION MEMBER
// ============================================================================

/// A single member of the population
#[derive(Clone)]
pub struct PopulationMember {
    /// Unique identifier
    pub id: usize,
    /// Current network weights
    pub weights: WeightSnapshot,
    /// Hyperparameters for this member
    pub hyperparams: Hyperparameters,
    /// Current performance score (lower = better for loss)
    pub score: f64,
    /// Number of training steps completed
    pub steps: usize,
    /// Generation (how many times this lineage has been evolved)
    pub generation: usize,
    /// Parent ID (for lineage tracking)
    pub parent_id: Option<usize>,
    /// History of scores for this member
    pub score_history: VecDeque<f64>,
}

impl PopulationMember {
    pub fn new(id: usize, nca: &NCA) -> Self {
        Self {
            id,
            weights: nca.update_net.snapshot(),
            hyperparams: Hyperparameters::random(),
            score: f64::MAX,
            steps: 0,
            generation: 0,
            parent_id: None,
            score_history: VecDeque::new(),
        }
    }

    /// Record a score
    pub fn record_score(&mut self, score: f64) {
        self.score = score;
        self.score_history.push_back(score);
        if self.score_history.len() > 50 {
            self.score_history.pop_front();
        }
    }

    /// Get average recent score
    pub fn avg_recent_score(&self) -> f64 {
        if self.score_history.is_empty() {
            return self.score;
        }
        self.score_history.iter().sum::<f64>() / self.score_history.len() as f64
    }

    /// Check if improving
    pub fn is_improving(&self) -> bool {
        if self.score_history.len() < 10 {
            return true;
        }
        let recent: f64 = self.score_history.iter().rev().take(5).sum::<f64>() / 5.0;
        let older: f64 = self.score_history.iter().take(5).sum::<f64>() / 5.0;
        recent < older
    }
}

// ============================================================================
// POPULATION BASED TRAINING
// ============================================================================

/// Population Based Training system
pub struct PopulationBasedTraining {
    /// Population of network members
    population: Vec<PopulationMember>,
    /// Population size
    population_size: usize,
    /// Fraction of population to exploit (bottom X% copies from top X%)
    exploit_fraction: f64,
    /// How often to run exploit/explore (every N steps)
    evolution_interval: usize,
    /// Total steps across all members
    total_steps: usize,
    /// Best hyperparameters found so far
    best_hyperparams: Hyperparameters,
    /// Best score achieved
    best_score: f64,
    /// History of best scores over time
    best_score_history: Vec<(usize, f64)>,
    /// Hyperparameter schedule (history of best hyperparams at each evolution)
    hyperparameter_schedule: Vec<(usize, Hyperparameters)>,
}

impl PopulationBasedTraining {
    /// Create a new PBT system
    pub fn new(population_size: usize, nca: &NCA) -> Self {
        let mut population = Vec::with_capacity(population_size);

        for i in 0..population_size {
            let mut member = PopulationMember::new(i, nca);
            // Ensure diversity by randomizing hyperparams
            member.hyperparams = Hyperparameters::random();
            population.push(member);
        }

        Self {
            population,
            population_size,
            exploit_fraction: 0.2,  // Bottom/top 20%
            evolution_interval: 50,  // Evolve every 50 steps
            total_steps: 0,
            best_hyperparams: Hyperparameters::default(),
            best_score: f64::MAX,
            best_score_history: Vec::new(),
            hyperparameter_schedule: Vec::new(),
        }
    }

    /// Configure exploit fraction
    pub fn with_exploit_fraction(mut self, fraction: f64) -> Self {
        self.exploit_fraction = fraction.clamp(0.1, 0.5);
        self
    }

    /// Configure evolution interval
    pub fn with_evolution_interval(mut self, interval: usize) -> Self {
        self.evolution_interval = interval.max(10);
        self
    }

    /// Get population size
    pub fn population_size(&self) -> usize {
        self.population_size
    }

    /// Get a member's hyperparameters
    pub fn get_hyperparams(&self, member_id: usize) -> Option<&Hyperparameters> {
        self.population.get(member_id).map(|m| &m.hyperparams)
    }

    /// Get best hyperparameters found so far
    pub fn get_best_hyperparams(&self) -> &Hyperparameters {
        &self.best_hyperparams
    }

    /// Apply a member's weights to an NCA
    pub fn apply_member_weights(&self, member_id: usize, nca: &mut NCA) {
        if let Some(member) = self.population.get(member_id) {
            nca.update_net.load_snapshot(&member.weights);
        }
    }

    /// Save a member's weights from an NCA
    pub fn save_member_weights(&mut self, member_id: usize, nca: &NCA) {
        if let Some(member) = self.population.get_mut(member_id) {
            member.weights = nca.update_net.snapshot();
        }
    }

    /// Record evaluation result for a member
    pub fn record_evaluation(&mut self, member_id: usize, score: f64, nca: &NCA) {
        if let Some(member) = self.population.get_mut(member_id) {
            member.record_score(score);
            member.steps += 1;
            member.weights = nca.update_net.snapshot();
        }

        self.total_steps += 1;

        // Track best score
        if score < self.best_score {
            self.best_score = score;
            if let Some(member) = self.population.get(member_id) {
                self.best_hyperparams = member.hyperparams.clone();
            }
            self.best_score_history.push((self.total_steps, score));
        }

        // Check if it's time to evolve
        if self.total_steps % self.evolution_interval == 0 {
            self.evolve();
        }
    }

    /// Run exploit/explore evolution step
    pub fn evolve(&mut self) -> EvolutionResult {
        // Sort population by score (lower = better)
        self.population.sort_by(|a, b| {
            a.avg_recent_score().partial_cmp(&b.avg_recent_score()).unwrap()
        });

        let n = self.population_size;
        let cutoff = (n as f64 * self.exploit_fraction) as usize;
        let cutoff = cutoff.max(1);

        let mut exploited = 0;
        let mut explored = 0;

        // Bottom performers copy from top performers
        for i in (n - cutoff)..n {
            // Select random member from top performers
            let source_idx = rand::thread_rng().gen_range(0..cutoff);

            // EXPLOIT: Copy weights and hyperparams
            let source_weights = self.population[source_idx].weights.clone();
            let source_hyperparams = self.population[source_idx].hyperparams.clone();
            let source_id = self.population[source_idx].id;

            self.population[i].weights = source_weights;
            self.population[i].hyperparams = source_hyperparams;
            self.population[i].parent_id = Some(source_id);
            self.population[i].generation += 1;
            self.population[i].score_history.clear();
            exploited += 1;

            // EXPLORE: Mutate hyperparameters
            self.population[i].hyperparams.mutate();
            explored += 1;
        }

        // Record hyperparameter schedule
        let best_hp = self.population[0].hyperparams.clone();
        self.hyperparameter_schedule.push((self.total_steps, best_hp));

        EvolutionResult {
            step: self.total_steps,
            members_exploited: exploited,
            members_explored: explored,
            best_score: self.population[0].avg_recent_score(),
            worst_score: self.population[n - 1].avg_recent_score(),
            best_hyperparams: self.population[0].hyperparams.clone(),
        }
    }

    /// Get population statistics
    pub fn get_stats(&self) -> PBTStats {
        let scores: Vec<f64> = self.population.iter()
            .map(|m| m.avg_recent_score())
            .filter(|s| s.is_finite())
            .collect();

        let avg_score = if !scores.is_empty() {
            scores.iter().sum::<f64>() / scores.len() as f64
        } else {
            f64::MAX
        };

        let score_variance = if scores.len() > 1 {
            let mean = avg_score;
            scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / scores.len() as f64
        } else {
            0.0
        };

        // Count unique hyperparameter configurations (diversity)
        let diversity = self.calculate_diversity();

        PBTStats {
            total_steps: self.total_steps,
            population_size: self.population_size,
            best_score: self.best_score,
            avg_score,
            score_variance,
            diversity,
            evolutions: self.hyperparameter_schedule.len(),
            best_hyperparams: self.best_hyperparams.clone(),
            best_lr: self.best_hyperparams.learning_rate,
        }
    }

    /// Calculate hyperparameter diversity in population
    fn calculate_diversity(&self) -> f64 {
        if self.population.len() < 2 {
            return 0.0;
        }

        // Compute variance of each hyperparameter
        let lrs: Vec<f64> = self.population.iter().map(|m| m.hyperparams.learning_rate.ln()).collect();
        let steps: Vec<f64> = self.population.iter().map(|m| m.hyperparams.evolution_steps as f64).collect();

        let lr_var = variance(&lrs);
        let steps_var = variance(&steps);

        // Normalize and combine
        (lr_var + steps_var / 1000.0) / 2.0
    }

    /// Get the hyperparameter schedule discovered
    pub fn get_schedule(&self) -> &[(usize, Hyperparameters)] {
        &self.hyperparameter_schedule
    }

    /// Get all members for inspection
    pub fn get_population(&self) -> &[PopulationMember] {
        &self.population
    }

    /// Get best member
    pub fn get_best_member(&self) -> Option<&PopulationMember> {
        self.population.iter()
            .min_by(|a, b| a.avg_recent_score().partial_cmp(&b.avg_recent_score()).unwrap())
    }
}

fn variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64
}

// ============================================================================
// RESULT TYPES
// ============================================================================

#[derive(Clone, Debug)]
pub struct EvolutionResult {
    pub step: usize,
    pub members_exploited: usize,
    pub members_explored: usize,
    pub best_score: f64,
    pub worst_score: f64,
    pub best_hyperparams: Hyperparameters,
}

#[derive(Clone, Debug)]
pub struct PBTStats {
    pub total_steps: usize,
    pub population_size: usize,
    pub best_score: f64,
    pub avg_score: f64,
    pub score_variance: f64,
    pub diversity: f64,
    pub evolutions: usize,
    pub best_hyperparams: Hyperparameters,
    pub best_lr: f64,  // Best learning rate found
}

// ============================================================================
// TRAINING HELPER
// ============================================================================

/// Helper for running PBT training loop
pub struct PBTTrainer {
    pub pbt: PopulationBasedTraining,
    pub current_member: usize,
}

impl PBTTrainer {
    pub fn new(population_size: usize, nca: &NCA) -> Self {
        Self {
            pbt: PopulationBasedTraining::new(population_size, nca),
            current_member: 0,
        }
    }

    /// Get hyperparameters for current member
    pub fn current_hyperparams(&self) -> Hyperparameters {
        self.pbt.get_hyperparams(self.current_member)
            .cloned()
            .unwrap_or_default()
    }

    /// Apply current member's weights to NCA
    pub fn apply_current_weights(&self, nca: &mut NCA) {
        self.pbt.apply_member_weights(self.current_member, nca);
    }

    /// Record result and potentially evolve
    pub fn record_result(&mut self, score: f64, nca: &NCA) -> Option<EvolutionResult> {
        let _step_before = self.pbt.total_steps;
        self.pbt.record_evaluation(self.current_member, score, nca);

        // Move to next member (round-robin)
        self.current_member = (self.current_member + 1) % self.pbt.population_size;

        // Check if evolution happened
        let evolutions_before = self.pbt.hyperparameter_schedule.len();
        if self.pbt.hyperparameter_schedule.len() > evolutions_before {
            // Return the evolution result
            let last_hp = self.pbt.hyperparameter_schedule.last().unwrap();
            Some(EvolutionResult {
                step: last_hp.0,
                members_exploited: (self.pbt.population_size as f64 * self.pbt.exploit_fraction) as usize,
                members_explored: (self.pbt.population_size as f64 * self.pbt.exploit_fraction) as usize,
                best_score: self.pbt.best_score,
                worst_score: self.pbt.get_population().last().map(|m| m.score).unwrap_or(f64::MAX),
                best_hyperparams: last_hp.1.clone(),
            })
        } else {
            None
        }
    }

    /// Get best hyperparameters found
    pub fn best_hyperparams(&self) -> &Hyperparameters {
        self.pbt.get_best_hyperparams()
    }

    /// Get statistics
    pub fn stats(&self) -> PBTStats {
        self.pbt.get_stats()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperparameters() {
        let hp = Hyperparameters::default();
        assert!(hp.learning_rate > 0.0);
        assert!(hp.evolution_steps > 0);

        let mut hp2 = hp.clone();
        hp2.mutate();
        // At least one param should be different (probabilistically)
    }

    #[test]
    fn test_population_creation() {
        let nca = NCA::new();
        let pbt = PopulationBasedTraining::new(8, &nca);

        assert_eq!(pbt.population_size(), 8);

        // Check diversity - use string representation since f64 doesn't impl Hash
        let lrs: Vec<String> = pbt.get_population().iter()
            .map(|m| format!("{:.10}", m.hyperparams.learning_rate))
            .collect();
        let unique_count = lrs.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 1, "Population should have diverse hyperparams");
    }

    #[test]
    fn test_pbt_trainer() {
        let nca = NCA::new();
        let mut trainer = PBTTrainer::new(4, &nca);

        // Simulate some training
        for i in 0..20 {
            let score = 1.0 / (i as f64 + 1.0);
            trainer.record_result(score, &nca);
        }

        let stats = trainer.stats();
        assert!(stats.total_steps == 20);
        assert!(stats.best_score < 1.0);
    }
}

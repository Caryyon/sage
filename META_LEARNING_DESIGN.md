# SAGE Meta-Learning System Design

## Executive Summary

This document outlines a comprehensive meta-learning architecture for SAGE that enables it to learn *how to learn* more effectively. The design synthesizes cutting-edge research from MAML, Population Based Training, Self-Paced Learning, Learning to Optimize, and Universal Neural Cellular Automata.

---

## Research Foundation

### 1. Model-Agnostic Meta-Learning (MAML) & Reptile

**Key Insight**: Find an initialization θ* that can quickly adapt to new tasks with just a few gradient steps.

**Reptile Algorithm** (OpenAI, 2018):
```
Initialize θ
for iteration = 1, 2, ...
    Sample task T
    Compute θ' = SGD(T, θ, k steps)  // Train on task
    θ ← θ + ε(θ' - θ)                // Move toward adapted weights
```

**Application to SAGE**: The NCA network learns an initialization that can rapidly adapt to ANY pattern, not just memorize specific ones. This enables few-shot pattern learning.

### 2. Population Based Training (PBT)

**Key Insight**: Discover hyperparameter *schedules*, not just fixed values.

**Algorithm**:
1. Train population of N networks in parallel with different hyperparameters
2. Periodically evaluate all networks
3. **Exploit**: Bottom 20% copy weights from top 20%
4. **Explore**: Perturb hyperparameters of copied networks
5. Continue training

**Application to SAGE**: Maintain multiple "sub-brains" with different hyperparameters, evolve toward best-performing configurations.

### 3. Self-Paced Learning (SPL)

**Key Insight**: Let the model decide what's easy/hard, not humans.

**Algorithm**:
```
v* = argmin_v { L(w,v) + g(v,λ) }
    where L = loss, v = sample weights, g = regularizer

# Samples with loss < λ get v=1, others get v=0
# λ increases during training (easy → hard)
```

**Application to SAGE**: Dynamically weight patterns based on current loss. Train more on learnable patterns, less on currently-impossible ones.

### 4. Universal Neural Cellular Automata

**Key Insight**: Separate "hardware" (immutable context) from "state" (mutable computation).

**Architecture**:
- **Hardware channels**: Fixed per-cell vectors that condition behavior
- **State channels**: Dynamic values that evolve through NCA updates
- **Attention mechanism**: Hardware activates different "computational modes"

**Application to SAGE**: Use pattern condition channels as "hardware" to enable one network to produce any learned pattern. Fine-tune only hardware while keeping update rules frozen.

### 5. Learning to Optimize (L2O)

**Key Insight**: Learn the optimizer itself, not just the weights.

**Architecture**:
```
Δθ = f_φ(∇L, h_t)  // Neural network produces update
h_{t+1} = RNN(h_t, ∇L, θ)  // Maintain learning state
```

**Application to SAGE**: Replace Adam optimizer with a learned LSTM-based optimizer that discovers optimal update rules for NCA training.

---

## SAGE Meta-Learning Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    SAGE Meta-Learning System                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Level 3: Meta-Meta-Learning                      │ │
│  │                                                                      │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │ │
│  │  │  Strategy   │  │  Algorithm  │  │  Architecture               │ │ │
│  │  │  Selection  │  │  Discovery  │  │  Self-Modification          │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                │                                         │
│                                ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Level 2: Meta-Learning                           │ │
│  │                                                                      │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │ │
│  │  │  Reptile    │  │  Population │  │  Learned                    │ │ │
│  │  │  Init       │  │  Based      │  │  Optimizer                  │ │ │
│  │  │  Optimizer  │  │  Training   │  │  (L2O)                      │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                │                                         │
│                                ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Level 1: Adaptive Learning                       │ │
│  │                                                                      │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────┐ │ │
│  │  │  Self-Paced │  │  Adaptive   │  │  Curiosity-                 │ │ │
│  │  │  Curriculum │  │  Learning   │  │  Driven                     │ │ │
│  │  │             │  │  Rate       │  │  Exploration                │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                │                                         │
│                                ▼                                         │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Level 0: Base Learning                           │ │
│  │                                                                      │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │                   NCA Training Loop                          │   │ │
│  │  │   Pattern Formation → Stability → Damage Resistance          │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Level 1: Enhanced Adaptive Learning

### 1.1 Self-Paced Curriculum Learning

**Current**: Fixed spiral curriculum (Circle → Square → Cross → Spiral)

**Enhanced**: Dynamic difficulty-based selection

```rust
pub struct SelfPacedCurriculum {
    patterns: Vec<PatternTask>,
    difficulty_threshold: f64,  // λ - increases over time
    pace_function: PaceFunction,
    pattern_losses: HashMap<String, VecDeque<f64>>,
}

pub enum PaceFunction {
    Linear,      // λ(t) = λ_0 + αt
    Logarithmic, // λ(t) = λ_0 * (1 + log(1 + t/τ))
    SelfTuned,   // λ adapts based on success rate
}

impl SelfPacedCurriculum {
    /// Select next pattern based on current learning state
    pub fn select_pattern(&self) -> &PatternTask {
        // Calculate sample weights: v_i = 1 if loss_i < λ, else 0
        let learnable: Vec<_> = self.patterns.iter()
            .filter(|p| self.get_loss(p) < self.difficulty_threshold)
            .collect();

        if learnable.is_empty() {
            // No patterns below threshold - pick easiest
            self.patterns.iter().min_by_key(|p| self.get_loss(p)).unwrap()
        } else {
            // Sample from learnable patterns, preferring harder ones
            // (within the learnable set)
            self.sample_by_difficulty(&learnable)
        }
    }

    /// Update threshold based on mastery progress
    pub fn update_threshold(&mut self, success_rate: f64) {
        match self.pace_function {
            PaceFunction::SelfTuned => {
                if success_rate > 0.7 {
                    self.difficulty_threshold *= 1.1;  // Increase challenge
                } else if success_rate < 0.3 {
                    self.difficulty_threshold *= 0.9;  // Reduce challenge
                }
            }
            // ... other functions
        }
    }
}
```

### 1.2 Improved Adaptive Learning Rate

**Current**: Simple patience-based reduction

**Enhanced**: Gradient-aware with warmup and cyclical restarts

```rust
pub struct EnhancedAdaptiveLR {
    base_rate: f64,
    current_rate: f64,
    warmup_steps: usize,
    cycle_length: usize,
    current_step: usize,

    // Gradient statistics for adaptation
    grad_history: VecDeque<GradientStats>,
    loss_history: VecDeque<f64>,
}

pub struct GradientStats {
    pub magnitude: f64,      // ||∇L||
    pub variance: f64,       // Var(∇L) across batch
    pub snr: f64,            // Signal-to-noise ratio
}

impl EnhancedAdaptiveLR {
    /// Compute learning rate with warmup + cosine annealing + restarts
    pub fn get_rate(&self) -> f64 {
        let step = self.current_step;

        // Warmup phase
        if step < self.warmup_steps {
            return self.base_rate * (step as f64 / self.warmup_steps as f64);
        }

        // Cosine annealing with warm restarts (SGDR)
        let cycle_step = (step - self.warmup_steps) % self.cycle_length;
        let cycle_progress = cycle_step as f64 / self.cycle_length as f64;

        let min_rate = self.base_rate * 0.01;
        let cosine_factor = 0.5 * (1.0 + (PI * cycle_progress).cos());

        min_rate + (self.current_rate - min_rate) * cosine_factor
    }

    /// Adapt base rate based on gradient signal-to-noise ratio
    pub fn adapt_based_on_gradients(&mut self, stats: GradientStats) {
        self.grad_history.push_back(stats);

        if self.grad_history.len() >= 10 {
            let avg_snr = self.grad_history.iter()
                .map(|s| s.snr)
                .sum::<f64>() / self.grad_history.len() as f64;

            // High SNR = clear gradient signal = can use higher LR
            // Low SNR = noisy gradients = need lower LR
            if avg_snr > 5.0 {
                self.current_rate = (self.current_rate * 1.1).min(self.base_rate * 10.0);
            } else if avg_snr < 1.0 {
                self.current_rate = (self.current_rate * 0.8).max(self.base_rate * 0.01);
            }
        }
    }
}
```

---

## Level 2: Core Meta-Learning

### 2.1 Reptile-Based Initialization Learning

**Goal**: Learn an NCA initialization θ* that can quickly adapt to any pattern.

```rust
pub struct ReptileMetaLearner {
    meta_weights: NetworkWeights,    // θ - the meta-initialization
    meta_lr: f64,                    // ε - meta learning rate
    inner_steps: usize,              // k - SGD steps per task
    inner_lr: f64,                   // α - task learning rate
    task_batch_size: usize,          // Number of tasks per meta-update
}

impl ReptileMetaLearner {
    /// One meta-learning iteration
    pub fn meta_step(&mut self, nca: &mut NCA, patterns: &[PatternTask]) {
        let mut weight_deltas: Vec<NetworkWeights> = Vec::new();

        // Sample batch of tasks
        let tasks = self.sample_tasks(patterns, self.task_batch_size);

        for task in tasks {
            // Clone current meta-weights
            nca.set_weights(self.meta_weights.clone());

            // Inner loop: Train on this task for k steps
            for _ in 0..self.inner_steps {
                let target = task.generate_target();
                nca.train_step(&target, self.inner_lr);
            }

            // Compute weight delta: θ' - θ
            let adapted_weights = nca.get_weights();
            let delta = adapted_weights.subtract(&self.meta_weights);
            weight_deltas.push(delta);
        }

        // Average deltas and update meta-weights
        let avg_delta = NetworkWeights::average(&weight_deltas);
        self.meta_weights = self.meta_weights.add(&avg_delta.scale(self.meta_lr));
    }

    /// Adapt to new pattern quickly using meta-initialization
    pub fn adapt_to_pattern(&self, nca: &mut NCA, pattern: &PatternTask) -> f64 {
        nca.set_weights(self.meta_weights.clone());

        let mut final_loss = f64::MAX;
        for _ in 0..self.inner_steps {
            let target = pattern.generate_target();
            final_loss = nca.train_step(&target, self.inner_lr);
        }

        final_loss
    }
}
```

### 2.2 Population Based Training

**Goal**: Discover optimal hyperparameter schedules through evolution.

```rust
pub struct PopulationBasedTraining {
    population: Vec<PopulationMember>,
    population_size: usize,
    exploit_fraction: f64,  // Bottom 20% copy from top 20%
    explore_mutations: Vec<MutationType>,
    evaluation_interval: usize,
}

pub struct PopulationMember {
    pub id: usize,
    pub weights: NetworkWeights,
    pub hyperparams: Hyperparameters,
    pub score: f64,
    pub generation: usize,
    pub lineage: Vec<usize>,  // Track ancestry
}

#[derive(Clone)]
pub struct Hyperparameters {
    pub learning_rate: f64,
    pub evolution_steps: usize,
    pub batch_size: usize,
    pub pool_sample_rate: f64,
    pub stochastic_update_rate: f64,
    pub hidden_channels: usize,
}

pub enum MutationType {
    ScaleLR(f64),           // Multiply LR by factor
    PerturbLR(f64),         // Add Gaussian noise to LR
    ChangeEvolutionSteps(i32),
    ChangeBatchSize(i32),
    ChangePoolRate(f64),
}

impl PopulationBasedTraining {
    /// Evaluate and evolve population
    pub fn evolve(&mut self, eval_fn: impl Fn(&mut NCA, &Hyperparameters) -> f64) {
        // Evaluate all members
        for member in &mut self.population {
            let mut nca = NCA::from_weights(&member.weights);
            member.score = eval_fn(&mut nca, &member.hyperparams);
            member.weights = nca.get_weights();
        }

        // Sort by score (lower is better for loss)
        self.population.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        // Exploit: Bottom performers copy from top
        let n = self.population_size;
        let top_cutoff = (n as f64 * self.exploit_fraction) as usize;
        let bottom_start = n - top_cutoff;

        for i in bottom_start..n {
            // Select random member from top performers
            let source_idx = rand::random::<usize>() % top_cutoff;

            // Copy weights and hyperparams
            self.population[i].weights = self.population[source_idx].weights.clone();
            self.population[i].hyperparams = self.population[source_idx].hyperparams.clone();
            self.population[i].lineage = self.population[source_idx].lineage.clone();
            self.population[i].lineage.push(self.population[source_idx].id);

            // Explore: Mutate hyperparams
            self.mutate_hyperparams(&mut self.population[i]);
        }
    }

    fn mutate_hyperparams(&self, member: &mut PopulationMember) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Apply 1-3 random mutations
        let num_mutations = rng.gen_range(1..=3);
        for _ in 0..num_mutations {
            match rng.gen_range(0..5) {
                0 => member.hyperparams.learning_rate *= 1.2_f64.powf(rng.gen_range(-1.0..1.0)),
                1 => member.hyperparams.evolution_steps =
                    (member.hyperparams.evolution_steps as i32 + rng.gen_range(-20..20)).max(50) as usize,
                2 => member.hyperparams.batch_size =
                    (member.hyperparams.batch_size as i32 + rng.gen_range(-2..2)).clamp(1, 10) as usize,
                3 => member.hyperparams.pool_sample_rate =
                    (member.hyperparams.pool_sample_rate + rng.gen_range(-0.1..0.1)).clamp(0.0, 0.5),
                4 => member.hyperparams.stochastic_update_rate =
                    (member.hyperparams.stochastic_update_rate + rng.gen_range(-0.1..0.1)).clamp(0.3, 0.7),
                _ => {}
            }
        }
    }

    /// Get best hyperparameters found so far
    pub fn get_best_hyperparams(&self) -> &Hyperparameters {
        &self.population[0].hyperparams
    }

    /// Get hyperparameter schedule (how best params evolved over time)
    pub fn get_schedule(&self) -> Vec<(usize, Hyperparameters)> {
        self.population[0].lineage.iter()
            .enumerate()
            .map(|(gen, id)| (gen, self.get_historic_params(*id)))
            .collect()
    }
}
```

### 2.3 Learned Optimizer (L2O)

**Goal**: Replace hand-crafted optimizer with a learned update rule.

```rust
pub struct LearnedOptimizer {
    /// LSTM that produces weight updates
    update_network: LSTM,
    /// Hidden state maintained across training
    hidden_state: Vec<f64>,
    /// Parameters for the optimizer itself
    optimizer_params: Vec<f64>,
    /// Learning rate for meta-training the optimizer
    meta_lr: f64,
}

impl LearnedOptimizer {
    /// Produce weight update given gradient and history
    pub fn get_update(&mut self, gradient: &[f64], loss: f64) -> Vec<f64> {
        // Input features: gradient, loss, gradient magnitude, etc.
        let grad_magnitude = gradient.iter().map(|g| g * g).sum::<f64>().sqrt();
        let grad_mean = gradient.iter().sum::<f64>() / gradient.len() as f64;

        let input = vec![
            grad_magnitude.ln(),           // Log gradient magnitude
            grad_mean,                     // Mean gradient
            loss.ln(),                     // Log loss
            self.hidden_state[0],          // Previous momentum-like state
        ];

        // LSTM produces update direction and step size
        let (output, new_hidden) = self.update_network.forward(&input, &self.hidden_state);
        self.hidden_state = new_hidden;

        // Output: [direction_scale, step_size] per coordinate
        // Update = gradient * sigmoid(direction_scale) * exp(step_size)
        gradient.iter().enumerate().map(|(i, g)| {
            let direction = sigmoid(output[i * 2]);
            let step_size = output[i * 2 + 1].exp().min(0.1);  // Clamp max step
            -g * direction * step_size
        }).collect()
    }

    /// Meta-train the optimizer on a distribution of tasks
    pub fn meta_train(&mut self, tasks: &[PatternTask], nca: &mut NCA) {
        // Unroll optimizer on tasks and backprop through entire trajectory
        // (Simplified - full implementation would use BPTT)

        for task in tasks {
            self.hidden_state = vec![0.0; self.hidden_state.len()];
            let mut trajectory_loss = 0.0;

            for step in 0..20 {
                let target = task.generate_target();
                let gradient = nca.compute_gradient(&target);
                let loss = nca.compute_loss(&target);

                let update = self.get_update(&gradient, loss);
                nca.apply_update(&update);

                // Accumulate loss for meta-gradient
                trajectory_loss += loss * (0.9_f64.powi(20 - step as i32));  // Discount
            }

            // Backprop through trajectory to update optimizer params
            self.update_optimizer_params(trajectory_loss);
        }
    }
}
```

---

## Level 3: Meta-Meta-Learning

### 3.1 Strategy Selection Network

**Goal**: Learn when to use which meta-learning strategy.

```rust
pub struct MetaStrategySelector {
    /// Neural network that predicts best strategy given context
    strategy_network: NeuralNetwork,
    /// History of (context, strategy, outcome) tuples
    experience: Vec<StrategyExperience>,
    /// Available strategies
    strategies: Vec<MetaStrategy>,
}

pub enum MetaStrategy {
    Reptile,           // Good for few-shot adaptation
    PBT,               // Good for hyperparameter discovery
    SelfPaced,         // Good for difficult patterns
    Standard,          // Standard Adam training
    Aggressive,        // High LR, fast convergence
    Conservative,      // Low LR, stable convergence
}

pub struct StrategyContext {
    pub current_loss: f64,
    pub loss_variance: f64,
    pub loss_trend: f64,           // Positive = improving
    pub pattern_difficulty: f64,
    pub training_progress: f64,    // 0-1 how far into training
    pub recent_novelty: f64,
    pub diversity: f64,
    pub complexity: f64,
}

impl MetaStrategySelector {
    /// Select strategy based on current context
    pub fn select_strategy(&self, context: &StrategyContext) -> MetaStrategy {
        let features = context.to_features();
        let predictions = self.strategy_network.predict(&features);

        // Softmax + sampling (exploration) or argmax (exploitation)
        let strategy_idx = if rand::random::<f64>() < 0.1 {
            // 10% exploration
            self.sample_from_softmax(&predictions)
        } else {
            predictions.iter().enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap().0
        };

        self.strategies[strategy_idx].clone()
    }

    /// Update strategy network based on observed outcomes
    pub fn learn_from_outcome(&mut self, context: StrategyContext, strategy: MetaStrategy, outcome: f64) {
        self.experience.push(StrategyExperience {
            context,
            strategy,
            outcome,
        });

        // Periodically retrain strategy network
        if self.experience.len() % 100 == 0 {
            self.retrain_network();
        }
    }
}
```

### 3.2 Architecture Self-Modification

**Goal**: SAGE can modify its own network architecture (DANGEROUS - requires safety constraints).

```rust
pub struct ArchitectureSelfModification {
    /// Current architecture specification
    architecture: NCArchitecture,
    /// Modification history for rollback
    history: Vec<ArchitectureSnapshot>,
    /// Safety constraints
    constraints: SafetyConstraints,
    /// Approval queue for major changes
    pending_approvals: Vec<ProposedModification>,
}

pub struct NCArchitecture {
    pub hidden_channels: usize,      // Currently 12
    pub perception_kernel_size: usize,  // Currently 3x3
    pub update_network_layers: Vec<usize>,  // Currently [128, 22]
    pub activation_function: ActivationType,
    pub stochastic_rate: f64,
}

pub struct SafetyConstraints {
    pub max_hidden_channels: usize,     // Don't exceed this
    pub min_hidden_channels: usize,     // Don't go below this
    pub max_parameter_change: f64,      // Max % change per modification
    pub require_approval_for_major: bool,
    pub max_modifications_per_hour: usize,
    pub performance_floor: f64,         // Must stay above this
}

pub struct ProposedModification {
    pub modification_type: ModificationType,
    pub rationale: String,
    pub predicted_impact: f64,
    pub timestamp: SystemTime,
    pub requires_approval: bool,
}

pub enum ModificationType {
    AddHiddenChannels(usize),
    RemoveHiddenChannels(usize),
    ChangeKernelSize(usize),
    AddNetworkLayer(usize),
    ChangeActivation(ActivationType),
    ChangeStochasticRate(f64),
}

impl ArchitectureSelfModification {
    /// Propose an architecture modification
    pub fn propose_modification(&mut self, diagnosis: &PerformanceDiagnosis) -> Option<ProposedModification> {
        let modification = match diagnosis {
            PerformanceDiagnosis::Plateau { .. } => {
                // Plateau might benefit from more capacity
                if self.architecture.hidden_channels < self.constraints.max_hidden_channels {
                    Some(ModificationType::AddHiddenChannels(2))
                } else {
                    None
                }
            }
            PerformanceDiagnosis::Unstable { .. } => {
                // Instability might benefit from reduced capacity or lower stochastic rate
                Some(ModificationType::ChangeStochasticRate(
                    self.architecture.stochastic_rate * 0.9
                ))
            }
            PerformanceDiagnosis::Stuck { .. } => {
                // Try different activation function
                Some(ModificationType::ChangeActivation(ActivationType::GELU))
            }
            _ => None,
        };

        modification.map(|m| ProposedModification {
            modification_type: m,
            rationale: format!("{:?}", diagnosis),
            predicted_impact: self.estimate_impact(&m),
            timestamp: SystemTime::now(),
            requires_approval: self.is_major_change(&m),
        })
    }

    /// Apply modification with safety checks
    pub fn apply_modification(&mut self, modification: &ProposedModification, nca: &mut NCA) -> Result<(), String> {
        // Check safety constraints
        if modification.requires_approval && !self.has_approval(modification) {
            return Err("Modification requires human approval".to_string());
        }

        if self.modifications_this_hour() >= self.constraints.max_modifications_per_hour {
            return Err("Too many modifications this hour".to_string());
        }

        // Save snapshot for rollback
        self.history.push(ArchitectureSnapshot {
            architecture: self.architecture.clone(),
            weights: nca.get_weights(),
            timestamp: SystemTime::now(),
        });

        // Apply modification
        match &modification.modification_type {
            ModificationType::AddHiddenChannels(n) => {
                self.architecture.hidden_channels += n;
                nca.resize_hidden_channels(self.architecture.hidden_channels);
            }
            // ... other modifications
        }

        // Log to SpacetimeDB
        self.log_modification(modification);

        Ok(())
    }

    /// Rollback to previous architecture if performance degraded
    pub fn rollback(&mut self, nca: &mut NCA) -> Result<(), String> {
        if let Some(snapshot) = self.history.pop() {
            self.architecture = snapshot.architecture;
            nca.set_weights(snapshot.weights);
            Ok(())
        } else {
            Err("No snapshots to rollback to".to_string())
        }
    }
}
```

---

## Integration: Unified Meta-Learning Controller

```rust
pub struct UnifiedMetaLearningController {
    // Level 1
    pub self_paced_curriculum: SelfPacedCurriculum,
    pub adaptive_lr: EnhancedAdaptiveLR,
    pub curiosity_system: CuriositySystem,

    // Level 2
    pub reptile: ReptileMetaLearner,
    pub pbt: PopulationBasedTraining,
    pub learned_optimizer: Option<LearnedOptimizer>,

    // Level 3
    pub strategy_selector: MetaStrategySelector,
    pub architecture_mod: ArchitectureSelfModification,

    // State
    pub current_strategy: MetaStrategy,
    pub generation: u64,
}

impl UnifiedMetaLearningController {
    /// Main training loop with meta-learning
    pub fn train_step(&mut self, nca: &mut NCA) -> TrainingResult {
        // 1. Gather context
        let context = self.gather_context(nca);

        // 2. Select meta-strategy
        self.current_strategy = self.strategy_selector.select_strategy(&context);

        // 3. Select pattern using self-paced curriculum
        let pattern = self.self_paced_curriculum.select_pattern();

        // 4. Get hyperparameters (from PBT or fixed)
        let hyperparams = match self.current_strategy {
            MetaStrategy::PBT => self.pbt.get_best_hyperparams().clone(),
            _ => self.get_default_hyperparams(),
        };

        // 5. Get learning rate
        let lr = self.adaptive_lr.get_rate();

        // 6. Execute training based on strategy
        let result = match self.current_strategy {
            MetaStrategy::Reptile => {
                self.reptile.meta_step(nca, &[pattern.clone()]);
                self.reptile.adapt_to_pattern(nca, pattern)
            }
            MetaStrategy::PBT => {
                // PBT manages its own population
                self.pbt.evolve(|nca, hp| self.evaluate(nca, hp, pattern));
                self.evaluate(nca, &hyperparams, pattern)
            }
            _ => {
                // Standard training with adaptive LR
                let target = pattern.generate_target();
                nca.train_step(&target, lr)
            }
        };

        // 7. Update meta-learning systems
        self.update_systems(result, &context, pattern);

        // 8. Check for architecture modifications (periodic)
        if self.generation % 1000 == 0 {
            self.consider_architecture_modification(nca);
        }

        self.generation += 1;
        result
    }

    fn update_systems(&mut self, result: TrainingResult, context: &StrategyContext, pattern: &PatternTask) {
        // Update curriculum threshold
        let success = result.loss < 0.1;
        self.self_paced_curriculum.update_threshold(if success { 0.8 } else { 0.4 });

        // Update adaptive LR
        self.adaptive_lr.adapt_based_on_gradients(result.grad_stats);

        // Update strategy selector
        self.strategy_selector.learn_from_outcome(
            context.clone(),
            self.current_strategy.clone(),
            -result.loss,  // Negative loss = positive outcome
        );

        // Update curiosity
        self.curiosity_system.record_observation(&result.final_grid, self.generation);
    }
}
```

---

## Implementation Roadmap

### Phase A: Enhanced Adaptive Learning (1-2 weeks)
1. Implement `SelfPacedCurriculum` with dynamic difficulty thresholds
2. Implement `EnhancedAdaptiveLR` with warmup and cosine annealing
3. Integrate with existing training loop
4. Test: Verify faster convergence on all patterns

### Phase B: Reptile Meta-Learning (2-3 weeks)
1. Implement `ReptileMetaLearner` with task sampling
2. Create few-shot adaptation benchmark
3. Test: Can SAGE learn new patterns in <10 gradient steps?
4. Measure transfer learning improvement

### Phase C: Population Based Training (2-3 weeks)
1. Implement `PopulationBasedTraining` with exploit/explore
2. Run hyperparameter discovery experiments
3. Extract discovered schedules
4. Test: Do PBT schedules outperform hand-tuned ones?

### Phase D: Meta-Strategy Selection (3-4 weeks)
1. Implement `MetaStrategySelector` with experience replay
2. Gather strategy performance data
3. Train strategy selection network
4. Test: Does adaptive strategy selection improve overall performance?

### Phase E: Architecture Self-Modification (4-6 weeks)
1. Implement `ArchitectureSelfModification` with safety constraints
2. Add SpacetimeDB logging for all modifications
3. Implement rollback capability
4. Create approval workflow for major changes
5. Test extensively in sandboxed environment

### Phase F: Learned Optimizer (6-8 weeks)
1. Implement LSTM-based learned optimizer
2. Meta-train on distribution of NCA tasks
3. Compare against Adam baseline
4. Test generalization to unseen patterns

---

## Safety Considerations

### Architecture Self-Modification Safety

1. **Logging**: ALL modifications logged to SpacetimeDB with timestamps
2. **Rollback**: Snapshots before each modification enable instant rollback
3. **Rate Limiting**: Maximum N modifications per hour
4. **Performance Floor**: Automatic rollback if performance drops below threshold
5. **Human Approval**: Major changes (>10% parameter change) require approval
6. **Sandboxing**: Test modifications in isolated copy before applying

### Monitoring Dashboard

Add TUI screen showing:
- Current meta-strategy in use
- Recent strategy selection history
- Architecture modification history
- PBT population diversity
- Reptile adaptation speed

---

## Success Metrics

1. **Few-Shot Adaptation**: Learn new pattern in <10 steps (vs current ~100)
2. **Curriculum Efficiency**: 30% faster to master all patterns
3. **Hyperparameter Discovery**: PBT finds better configs than hand-tuning
4. **Transfer Learning**: New patterns benefit from previous learning
5. **Self-Stability**: Architecture modifications improve, don't hurt, performance

---

## References

1. Finn et al. (2017). "Model-Agnostic Meta-Learning for Fast Adaptation"
2. Nichol et al. (2018). "On First-Order Meta-Learning Algorithms" (Reptile)
3. Jaderberg et al. (2017). "Population Based Training of Neural Networks"
4. Kumar et al. (2010). "Self-Paced Learning for Latent Variable Models"
5. Andrychowicz et al. (2016). "Learning to Learn by Gradient Descent"
6. Mordvintsev et al. (2020). "Growing Neural Cellular Automata"
7. Sudhakaran et al. (2022). "Goal-Guided Neural Cellular Automata"
8. Alexander et al. (2025). "A Path to Universal Neural Cellular Automata"

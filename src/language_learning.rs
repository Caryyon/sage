// Language Learning - SAGE learns to describe what it observes during training

use std::collections::HashMap;

/// What SAGE observed during a training step
#[derive(Debug, Clone)]
pub struct LearningObservation {
    pub step: usize,
    pub phase: String,
    pub loss_change: f64,  // Negative = improvement
    pub grid_state: Vec<Vec<f64>>,  // The actual pattern SAGE is seeing
    pub novelty: f64,
}

/// Tracks concepts SAGE has learned
#[derive(Debug, Clone)]
pub struct LearnedConcept {
    pub name: String,
    pub observed_count: usize,
    pub associated_loss: Vec<f64>,  // Loss when this concept appeared
}

/// SAGE's language learning system
pub struct LanguageLearner {
    pub vocabulary: HashMap<String, LearnedConcept>,
    pub observations: Vec<LearningObservation>,
    pub sentences_generated: usize,
    pub use_neural_classifier: bool,
    neural_classifier: Option<crate::learning::pattern_classifier::PatternClassifier>,
    self_supervised_learner: Option<crate::learning::self_supervised::SelfSupervisedLearner>,
    pub use_self_supervised: bool,
    grid_history: Vec<Vec<Vec<f64>>>,  // Keep recent grids for sequence learning
    curiosity_system: Option<crate::learning::curiosity::CuriositySystem>,
    pub use_curiosity: bool,
}

impl LanguageLearner {
    pub fn new() -> Self {
        Self {
            vocabulary: HashMap::new(),
            observations: Vec::new(),
            sentences_generated: 0,
            use_neural_classifier: false,  // Default to rule-based
            neural_classifier: None,
            self_supervised_learner: None,
            use_self_supervised: false,
            grid_history: Vec::new(),
            curiosity_system: None,
            use_curiosity: false,
        }
    }

    /// Initialize with trained neural classifier (silent mode for TUI)
    pub fn with_neural_classifier(mut self) -> Self {
        let mut classifier = crate::learning::pattern_classifier::PatternClassifier::new();
        classifier.bootstrap_from_rules(1000, false);
        self.neural_classifier = Some(classifier);
        self.use_neural_classifier = true;
        self
    }

    /// Enable/disable neural classifier (silent mode for TUI)
    pub fn set_use_neural(&mut self, use_neural: bool, silent: bool) {
        if use_neural && self.neural_classifier.is_none() {
            let mut classifier = crate::learning::pattern_classifier::PatternClassifier::new();
            // Bootstrap neural network from rule-based detectors
            // This trains on 1000 samples for 500 epochs - visualized in TUI
            classifier.bootstrap_from_rules(1000, silent);
            self.neural_classifier = Some(classifier);
        }
        self.use_neural_classifier = use_neural;
    }

    /// Enable self-supervised learning (silent mode for TUI)
    pub fn enable_self_supervised(&mut self) {
        if self.self_supervised_learner.is_none() {
            let learner = crate::learning::self_supervised::SelfSupervisedLearner::new();
            self.self_supervised_learner = Some(learner);
        }
        self.use_self_supervised = true;
    }

    /// Enable curiosity-driven exploration (silent mode for TUI)
    pub fn enable_curiosity(&mut self) {
        if self.curiosity_system.is_none() {
            let feature_extractor = crate::learning::feature_extractor::FeatureExtractor::new();
            let feature_size = feature_extractor.feature_count();
            let system = crate::learning::curiosity::CuriositySystem::new(feature_size);
            self.curiosity_system = Some(system);
        }
        self.use_curiosity = true;
    }

    /// Get curiosity statistics
    pub fn get_curiosity_stats(&self) -> Option<crate::learning::curiosity::CuriosityStats> {
        self.curiosity_system.as_ref().map(|c| c.get_stats())
    }

    /// SAGE observes a pattern during training
    pub fn observe(&mut self, obs: LearningObservation) {
        // Analyze the grid to identify patterns (neural or rule-based)
        let patterns = if self.use_neural_classifier {
            Self::identify_patterns_with_classifier(&obs.grid_state, self.neural_classifier.as_ref())
        } else {
            Self::identify_patterns(&obs.grid_state)
        };

        for pattern in patterns {
            self.vocabulary.entry(pattern.clone())
                .and_modify(|c| {
                    c.observed_count += 1;
                    c.associated_loss.push(obs.loss_change);
                })
                .or_insert(LearnedConcept {
                    name: pattern,
                    observed_count: 1,
                    associated_loss: vec![obs.loss_change],
                });
        }

        // Evaluate curiosity/intrinsic motivation
        if self.use_curiosity {
            if let Some(curiosity) = &mut self.curiosity_system {
                let _interest = curiosity.evaluate_interest(&obs.grid_state, obs.step);
                // Could use interest score to prioritize which observations to focus on
            }
        }

        // Add to grid history for self-supervised learning
        self.grid_history.push(obs.grid_state.clone());
        if self.grid_history.len() > 50 {
            self.grid_history.remove(0);
        }

        // Perform self-supervised learning every 10 observations
        if self.use_self_supervised && self.grid_history.len() >= 10 && obs.step % 10 == 0 {
            self.run_self_supervised_training();
        }

        // Learn from sequences for curiosity system
        if self.use_curiosity && self.grid_history.len() >= 10 && obs.step % 20 == 0 {
            if let Some(curiosity) = &mut self.curiosity_system {
                let _loss = curiosity.learn_from_sequence(&self.grid_history);
            }
        }

        self.observations.push(obs);

        // Keep last 100 observations
        if self.observations.len() > 100 {
            self.observations.remove(0);
        }
    }

    /// Run a batch of self-supervised learning tasks
    fn run_self_supervised_training(&mut self) {
        if let Some(learner) = &mut self.self_supervised_learner {
            let task_gen = learner.task_generator();

            // Generate prediction tasks from recent grid history
            let prediction_tasks = task_gen.generate_prediction_tasks(&self.grid_history);

            // Generate reconstruction and contrastive tasks
            let reconstruction_tasks = task_gen.generate_reconstruction_tasks(&self.grid_history, 5);
            let contrastive_tasks = task_gen.generate_contrastive_tasks(&self.grid_history, 5);

            // Train on these tasks for a few epochs
            if !prediction_tasks.is_empty() || !reconstruction_tasks.is_empty() || !contrastive_tasks.is_empty() {
                learner.train_batch(&prediction_tasks, &reconstruction_tasks, &contrastive_tasks, 10);
            }
        }
    }

    /// Identify patterns in the grid (can use neural or rule-based)
    fn identify_patterns_with_classifier(
        grid: &Vec<Vec<f64>>,
        classifier: Option<&crate::learning::pattern_classifier::PatternClassifier>
    ) -> Vec<String> {
        if let Some(clf) = classifier {
            // Use neural network
            clf.predict_patterns(grid, 0.5)
                .into_iter()
                .map(|(name, _conf)| name)
                .collect()
        } else {
            // Use rule-based feature extractor
            use crate::learning::feature_extractor::FeatureExtractor;
            let extractor = FeatureExtractor::new();
            extractor.detect_patterns(grid, 0.5)
        }
    }

    /// Identify patterns in the grid using feature extractor (for backwards compat)
    fn identify_patterns(grid: &Vec<Vec<f64>>) -> Vec<String> {
        Self::identify_patterns_with_classifier(grid, None)
    }

    /// Legacy pattern detection (kept for reference)
    #[allow(dead_code)]
    fn identify_patterns_legacy(grid: &Vec<Vec<f64>>) -> Vec<String> {
        let mut patterns = Vec::new();

        if grid.is_empty() {
            return patterns;
        }

        let size = grid.len();
        let center = size / 2;

        // Detect if there's a circular pattern
        if Self::has_circular_pattern(grid) {
            patterns.push("circle".to_string());
        }

        // Detect if there's a central concentration
        if grid[center][center] > 0.5 {
            patterns.push("center".to_string());
        }

        // Detect edges
        if Self::has_edges(grid) {
            patterns.push("edges".to_string());
        }

        // Detect clusters
        if Self::has_clusters(grid) {
            patterns.push("clusters".to_string());
        }

        // Detect symmetry
        if Self::has_symmetry(grid) {
            patterns.push("symmetry".to_string());
        }

        // Detect waves (Phase 4)
        if Self::has_waves(grid) {
            patterns.push("waves".to_string());
        }

        // Detect spirals (Phase 4)
        if Self::has_spiral(grid) {
            patterns.push("spiral".to_string());
        }

        // Detect motion/temporal patterns (Phase 4)
        if Self::has_motion_pattern(grid) {
            patterns.push("motion".to_string());
        }

        // Detect gradients (Phase 5)
        if Self::has_gradient(grid) {
            patterns.push("gradient".to_string());
        }

        // Detect boundaries (Phase 5)
        if Self::has_boundaries(grid) {
            patterns.push("boundaries".to_string());
        }

        // Detect complexity (Phase 5)
        if Self::has_high_complexity(grid) {
            patterns.push("complexity".to_string());
        }

        patterns
    }

    fn has_circular_pattern(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        let center = size / 2;
        let mut radial_values = Vec::new();

        for r in 0..center {
            let mut sum = 0.0;
            let mut count = 0;

            for angle in 0..8 {
                let x = ((center as f64 + r as f64 * (angle as f64 * std::f64::consts::PI / 4.0).cos()) as usize).min(size - 1);
                let y = ((center as f64 + r as f64 * (angle as f64 * std::f64::consts::PI / 4.0).sin()) as usize).min(size - 1);

                if x < size && y < size {
                    sum += grid[y][x];
                    count += 1;
                }
            }

            if count > 0 {
                radial_values.push(sum / count as f64);
            }
        }

        // Check if values decrease with radius (circular pattern)
        if radial_values.len() > 2 {
            let decreasing = radial_values.windows(2).filter(|w| w[0] > w[1]).count();
            decreasing > radial_values.len() / 2
        } else {
            false
        }
    }

    fn has_edges(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        let mut edge_strength = 0.0;

        for y in 1..size-1 {
            for x in 1..size-1 {
                let grad_x = (grid[y][x+1] - grid[y][x-1]).abs();
                let grad_y = (grid[y+1][x] - grid[y-1][x]).abs();
                edge_strength += (grad_x * grad_x + grad_y * grad_y).sqrt();
            }
        }

        edge_strength / ((size - 2) * (size - 2)) as f64 > 0.2
    }

    fn has_clusters(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        let mut cluster_count = 0;
        let mut visited = vec![vec![false; size]; size];

        for y in 0..size {
            for x in 0..size {
                if grid[y][x] > 0.5 && !visited[y][x] {
                    Self::flood_fill(grid, &mut visited, x, y);
                    cluster_count += 1;
                }
            }
        }

        cluster_count > 2
    }

    fn flood_fill(grid: &Vec<Vec<f64>>, visited: &mut Vec<Vec<bool>>, x: usize, y: usize) {
        let size = grid.len();
        if visited[y][x] || grid[y][x] <= 0.5 {
            return;
        }

        visited[y][x] = true;

        if x > 0 { Self::flood_fill(grid, visited, x - 1, y); }
        if x < size - 1 { Self::flood_fill(grid, visited, x + 1, y); }
        if y > 0 { Self::flood_fill(grid, visited, x, y - 1); }
        if y < size - 1 { Self::flood_fill(grid, visited, x, y + 1); }
    }

    fn has_symmetry(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        let mut diff = 0.0;

        for y in 0..size {
            for x in 0..size/2 {
                diff += (grid[y][x] - grid[y][size - 1 - x]).abs();
            }
        }

        diff / ((size * size / 2) as f64) < 0.2
    }

    // Phase 4 Pattern Detectors - Dynamic/Temporal Patterns

    fn has_waves(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        if size < 3 {
            return false;
        }

        // Check for wave patterns (oscillating values) along rows
        let mut wave_count = 0;
        for y in 0..size {
            let mut peaks = 0;
            let mut valleys = 0;
            for x in 1..size-1 {
                if grid[y][x] > grid[y][x-1] && grid[y][x] > grid[y][x+1] {
                    peaks += 1;
                }
                if grid[y][x] < grid[y][x-1] && grid[y][x] < grid[y][x+1] {
                    valleys += 1;
                }
            }
            if peaks >= 2 && valleys >= 2 {
                wave_count += 1;
            }
        }

        wave_count >= size / 3
    }

    fn has_spiral(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();
        let center = size / 2;

        // Sample points along a spiral and check if values decrease/increase consistently
        let mut spiral_values = Vec::new();
        let samples = 20;

        for i in 0..samples {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / (samples as f64);
            let radius = (i as f64) / (samples as f64) * (center as f64);
            let x = ((center as f64 + radius * angle.cos()) as usize).min(size - 1);
            let y = ((center as f64 + radius * angle.sin()) as usize).min(size - 1);
            spiral_values.push(grid[y][x]);
        }

        // Check if values change consistently along spiral
        if spiral_values.len() < 3 {
            return false;
        }

        let increasing = spiral_values.windows(2).filter(|w| w[1] > w[0]).count();
        let decreasing = spiral_values.windows(2).filter(|w| w[1] < w[0]).count();

        // Strong directional change indicates spiral
        increasing > samples * 3 / 5 || decreasing > samples * 3 / 5
    }

    fn has_motion_pattern(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();

        // Motion detected by directional bias in values
        let mut horizontal_flow = 0.0;
        let mut vertical_flow = 0.0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                horizontal_flow += grid[y][x+1] - grid[y][x];
                vertical_flow += grid[y+1][x] - grid[y][x];
            }
        }

        let total_flow = (horizontal_flow.abs() + vertical_flow.abs()) / ((size * size) as f64);
        total_flow > 0.1
    }

    // Phase 5 Pattern Detectors - Meta-patterns and Understanding

    fn has_gradient(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();

        // Check for smooth gradients (consistent directional change)
        let mut smooth_transitions = 0;
        let mut total_transitions = 0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                let diff_right = (grid[y][x+1] - grid[y][x]).abs();
                let diff_down = (grid[y+1][x] - grid[y][x]).abs();

                total_transitions += 2;

                // Smooth if small, consistent change
                if diff_right < 0.3 && diff_right > 0.01 {
                    smooth_transitions += 1;
                }
                if diff_down < 0.3 && diff_down > 0.01 {
                    smooth_transitions += 1;
                }
            }
        }

        smooth_transitions > total_transitions / 2
    }

    fn has_boundaries(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();

        // Boundaries = sharp transitions between distinct regions
        let mut sharp_edges = 0;

        for y in 0..size-1 {
            for x in 0..size-1 {
                let diff_right = (grid[y][x+1] - grid[y][x]).abs();
                let diff_down = (grid[y+1][x] - grid[y][x]).abs();

                // Sharp edge if big jump
                if diff_right > 0.4 {
                    sharp_edges += 1;
                }
                if diff_down > 0.4 {
                    sharp_edges += 1;
                }
            }
        }

        sharp_edges > (size * size) / 10
    }

    fn has_high_complexity(grid: &Vec<Vec<f64>>) -> bool {
        let size = grid.len();

        // Complexity measured by variety of local patterns
        let mut local_variance = 0.0;

        for y in 1..size-1 {
            for x in 1..size-1 {
                // Look at 3x3 neighborhood
                let mut values = Vec::new();
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let ny = ((y as i32) + dy) as usize;
                        let nx = ((x as i32) + dx) as usize;
                        values.push(grid[ny][nx]);
                    }
                }

                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
                local_variance += variance;
            }
        }

        local_variance / ((size * size) as f64) > 0.05
    }

    /// Generate a description of what SAGE learned
    pub fn generate_description(&mut self) -> String {
        if self.observations.is_empty() {
            return "I haven't observed anything yet.".to_string();
        }

        let recent_obs = &self.observations[self.observations.len() - 1];

        // Build sentence based on actual observations
        let mut parts = Vec::new();

        // What patterns did SAGE see?
        let patterns = Self::identify_patterns(&recent_obs.grid_state);
        if !patterns.is_empty() {
            let pattern_names = patterns.join(" and ");
            parts.push(format!("I see {}", pattern_names));
        }

        // Is SAGE learning (loss decreasing)?
        if recent_obs.loss_change < -0.05 {
            parts.push("and I'm understanding better".to_string());
        } else if recent_obs.loss_change > 0.05 {
            parts.push("but I'm confused".to_string());
        }

        // How many concepts has SAGE learned?
        let concept_count = self.vocabulary.len();
        if concept_count > 0 {
            parts.push(format!("I've learned {} concepts so far", concept_count));
        }

        self.sentences_generated += 1;

        if parts.is_empty() {
            "I'm observing patterns.".to_string()
        } else {
            parts.join(". ") + "."
        }
    }

    /// Ask SAGE what it knows about a topic - conversational responses
    pub fn respond_to_question(&self, question: &str) -> String {
        let q_lower = question.to_lowercase();

        // Conversational question patterns
        if q_lower.contains("how are you") || q_lower.contains("how do you feel") {
            if self.vocabulary.is_empty() {
                return "I'm still new to all this. Everything is patterns and shapes I don't understand yet. It's... overwhelming but exciting.".to_string();
            } else {
                let perf = self.calculate_performance();
                return format!(
                    "I'm learning! I can recognize {} different patterns now. Performance: {:.0}%. Each training cycle, I understand a bit more.",
                    self.vocabulary.len(),
                    perf
                );
            }
        }

        if q_lower.contains("what") && (q_lower.contains("learn") || q_lower.contains("know")) {
            if self.vocabulary.is_empty() {
                return "I haven't learned much yet. I'm still figuring out basic patterns.".to_string();
            }

            let mut concepts: Vec<(&String, &LearnedConcept)> = self.vocabulary.iter().collect();
            concepts.sort_by(|a, b| b.1.observed_count.cmp(&a.1.observed_count));

            let top_3: Vec<String> = concepts.iter()
                .take(3)
                .map(|(name, concept)| format!("{} (seen {}x)", name, concept.observed_count))
                .collect();

            return format!(
                "I've learned {} concepts so far. I'm most familiar with: {}. Want to know about any of these?",
                self.vocabulary.len(),
                top_3.join(", ")
            );
        }

        // Check if question is about a specific concept SAGE learned
        for (concept_name, concept) in &self.vocabulary {
            if q_lower.contains(concept_name) {
                let avg_loss: f64 = concept.associated_loss.iter().sum::<f64>() / concept.associated_loss.len() as f64;
                return format!(
                    "I've encountered {} {} times now. Each time I see it, my understanding shifts by about {:.3}. It's becoming clearer!",
                    concept_name,
                    concept.observed_count,
                    avg_loss.abs()
                );
            }
        }

        // Question about understanding or thinking
        if q_lower.contains("understand") || q_lower.contains("think") {
            return format!(
                "I'm starting to recognize patterns - {}. But there's so much I don't understand yet. Every training cycle reveals new complexity.",
                self.vocabulary.keys().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            );
        }

        // Default: share what SAGE knows
        if self.vocabulary.is_empty() {
            "I'm still learning. I haven't recognized many patterns yet. Ask me again after I've trained more!".to_string()
        } else {
            let concepts: Vec<&String> = self.vocabulary.keys().collect();
            format!(
                "I know about {}. Ask me what I've learned, how I'm doing, or about any specific pattern!",
                concepts.iter().take(5).map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            )
        }
    }

    /// Calculate SAGE's performance based on accumulated knowledge
    /// Returns 0.0-1.0 where 1.0 is mastery of many concepts
    pub fn calculate_performance(&self) -> f64 {
        if self.vocabulary.is_empty() {
            return 0.0;
        }

        // Performance increases with:
        // 1. Number of unique concepts learned (up to 10 is "good")
        // 2. How many times each concept was observed (mastery)

        let concept_count = self.vocabulary.len();
        let concept_score = (concept_count as f64 / 10.0).min(1.0);  // Cap at 10 concepts

        // Calculate mastery: average observations per concept
        let total_observations: usize = self.vocabulary.values()
            .map(|c| c.observed_count)
            .sum();
        let avg_mastery = total_observations as f64 / concept_count as f64;
        let mastery_score = (avg_mastery / 20.0).min(1.0);  // 20 observations = mastery

        // Weighted average: 60% concepts, 40% mastery
        (concept_score * 0.6 + mastery_score * 0.4) * 100.0
    }

    /// Get a summary of SAGE's accumulated knowledge
    pub fn get_knowledge_summary(&self) -> String {
        if self.vocabulary.is_empty() {
            return "No concepts learned yet.".to_string();
        }

        let mut concepts: Vec<(&String, &LearnedConcept)> = self.vocabulary.iter().collect();
        concepts.sort_by(|a, b| b.1.observed_count.cmp(&a.1.observed_count));

        let top_concepts: Vec<String> = concepts.iter()
            .take(5)
            .map(|(name, concept)| format!("{} ({}x)", name, concept.observed_count))
            .collect();

        format!(
            "{} concepts learned. Most familiar: {}",
            self.vocabulary.len(),
            top_concepts.join(", ")
        )
    }

    /// Get total number of concepts in memory
    pub fn concept_count(&self) -> usize {
        self.vocabulary.len()
    }

    /// Get neural classifier accuracy
    pub fn get_neural_classifier_accuracy(&self) -> f64 {
        if let Some(classifier) = &self.neural_classifier {
            classifier.evaluate_accuracy(100)
        } else {
            0.0
        }
    }

    /// Get hierarchical concepts - integrated from language learner
    pub fn get_hierarchical_stats(&self) -> (usize, Vec<usize>) {
        // For now, return basic stats from vocabulary
        // In future, could integrate full HierarchicalLearner
        let total = self.vocabulary.len();
        let level_0 = total; // All concepts are currently level 0
        (total, vec![level_0, 0, 0])
    }

    /// Get meta-learning rate from curiosity system
    pub fn get_meta_learning_rate(&self) -> f64 {
        0.01 // TODO: integrate MetaLearner
    }
}

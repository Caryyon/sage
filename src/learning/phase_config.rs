// Config-driven phase system for scalable learning

use std::sync::Arc;

/// A training phase definition
#[derive(Clone)]
pub struct PhaseDefinition {
    pub id: String,
    pub name: String,
    pub duration_steps: usize,
    pub difficulty: f64,
    pub pattern_generators: Vec<Arc<dyn PatternGenerator>>,
    pub expected_patterns: Vec<String>,
}

/// Grid type for pattern generation
pub type Grid = Vec<Vec<f64>>;

/// Trait for generating training patterns
pub trait PatternGenerator: Send + Sync {
    /// Generate a pattern grid for a given step
    fn generate(&self, step: usize, progress: f64) -> Grid;

    /// Get the name of this generator
    fn name(&self) -> &str;

    /// Get patterns this generator is designed to teach
    fn teaches_patterns(&self) -> Vec<String>;
}

impl PhaseDefinition {
    pub fn new(id: &str, name: &str, duration_steps: usize) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            duration_steps,
            difficulty: 1.0,
            pattern_generators: Vec::new(),
            expected_patterns: Vec::new(),
        }
    }

    pub fn with_difficulty(mut self, difficulty: f64) -> Self {
        self.difficulty = difficulty;
        self
    }

    pub fn with_generator(mut self, generator: Arc<dyn PatternGenerator>) -> Self {
        // Add patterns this generator teaches
        for pattern in generator.teaches_patterns() {
            if !self.expected_patterns.contains(&pattern) {
                self.expected_patterns.push(pattern);
            }
        }
        self.pattern_generators.push(generator);
        self
    }

    /// Generate a pattern for this phase
    pub fn generate_pattern(&self, step: usize, progress: f64) -> Grid {
        if self.pattern_generators.is_empty() {
            // Default: empty grid
            return vec![vec![0.0; 32]; 32];
        }

        // Use first generator for now (can be extended to blend multiple)
        self.pattern_generators[0].generate(step, progress)
    }
}

/// Load phase definitions from configuration
pub fn load_default_phases() -> Vec<PhaseDefinition> {
    use crate::learning::pattern_generators::*;

    vec![
        PhaseDefinition::new("phase1_primitives", "Phase 1: Primitives", 100)
            .with_difficulty(1.0)
            .with_generator(Arc::new(CircleGenerator::new())),

        PhaseDefinition::new("phase2_geometric", "Phase 2: Geometric", 100)
            .with_difficulty(1.5)
            .with_generator(Arc::new(GeometricGenerator::new())),

        PhaseDefinition::new("phase3_civilization", "Phase 3: Civilization", 100)
            .with_difficulty(2.0)
            .with_generator(Arc::new(ClusterGenerator::new())),

        PhaseDefinition::new("phase4_curiosity", "Phase 4: Curiosity", 100)
            .with_difficulty(2.5)
            .with_generator(Arc::new(DynamicGenerator::new())),

        PhaseDefinition::new("phase5_self_improvement", "Phase 5: Self-Improvement", 100)
            .with_difficulty(3.0)
            .with_generator(Arc::new(MetaPatternGenerator::new())),

        // NEW: Phase 6 - Neural Cellular Automata (Emergent Patterns)
        PhaseDefinition::new("phase6_nca_emergence", "Phase 6: NCA Emergence", 100)
            .with_difficulty(3.5)
            .with_generator(Arc::new(NCAGenerator::new())),
    ]
}

// Neural network-based pattern classifier

use super::neural_network::NeuralNetwork;
use super::feature_extractor::FeatureExtractor;
use super::phase_config::Grid;
use std::collections::HashMap;

/// Pattern classifier using neural network
pub struct PatternClassifier {
    network: NeuralNetwork,
    feature_extractor: FeatureExtractor,
    pattern_names: Vec<String>,
    pattern_to_index: HashMap<String, usize>,
}

impl PatternClassifier {
    pub fn new() -> Self {
        let pattern_names = vec![
            "circle".to_string(),
            "center".to_string(),
            "edges".to_string(),
            "clusters".to_string(),
            "symmetry".to_string(),
            "waves".to_string(),
            "spiral".to_string(),
            "motion".to_string(),
            "gradient".to_string(),
            "boundaries".to_string(),
            "complexity".to_string(),
        ];

        let pattern_to_index: HashMap<String, usize> = pattern_names.iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        let feature_extractor = FeatureExtractor::new();
        let input_size = feature_extractor.feature_count();
        let hidden_size = 32;  // More capacity for complex patterns
        let output_size = pattern_names.len();

        let network = NeuralNetwork::new(input_size, hidden_size, output_size);

        Self {
            network,
            feature_extractor,
            pattern_names,
            pattern_to_index,
        }
    }

    /// Predict which patterns are present in a grid
    pub fn predict_patterns(&self, grid: &Grid, threshold: f64) -> Vec<(String, f64)> {
        let features = self.feature_extractor.extract_all(grid);
        let outputs = self.network.predict(&features);

        outputs.iter()
            .enumerate()
            .filter(|(_, &confidence)| confidence > threshold)
            .map(|(i, &confidence)| (self.pattern_names[i].clone(), confidence))
            .collect()
    }

    /// Get confidence scores for all patterns
    pub fn get_pattern_scores(&self, grid: &Grid) -> HashMap<String, f64> {
        let features = self.feature_extractor.extract_all(grid);
        let outputs = self.network.predict(&features);

        outputs.iter()
            .enumerate()
            .map(|(i, &score)| (self.pattern_names[i].clone(), score))
            .collect()
    }

    /// Train the network on a labeled example
    pub fn train_example(&mut self, grid: &Grid, pattern_labels: &[String]) -> f64 {
        let features = self.feature_extractor.extract_all(grid);

        // Create target vector (multi-label: 1.0 for present patterns, 0.0 for absent)
        let mut targets = vec![0.0; self.pattern_names.len()];
        for label in pattern_labels {
            if let Some(&index) = self.pattern_to_index.get(label) {
                targets[index] = 1.0;
            }
        }

        self.network.train(&features, &targets)
    }

    /// Train on multiple examples with optional progress callback
    pub fn train_batch(&mut self, examples: &[(Grid, Vec<String>)], epochs: usize, silent: bool) {
        self.train_batch_with_callback(examples, epochs, silent, |_, _| {});
    }

    /// Train with progress callback for visualization
    pub fn train_batch_with_callback<F>(&mut self, examples: &[(Grid, Vec<String>)], epochs: usize, silent: bool, mut callback: F)
    where
        F: FnMut(usize, f64),
    {
        for epoch in 0..epochs {
            let mut total_error = 0.0;
            for (grid, labels) in examples {
                total_error += self.train_example(grid, labels);
            }
            let avg_error = total_error / examples.len() as f64;

            // Call progress callback every epoch
            callback(epoch, avg_error);

            if !silent && epoch % 100 == 0 {
                println!("Classifier Epoch {}: Error = {:.6}", epoch, avg_error);
            }
        }
    }

    /// Generate training data from current rule-based detectors
    /// This allows us to bootstrap the neural network with existing knowledge
    pub fn bootstrap_from_rules(&mut self, num_samples: usize, silent: bool) {
        use crate::learning::phase_config::load_default_phases;

        let phases = load_default_phases();
        let mut training_data = Vec::new();

        if !silent {
            println!("Generating {} training samples from rule-based detectors...", num_samples);
        }

        for _ in 0..num_samples {
            // Generate random grid from random phase
            let phase_idx = rand::random::<usize>() % phases.len();
            let step = rand::random::<usize>() % 100;
            let progress = (step as f64 / 100.0) * 100.0;

            let grid = phases[phase_idx].generate_pattern(step, progress);

            // Get ground truth labels from rule-based detectors
            let labels = self.feature_extractor.detect_patterns(&grid, 0.5);

            training_data.push((grid, labels));
        }

        if !silent {
            println!("Training neural network on {} examples...", training_data.len());
        }
        // Train for 500 epochs to bootstrap the neural classifier properly
        // In TUI mode, this is visualized in real-time on the initialization screen
        self.train_batch(&training_data, 500, silent);
        if !silent {
            println!("Training complete!");
        }
    }

    /// Evaluate accuracy against rule-based detectors
    pub fn evaluate_accuracy(&self, num_samples: usize) -> f64 {
        use crate::learning::phase_config::load_default_phases;

        let phases = load_default_phases();
        let mut correct = 0;
        let mut total = 0;

        for _ in 0..num_samples {
            let phase_idx = rand::random::<usize>() % phases.len();
            let step = rand::random::<usize>() % 100;
            let progress = (step as f64 / 100.0) * 100.0;

            let grid = phases[phase_idx].generate_pattern(step, progress);

            // Ground truth from rules
            let true_patterns: Vec<String> = self.feature_extractor.detect_patterns(&grid, 0.5);

            // Predictions from network
            let predicted_patterns: Vec<String> = self.predict_patterns(&grid, 0.5)
                .into_iter()
                .map(|(name, _)| name)
                .collect();

            // Check if predictions match
            for pattern in &true_patterns {
                total += 1;
                if predicted_patterns.contains(pattern) {
                    correct += 1;
                }
            }
        }

        if total > 0 {
            correct as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.network.set_learning_rate(lr);
    }
}

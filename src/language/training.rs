//! Language Training - Training loop and configuration for SAGE language learning

use crate::language::predictor::CharacterPredictor;
use crate::language::corpus::Corpus;
use std::time::Instant;

/// Training configuration
#[derive(Clone)]
pub struct TrainingConfig {
    /// Learning rate for readout network
    pub learning_rate: f64,
    /// Number of epochs per level
    pub epochs_per_level: usize,
    /// Batch size for training
    pub batch_size: usize,
    /// Loss threshold to advance to next level
    pub advance_threshold: f64,
    /// Temperature for generation during evaluation
    pub eval_temperature: f64,
    /// How often to print progress (in steps)
    pub log_interval: usize,
    /// How often to evaluate (in epochs)
    pub eval_interval: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            epochs_per_level: 100,
            batch_size: 32,
            advance_threshold: 2.0,  // Perplexity threshold
            eval_temperature: 0.8,
            log_interval: 100,
            eval_interval: 10,
        }
    }
}

/// Training metrics
#[derive(Clone, Debug)]
pub struct TrainingMetrics {
    pub epoch: usize,
    pub step: usize,
    pub loss: f64,
    pub accuracy: f64,
    pub perplexity: f64,
    pub level: usize,
    pub level_name: String,
}

/// Language trainer orchestrates the training process
pub struct LanguageTrainer {
    pub predictor: CharacterPredictor,
    pub corpus: Corpus,
    pub config: TrainingConfig,
    pub metrics_history: Vec<TrainingMetrics>,
    pub total_steps: usize,
    pub current_epoch: usize,
}

impl LanguageTrainer {
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            predictor: CharacterPredictor::new(),
            corpus: Corpus::new(),
            config,
            metrics_history: Vec::new(),
            total_steps: 0,
            current_epoch: 0,
        }
    }

    /// Train for one epoch on current level
    pub fn train_epoch(&mut self) -> TrainingMetrics {
        let pairs = self.corpus.get_training_pairs();
        if pairs.is_empty() {
            return self.current_metrics(0.0, 0.0);
        }

        let mut total_loss = 0.0;
        let mut correct = 0;
        let mut count = 0;

        // Shuffle pairs (simple Fisher-Yates)
        let mut shuffled = pairs.clone();
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        shuffled.shuffle(&mut rng);

        for (input, target) in &shuffled {
            let loss = self.predictor.train_step(input, *target, self.config.learning_rate);
            total_loss += loss;

            // Check prediction
            let (predicted, _) = self.predictor.predict_next();
            if predicted == *target {
                correct += 1;
            }

            count += 1;
            self.total_steps += 1;

            if self.total_steps % self.config.log_interval == 0 {
                let avg_loss = total_loss / count as f64;
                let acc = correct as f64 / count as f64;
                eprintln!(
                    "  Step {}: loss={:.4}, acc={:.2}%",
                    self.total_steps, avg_loss, acc * 100.0
                );
            }
        }

        self.current_epoch += 1;

        let avg_loss = total_loss / count.max(1) as f64;
        let accuracy = correct as f64 / count.max(1) as f64;

        self.current_metrics(avg_loss, accuracy)
    }

    /// Get current metrics
    fn current_metrics(&self, loss: f64, accuracy: f64) -> TrainingMetrics {
        TrainingMetrics {
            epoch: self.current_epoch,
            step: self.total_steps,
            loss,
            accuracy,
            perplexity: loss.exp(),
            level: self.corpus.level,
            level_name: self.corpus.level_name().to_string(),
        }
    }

    /// Evaluate on held-out data
    pub fn evaluate(&mut self) -> (f64, f64) {
        let pairs = self.corpus.get_training_pairs();
        if pairs.is_empty() {
            return (0.0, f64::MAX);
        }

        // Use last 20% as test set
        let test_start = pairs.len() * 80 / 100;
        let test_pairs: Vec<(String, char)> = pairs[test_start..].to_vec();

        self.predictor.evaluate(&test_pairs)
    }

    /// Run full training loop
    pub fn train(&mut self) -> Vec<TrainingMetrics> {
        println!("\n========================================");
        println!("  SAGE Language Training");
        println!("========================================\n");
        println!("Config:");
        println!("  Learning rate: {}", self.config.learning_rate);
        println!("  Epochs per level: {}", self.config.epochs_per_level);
        println!("  Advance threshold (perplexity): {}", self.config.advance_threshold);
        println!();

        let start_time = Instant::now();
        let mut all_metrics = Vec::new();

        loop {
            println!("\n--- Level {}: {} ---", self.corpus.level, self.corpus.level_name());
            println!("Training examples: {}", self.corpus.level_size());

            let _level_start_epoch = self.current_epoch;

            for _ in 0..self.config.epochs_per_level {
                let metrics = self.train_epoch();

                if self.current_epoch % self.config.eval_interval == 0 {
                    let (eval_acc, eval_loss) = self.evaluate();
                    let perplexity = eval_loss.exp();

                    println!(
                        "\nEpoch {}: train_loss={:.4}, train_acc={:.2}%, eval_acc={:.2}%, perplexity={:.2}",
                        self.current_epoch, metrics.loss, metrics.accuracy * 100.0,
                        eval_acc * 100.0, perplexity
                    );

                    // Generate sample
                    let sample = self.generate_sample();
                    println!("Sample generation: \"{}\"", sample);

                    all_metrics.push(metrics.clone());
                    self.metrics_history.push(metrics);

                    // Check if ready to advance
                    if perplexity < self.config.advance_threshold {
                        println!("\n✓ Perplexity below threshold! Advancing level...");
                        break;
                    }
                }
            }

            // Try to advance level
            if !self.corpus.advance_level() {
                println!("\n✓ Completed all levels!");
                break;
            }
        }

        let elapsed = start_time.elapsed();
        println!("\n========================================");
        println!("Training Complete!");
        println!("Total time: {:.1}s", elapsed.as_secs_f64());
        println!("Total epochs: {}", self.current_epoch);
        println!("Total steps: {}", self.total_steps);
        println!("========================================\n");

        all_metrics
    }

    /// Generate a sample from the model
    pub fn generate_sample(&mut self) -> String {
        // Pick a short prompt from current level
        let prompt = match self.corpus.level {
            0 => "ab",
            1 => "hel",
            2 => "how are",
            3 => "I am",
            4 => "User: Hi",
            _ => "a",
        };

        self.predictor.generate(prompt, 20, self.config.eval_temperature)
    }

    /// Interactive generation mode
    pub fn interactive_generate(&mut self, prompt: &str, length: usize) -> String {
        self.predictor.generate(prompt, length, self.config.eval_temperature)
    }

    /// Get training history
    pub fn history(&self) -> &[TrainingMetrics] {
        &self.metrics_history
    }

    /// Save/load model (stub for now)
    pub fn save(&self, _path: &str) -> Result<(), std::io::Error> {
        // TODO: Implement serialization
        Ok(())
    }

    pub fn load(&mut self, _path: &str) -> Result<(), std::io::Error> {
        // TODO: Implement deserialization
        Ok(())
    }
}

/// Quick training function for testing
pub fn quick_train(epochs: usize) -> LanguageTrainer {
    let config = TrainingConfig {
        epochs_per_level: epochs,
        log_interval: 50,
        eval_interval: 5,
        ..Default::default()
    };

    let mut trainer = LanguageTrainer::new(config);
    trainer.train();
    trainer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trainer_creation() {
        let trainer = LanguageTrainer::new(TrainingConfig::default());
        assert_eq!(trainer.total_steps, 0);
        assert_eq!(trainer.current_epoch, 0);
    }

    #[test]
    fn test_single_epoch() {
        let config = TrainingConfig {
            epochs_per_level: 1,
            log_interval: 1000,  // Don't log during test
            ..Default::default()
        };
        let mut trainer = LanguageTrainer::new(config);

        let metrics = trainer.train_epoch();
        assert!(metrics.loss.is_finite());
        assert!(metrics.loss > 0.0);
    }
}

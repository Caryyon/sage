//! Character Predictor - Core language model combining NCA reservoir with readout
//!
//! This is the main interface for SAGE's language capability:
//! 1. Encode input text into the NCA grid
//! 2. Let NCA evolve (reservoir processing)
//! 3. Readout network predicts next character

use crate::grid::{Grid, GRID_SIZE};
use crate::nca::UpdateNetwork;
use crate::language::encoder::CharacterEncoder;
use crate::language::readout::ReadoutNetwork;

/// Number of NCA evolution steps per prediction
pub const EVOLUTION_STEPS: usize = 16;

/// Character predictor combining all components
pub struct CharacterPredictor {
    /// Encodes characters into grid patterns
    pub encoder: CharacterEncoder,
    /// NCA network (the "reservoir")
    pub nca: UpdateNetwork,
    /// Trainable readout layer
    pub readout: ReadoutNetwork,
    /// Current grid state
    pub grid: Grid,
    /// Number of evolution steps per prediction
    pub evolution_steps: usize,
}

impl CharacterPredictor {
    /// Create a new predictor with fresh networks
    pub fn new() -> Self {
        // NCA input: 3×3 neighborhood × 22 channels = 198 (perception)
        // Actually it's: 22 channels × 3 (state + sobel_x + sobel_y) = 66
        let perception_size = 66;  // Match existing NCA architecture
        let hidden_size = 128;
        let output_size = 22;  // NUM_CHANNELS

        Self {
            encoder: CharacterEncoder::new(),
            nca: UpdateNetwork::new(perception_size, hidden_size, output_size),
            readout: ReadoutNetwork::new(),
            grid: Grid::new(GRID_SIZE, GRID_SIZE),
            evolution_steps: EVOLUTION_STEPS,
        }
    }

    /// Create predictor with existing NCA weights (transfer learning)
    pub fn with_nca(nca: UpdateNetwork) -> Self {
        Self {
            encoder: CharacterEncoder::new(),
            nca,
            readout: ReadoutNetwork::new(),
            grid: Grid::new(GRID_SIZE, GRID_SIZE),
            evolution_steps: EVOLUTION_STEPS,
        }
    }

    /// Encode input text and evolve NCA
    pub fn process_input(&mut self, input: &str) {
        // Encode text into grid
        self.encoder.encode_to_grid(input, &mut self.grid);

        // Evolve NCA (reservoir processing)
        for _ in 0..self.evolution_steps {
            self.evolve_step();
        }
    }

    /// Single NCA evolution step
    fn evolve_step(&mut self) {
        use crate::grid::perceive;
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut new_cells = self.grid.cells.clone();

        for y in 0..GRID_SIZE {
            for x in 0..GRID_SIZE {
                // Stochastic update (50% of cells)
                if rng.gen::<f64>() > 0.5 {
                    continue;
                }

                let perception = perceive(&self.grid, y, x);
                let (_, update) = self.nca.forward(&perception);

                // Apply update (residual)
                for c in 0..new_cells[y][x].len() {
                    new_cells[y][x][c] = (self.grid.cells[y][x][c] + update[c]).clamp(0.0, 1.0);
                }
            }
        }

        self.grid.cells = new_cells;
    }

    /// Predict next character given current grid state
    pub fn predict_next(&self) -> (char, f64) {
        let (idx, probs) = self.readout.predict(&self.grid);
        let confidence = probs[idx];
        let c = self.encoder.index_to_char(idx);
        (c, confidence)
    }

    /// Sample next character with temperature
    pub fn sample_next(&self, temperature: f64) -> char {
        let idx = self.readout.sample(&self.grid, temperature);
        self.encoder.index_to_char(idx)
    }

    /// Train on a single example: input -> target_char
    pub fn train_step(&mut self, input: &str, target_char: char, learning_rate: f64) -> f64 {
        // Process input through encoder and NCA
        self.process_input(input);

        // Get target index
        let target_idx = self.encoder.char_to_index(target_char);

        // Train readout
        self.readout.train_step(&self.grid, target_idx, learning_rate)
    }

    /// Generate text continuation
    pub fn generate(&mut self, prompt: &str, max_length: usize, temperature: f64) -> String {
        let mut text = prompt.to_string();

        for _ in 0..max_length {
            // Process current text
            self.process_input(&text);

            // Sample next character
            let next_char = self.sample_next(temperature);

            // Stop on certain characters or if repeating too much
            if next_char == '\0' {
                break;
            }

            text.push(next_char);

            // Prevent infinite loops on repetition
            if text.len() > 4 {
                let last_4: String = text.chars().rev().take(4).collect();
                if last_4.chars().all(|c| c == last_4.chars().next().unwrap()) {
                    break;
                }
            }
        }

        text
    }

    /// Evaluate on a test set (returns accuracy and average loss)
    pub fn evaluate(&mut self, test_data: &[(String, char)]) -> (f64, f64) {
        let mut correct = 0;
        let mut total_loss = 0.0;

        for (input, target) in test_data {
            self.process_input(input);
            let (predicted_char, _) = self.predict_next();

            if predicted_char == *target {
                correct += 1;
            }

            let target_idx = self.encoder.char_to_index(*target);
            let (_, probs) = self.readout.predict(&self.grid);
            total_loss += self.readout.compute_loss(&probs, target_idx);
        }

        let accuracy = correct as f64 / test_data.len() as f64;
        let avg_loss = total_loss / test_data.len() as f64;

        (accuracy, avg_loss)
    }

    /// Get perplexity (exp of average loss)
    pub fn perplexity(&self, avg_loss: f64) -> f64 {
        avg_loss.exp()
    }

    /// Reset grid to empty state
    pub fn reset(&mut self) {
        self.grid = Grid::new(GRID_SIZE, GRID_SIZE);
    }
}

impl Default for CharacterPredictor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_prediction() {
        let mut predictor = CharacterPredictor::new();

        // Process some input
        predictor.process_input("hello");

        // Should be able to predict something
        let (c, confidence) = predictor.predict_next();
        assert!(confidence > 0.0);
        assert!(confidence <= 1.0);
        println!("Predicted: '{}' with confidence {:.3}", c, confidence);
    }

    #[test]
    fn test_training_step() {
        let mut predictor = CharacterPredictor::new();

        // Train on a simple example
        let loss = predictor.train_step("hell", 'o', 0.001);

        assert!(loss.is_finite());
        assert!(loss > 0.0);
    }

    #[test]
    fn test_generation() {
        let mut predictor = CharacterPredictor::new();

        // Generate some text (won't be meaningful without training)
        let generated = predictor.generate("hi", 10, 1.0);

        assert!(generated.starts_with("hi"));
        assert!(generated.len() >= 2);
    }
}

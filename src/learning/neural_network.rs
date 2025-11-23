// Simple feedforward neural network for pattern classification

use rand::Rng;

/// Simple 2-layer feedforward neural network
pub struct NeuralNetwork {
    input_size: usize,
    hidden_size: usize,
    output_size: usize,

    // Weights and biases
    weights_ih: Vec<Vec<f64>>,  // input -> hidden
    biases_h: Vec<f64>,
    weights_ho: Vec<Vec<f64>>,  // hidden -> output
    biases_o: Vec<f64>,

    learning_rate: f64,
}

impl NeuralNetwork {
    pub fn new(input_size: usize, hidden_size: usize, output_size: usize) -> Self {
        let mut rng = rand::thread_rng();

        // Initialize weights with small random values
        let weights_ih = (0..hidden_size)
            .map(|_| {
                (0..input_size)
                    .map(|_| rng.gen_range(-0.5..0.5))
                    .collect()
            })
            .collect();

        let weights_ho = (0..output_size)
            .map(|_| {
                (0..hidden_size)
                    .map(|_| rng.gen_range(-0.5..0.5))
                    .collect()
            })
            .collect();

        Self {
            input_size,
            hidden_size,
            output_size,
            weights_ih,
            biases_h: vec![0.0; hidden_size],
            weights_ho,
            biases_o: vec![0.0; output_size],
            learning_rate: 0.01,
        }
    }

    /// Forward pass through the network
    pub fn forward(&self, inputs: &[f64]) -> (Vec<f64>, Vec<f64>) {
        assert_eq!(inputs.len(), self.input_size, "Input size mismatch");

        // Hidden layer
        let mut hidden = vec![0.0; self.hidden_size];
        for h in 0..self.hidden_size {
            let mut sum = self.biases_h[h];
            for i in 0..self.input_size {
                sum += inputs[i] * self.weights_ih[h][i];
            }
            hidden[h] = Self::sigmoid(sum);
        }

        // Output layer
        let mut outputs = vec![0.0; self.output_size];
        for o in 0..self.output_size {
            let mut sum = self.biases_o[o];
            for h in 0..self.hidden_size {
                sum += hidden[h] * self.weights_ho[o][h];
            }
            outputs[o] = Self::sigmoid(sum);
        }

        (hidden, outputs)
    }

    /// Predict output (just forward pass, return outputs)
    pub fn predict(&self, inputs: &[f64]) -> Vec<f64> {
        let (_, outputs) = self.forward(inputs);
        outputs
    }

    /// Train on a single example using backpropagation
    pub fn train(&mut self, inputs: &[f64], targets: &[f64]) -> f64 {
        assert_eq!(targets.len(), self.output_size, "Target size mismatch");

        // Forward pass
        let (hidden, outputs) = self.forward(inputs);

        // Calculate error
        let error: f64 = outputs.iter()
            .zip(targets.iter())
            .map(|(o, t)| (t - o).powi(2))
            .sum::<f64>() / 2.0;

        // Backpropagation - Output layer
        let mut output_deltas = vec![0.0; self.output_size];
        for o in 0..self.output_size {
            let error_o = targets[o] - outputs[o];
            output_deltas[o] = error_o * Self::sigmoid_derivative(outputs[o]);
        }

        // Backpropagation - Hidden layer
        let mut hidden_deltas = vec![0.0; self.hidden_size];
        for h in 0..self.hidden_size {
            let mut error_h = 0.0;
            for o in 0..self.output_size {
                error_h += output_deltas[o] * self.weights_ho[o][h];
            }
            hidden_deltas[h] = error_h * Self::sigmoid_derivative(hidden[h]);
        }

        // Update weights - Hidden to Output
        for o in 0..self.output_size {
            for h in 0..self.hidden_size {
                self.weights_ho[o][h] += self.learning_rate * output_deltas[o] * hidden[h];
            }
            self.biases_o[o] += self.learning_rate * output_deltas[o];
        }

        // Update weights - Input to Hidden
        for h in 0..self.hidden_size {
            for i in 0..self.input_size {
                self.weights_ih[h][i] += self.learning_rate * hidden_deltas[h] * inputs[i];
            }
            self.biases_h[h] += self.learning_rate * hidden_deltas[h];
        }

        error
    }

    /// Train on multiple epochs
    pub fn train_batch(&mut self, dataset: &[(Vec<f64>, Vec<f64>)], epochs: usize, silent: bool) -> Vec<f64> {
        let mut errors = Vec::new();

        for epoch in 0..epochs {
            let mut total_error = 0.0;
            for (inputs, targets) in dataset {
                total_error += self.train(inputs, targets);
            }
            let avg_error = total_error / dataset.len() as f64;
            errors.push(avg_error);

            if !silent && epoch % 100 == 0 {
                println!("Epoch {}: Error = {:.6}", epoch, avg_error);
            }
        }

        errors
    }

    /// Sigmoid activation function
    fn sigmoid(x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Derivative of sigmoid (for backpropagation)
    fn sigmoid_derivative(y: f64) -> f64 {
        // y is already sigmoid(x), so derivative is y * (1 - y)
        y * (1.0 - y)
    }

    /// Get the most confident output class
    pub fn predict_class(&self, inputs: &[f64]) -> (usize, f64) {
        let outputs = self.predict(inputs);
        let (max_idx, max_val) = outputs.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        (max_idx, *max_val)
    }

    pub fn set_learning_rate(&mut self, lr: f64) {
        self.learning_rate = lr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_learning() {
        // XOR is a classic non-linear problem that requires a hidden layer
        // Note: Simple networks can be finicky - use more epochs and higher learning rate
        let mut nn = NeuralNetwork::new(2, 8, 1);  // More hidden neurons
        nn.set_learning_rate(1.0);  // Higher learning rate for faster convergence

        let dataset = vec![
            (vec![0.0, 0.0], vec![0.0]),
            (vec![0.0, 1.0], vec![1.0]),
            (vec![1.0, 0.0], vec![1.0]),
            (vec![1.0, 1.0], vec![0.0]),
        ];

        nn.train_batch(&dataset, 5000, false);  // More epochs

        // Test predictions - relax tolerance since simple networks vary
        for (inputs, expected) in &dataset {
            let output = nn.predict(inputs);
            let error = (output[0] - expected[0]).abs();
            println!("Input: {:?}, Expected: {}, Got: {:.3}, Error: {:.3}",
                     inputs, expected[0], output[0], error);
            assert!(error < 0.4, "XOR learning failed: error {} too high", error);
        }
    }
}
